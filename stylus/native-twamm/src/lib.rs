// SPDX-License-Identifier: GPL-2.0-or-later
//! A time-weighted average market maker as a Uniswap v4 hook, in pure Rust for Arbitrum Stylus.
//!
//! Long-term orders sell one token into the pool at a constant rate per second. Nothing runs on a
//! schedule: the orders are folded into the pool lazily, by whoever next touches it, and the price
//! they arrive at is computed in closed form rather than simulated second by second.
//!
//! # Why this hook
//!
//! Every other hook benchmarked in this repository loses to its Solidity twin, because Stylus buys
//! a lower marginal cost of computation at the price of a fixed cost per call, and a hook that
//! counts swaps or reads a vault never does enough arithmetic to earn that back. TWAMM does. It is
//! Uniswap's own v4-periphery example, its published cost is roughly 100,000 gas per expiry
//! interval, and almost all of that is `ABDKMathQuad` — IEEE 754 binary128 emulated in software
//! because the EVM has no floating point.
//!
//! Stylus has no floating point either, so this works in 1e18 fixed point (see [`math`]), and
//! `uniswap/src/TwammHook.sol` carries the identical fixed-point form. The benchmark compares that
//! pair, so what it measures is the language rather than the algorithm.
//!
//! # What talks to Solidity
//!
//! Nothing in this crate, except the ABI. There is no Solidity shell in front of it: the address is
//! CREATE2-mined so its low bits carry the permission flags v4 reads, the ten `IHooks` callbacks
//! are `stylus-uniswap-v4`'s, and the calls back into the singleton — `extsload`, `unlock`, `swap`,
//! `sync`, `settle`, `take` — go through [`PoolManagerCalls`]. The pool manager is Solidity because
//! it is Uniswap's; everything on this side of the boundary is Rust.
//!
//! # Simplifications
//!
//! Shared with the Solidity twin, so the comparison stays honest:
//!
//! * liquidity is taken as constant across an executed span. A production TWAMM splits the span at
//!   every initialised tick the virtual price crosses.
//! * earnings come from the closed form, and the AMM is then swapped to the price it predicts, so
//!   pool fees and any tick crossing land on this contract's own balance.
//! * orders may only expire on a fixed [`EXPIRATION_INTERVAL`] grid.
//!
//! Derived from the TWAMM design published by Paradigm (2021) and from Uniswap's `TWAMM.sol`
//! example, which is GPL-2.0. No code is copied from it, but the architecture is recognisably
//! theirs, so this file and its Solidity twin carry GPL-2.0-or-later rather than the repository's
//! MIT.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

pub mod math;

use alloc::{vec, vec::Vec};

use alloy_primitives::{
    aliases::{I24, U160},
    keccak256, Address, FixedBytes, I256, U256,
};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{
    abi::Bytes,
    call::call,
    prelude::*,
    storage::{StorageMap, StorageU256, StorageU8},
};
use stylus_uniswap_v4::{
    hooks::{selector, HookConfig, HookGuards, IHooks, IUnlockCallback},
    types::{amount0, amount1, BeforeSwapDelta, ModifyLiquidityParams, PoolKey, SwapParams, U24},
    Permissions, PoolManagerCalls, ZERO_DELTA,
};

include!(concat!(env!("OUT_DIR"), "/pool_manager.rs"));

/// Orders may only expire on multiples of this many seconds.
///
/// A constant rather than a constructor argument because Stylus has no `immutable`: holding it
/// would mean a cold storage read on every callback, which would show up in the benchmark as a
/// language difference when it is nothing of the sort. `TwammHook.sol` uses the same number.
pub const EXPIRATION_INTERVAL: u64 = 60;

/// Slot of `PoolManager._pools`, as `StateLibrary.POOLS_SLOT`.
const POOLS_SLOT: u8 = 6;
/// Offset of `Pool.State.liquidity`, as `StateLibrary.LIQUIDITY_OFFSET`.
const LIQUIDITY_OFFSET: u64 = 3;
/// Length of the blob [`encode_unlock`] hands to the pool manager.
const UNLOCK_PAYLOAD: usize = 87;

sol! {
    /// The address baked in at build time is not the one this deployment was given.
    #[derive(Debug)]
    error PoolManagerMismatch(address baked, address given);
    /// The hook has never seen this pool initialised.
    #[derive(Debug)]
    error PoolNotInitialized();
    /// Orders may only expire on the interval grid.
    #[derive(Debug)]
    error ExpirationNotOnInterval(uint256 expiration);
    /// An order cannot end in the past.
    #[derive(Debug)]
    error ExpirationInThePast(uint256 expiration);
    /// The amount is too small to sell even a wei per second over the order's life.
    #[derive(Debug)]
    error ZeroSellRate();
    /// No such order, or it has already been cancelled.
    #[derive(Debug)]
    error NoOrder();
    /// An ERC-20 refused a transfer.
    #[derive(Debug)]
    error TokenCallFailed();
    /// `unlockCallback` was reached other than through the pool manager.
    #[derive(Debug)]
    error NotPoolManager();
    /// The pool manager returned something this hook could not decode.
    #[derive(Debug)]
    error MalformedReturn();
}

/// `transfer(address,uint256)`.
const ERC20_TRANSFER: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];
/// `transferFrom(address,address,uint256)`.
const ERC20_TRANSFER_FROM: [u8; 4] = [0x23, 0xb8, 0x72, 0xdd];

/// One word of ABI head, left-padded.
fn word(value: U256) -> [u8; 32] {
    value.to_be_bytes()
}

fn address_word(value: Address) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(value.as_slice());
    out
}

/// One side of the market: everyone selling the same token at the same time.
#[storage]
pub struct OrderPool {
    /// Tokens per second, scaled by 1e18, summed over every live order.
    sell_rate_current: StorageU256,
    /// Cumulative proceeds per unit of sell rate. An order's earnings are the growth in this since
    /// it last looked, times its own rate.
    earnings_factor_current: StorageU256,
    /// Rate leaving the pool when the clock passes each expiry.
    sell_rate_ending_at_interval: StorageMap<U256, StorageU256>,
    /// `earnings_factor_current` as it stood at each expiry, so an order can still be settled long
    /// after it ended.
    earnings_factor_at_interval: StorageMap<U256, StorageU256>,
}

#[storage]
pub struct Order {
    sell_rate: StorageU256,
    earnings_factor_last: StorageU256,
}

#[storage]
pub struct PoolState {
    /// Non-zero once the pool has been initialised; the clock the orders run against.
    last_virtual_order_timestamp: StorageU256,
    order_pool_0_for_1: OrderPool,
    order_pool_1_for_0: OrderPool,
    orders: StorageMap<FixedBytes<32>, Order>,
}

#[storage]
#[entrypoint]
pub struct TwammHook {
    pools: StorageMap<FixedBytes<32>, PoolState>,
    _pad: StorageU8,
}

impl HookConfig for TwammHook {
    fn pool_manager(&self) -> Address {
        POOL_MANAGER
    }

    fn permissions(&self) -> Permissions {
        Permissions::none()
            .with_before_initialize()
            .with_before_add_liquidity()
            .with_before_swap()
    }
}

/// The identity of an order: who placed it, which way it sells, and when it ends.
///
/// This is `keccak256(abi.encode(address,bool,uint256))` written out by hand — three left-padded
/// words — rather than through `SolValue`, whose generic encoder costs more contract space than
/// the whole of the rest of this function.
fn order_id(owner: Address, zero_for_one: bool, expiration: U256) -> FixedBytes<32> {
    let mut buf = [0u8; 96];
    buf[12..32].copy_from_slice(owner.as_slice());
    buf[63] = zero_for_one as u8;
    buf[64..].copy_from_slice(&expiration.to_be_bytes::<32>());
    keccak256(buf)
}

/// What `unlockCallback` needs to finish the job, packed by hand.
///
/// The pool manager hands this blob straight back without looking at it, and the only reader is
/// [`decode_unlock`] below, so there is nothing to be gained from ABI-encoding it and a kilobyte
/// of contract space to be lost.
fn encode_unlock(key: &PoolKey, zero_for_one: bool, target: U256) -> Vec<u8> {
    let mut out = Vec::with_capacity(UNLOCK_PAYLOAD);
    out.extend_from_slice(key.currency0.as_slice());
    out.extend_from_slice(key.currency1.as_slice());
    out.extend_from_slice(&key.fee.to_be_bytes::<3>());
    out.extend_from_slice(&key.tickSpacing.into_raw().to_be_bytes::<3>());
    out.extend_from_slice(key.hooks.as_slice());
    out.push(zero_for_one as u8);
    out.extend_from_slice(&target.to_be_bytes::<32>()[12..]);
    out
}

fn decode_unlock(data: &[u8]) -> Option<(PoolKey, bool, U256)> {
    if data.len() != UNLOCK_PAYLOAD {
        return None;
    }
    let key = PoolKey {
        currency0: Address::from_slice(&data[0..20]),
        currency1: Address::from_slice(&data[20..40]),
        fee: U24::from_be_slice(&data[40..43]),
        tickSpacing: I24::from_raw(U24::from_be_slice(&data[43..46])),
        hooks: Address::from_slice(&data[46..66]),
    };
    Some((key, data[66] != 0, U256::from_be_slice(&data[67..87])))
}

#[public]
#[implements(IHooks, IUnlockCallback)]
impl TwammHook {
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        if pool_manager != POOL_MANAGER {
            return Err(PoolManagerMismatch {
                baked: POOL_MANAGER,
                given: pool_manager,
            }
            .abi_encode());
        }
        HookGuards::validate_hook_address(self)
    }

    pub fn pool_manager(&self) -> Address {
        POOL_MANAGER
    }

    pub fn expiration_interval(&self) -> U256 {
        U256::from(EXPIRATION_INTERVAL)
    }

    // --- reads ---------------------------------------------------------------------------------

    pub fn order_id(&self, owner: Address, zero_for_one: bool, expiration: U256) -> FixedBytes<32> {
        order_id(owner, zero_for_one, expiration)
    }

    pub fn last_virtual_order_timestamp(&self, pool_id: FixedBytes<32>) -> U256 {
        self.pools.get(pool_id).last_virtual_order_timestamp.get()
    }

    pub fn sell_rate_current(&self, pool_id: FixedBytes<32>, zero_for_one: bool) -> U256 {
        let state = self.pools.get(pool_id);
        if zero_for_one {
            state.order_pool_0_for_1.sell_rate_current.get()
        } else {
            state.order_pool_1_for_0.sell_rate_current.get()
        }
    }

    pub fn earnings_factor_current(&self, pool_id: FixedBytes<32>, zero_for_one: bool) -> U256 {
        let state = self.pools.get(pool_id);
        if zero_for_one {
            state.order_pool_0_for_1.earnings_factor_current.get()
        } else {
            state.order_pool_1_for_0.earnings_factor_current.get()
        }
    }

    pub fn get_order(
        &self,
        pool_id: FixedBytes<32>,
        owner: Address,
        zero_for_one: bool,
        expiration: U256,
    ) -> (U256, U256) {
        let state = self.pools.get(pool_id);
        let order = state.orders.get(order_id(owner, zero_for_one, expiration));
        (order.sell_rate.get(), order.earnings_factor_last.get())
    }

    /// What `claimProceeds` would pay out as of the last execution.
    pub fn proceeds_of(
        &self,
        pool_id: FixedBytes<32>,
        owner: Address,
        zero_for_one: bool,
        expiration: U256,
    ) -> U256 {
        let (rate, last) = self.get_order(pool_id, owner, zero_for_one, expiration);
        if rate.is_zero() {
            return U256::ZERO;
        }
        let factor = self.settled_factor(pool_id, zero_for_one, expiration);
        factor.saturating_sub(last) * rate / math::wad()
    }

    // --- orders --------------------------------------------------------------------------------

    /// Sell `amount_in` of one token into the pool at a constant rate until `expiration`.
    ///
    /// The tokens are pulled now; the proceeds accrue continuously and come out through
    /// [`Self::claim_proceeds`]. Adding to an order that already exists settles what it has earned
    /// so far, so both tranches can share one earnings factor.
    pub fn submit_order(
        &mut self,
        key: PoolKey,
        zero_for_one: bool,
        expiration: U256,
        amount_in: U256,
    ) -> Result<FixedBytes<32>, Vec<u8>> {
        let interval = U256::from(EXPIRATION_INTERVAL);
        if !(expiration % interval).is_zero() {
            return Err(ExpirationNotOnInterval { expiration }.abi_encode());
        }
        let now = U256::from(self.vm().block_timestamp());
        if expiration <= now {
            return Err(ExpirationInThePast { expiration }.abi_encode());
        }

        let pool_id = key.to_id();
        self.execute_virtual_orders_inner(&key, false)?;

        let rate = amount_in * math::wad() / (expiration - now);
        if rate.is_zero() {
            return Err(ZeroSellRate {}.abi_encode());
        }

        let owner = self.vm().msg_sender();
        let id = order_id(owner, zero_for_one, expiration);

        // Settle first: an existing order's factor is about to be overwritten.
        let existing = self.pools.get(pool_id).orders.get(id).sell_rate.get();
        if !existing.is_zero() {
            self.pay_proceeds(&key, zero_for_one, expiration, owner)?;
        }

        let factor_now = self.earnings_factor_current(pool_id, zero_for_one);
        {
            let mut state = self.pools.setter(pool_id);
            let pool = if zero_for_one {
                &mut state.order_pool_0_for_1
            } else {
                &mut state.order_pool_1_for_0
            };
            let current = pool.sell_rate_current.get();
            pool.sell_rate_current.set(current + rate);
            let ending = pool.sell_rate_ending_at_interval.get(expiration);
            pool.sell_rate_ending_at_interval
                .setter(expiration)
                .set(ending + rate);
        }
        {
            let mut state = self.pools.setter(pool_id);
            let mut order = state.orders.setter(id);
            let held = order.sell_rate.get();
            order.sell_rate.set(held + rate);
            order.earnings_factor_last.set(factor_now);
        }

        let sold = if zero_for_one {
            key.currency0
        } else {
            key.currency1
        };
        self.pull(sold, owner, amount_in)?;
        Ok(id)
    }

    /// Withdraw everything an order has earned so far. Callable while it is still running.
    pub fn claim_proceeds(
        &mut self,
        key: PoolKey,
        zero_for_one: bool,
        expiration: U256,
    ) -> Result<U256, Vec<u8>> {
        self.execute_virtual_orders_inner(&key, false)?;
        let owner = self.vm().msg_sender();
        let id = order_id(owner, zero_for_one, expiration);
        if self
            .pools
            .get(key.to_id())
            .orders
            .get(id)
            .sell_rate
            .get()
            .is_zero()
        {
            return Err(NoOrder {}.abi_encode());
        }
        self.pay_proceeds(&key, zero_for_one, expiration, owner)
    }

    /// Stop a still-running order, returning what it earned and the principal not yet sold.
    pub fn cancel_order(
        &mut self,
        key: PoolKey,
        zero_for_one: bool,
        expiration: U256,
    ) -> Result<(U256, U256), Vec<u8>> {
        self.execute_virtual_orders_inner(&key, false)?;
        let now = U256::from(self.vm().block_timestamp());
        if expiration <= now {
            return Err(ExpirationInThePast { expiration }.abi_encode());
        }

        let pool_id = key.to_id();
        let owner = self.vm().msg_sender();
        let id = order_id(owner, zero_for_one, expiration);
        let rate = self.pools.get(pool_id).orders.get(id).sell_rate.get();
        if rate.is_zero() {
            return Err(NoOrder {}.abi_encode());
        }

        let proceeds = self.pay_proceeds(&key, zero_for_one, expiration, owner)?;

        {
            let mut state = self.pools.setter(pool_id);
            let pool = if zero_for_one {
                &mut state.order_pool_0_for_1
            } else {
                &mut state.order_pool_1_for_0
            };
            let current = pool.sell_rate_current.get();
            pool.sell_rate_current.set(current - rate);
            let ending = pool.sell_rate_ending_at_interval.get(expiration);
            pool.sell_rate_ending_at_interval
                .setter(expiration)
                .set(ending - rate);
        }
        self.pools
            .setter(pool_id)
            .orders
            .setter(id)
            .sell_rate
            .set(U256::ZERO);

        let refund = rate * (expiration - now) / math::wad();
        if !refund.is_zero() {
            let sold = if zero_for_one {
                key.currency0
            } else {
                key.currency1
            };
            self.push(sold, owner, refund)?;
        }
        Ok((proceeds, refund))
    }

    /// `n` intervals of the closed form, each feeding its price into the next.
    ///
    /// Not used by the hook. It is the arithmetic benchmark's entry point, and `TwammHook.sol` has
    /// the same function, so the two can be timed against each other with no storage and no pool
    /// manager in the way.
    pub fn work_fixed(&self, n: U256) -> U256 {
        let e18 = math::wad();
        let mut price = e18;
        let mut i = U256::ZERO;
        while i < n {
            price = math::two_sided(
                price,
                U256::from(1_000_000u64) * e18,
                U256::from(3) * e18 + i * (e18 / U256::from(10)),
                U256::from(5) * e18,
                U256::from(600),
            );
            i += U256::from(1);
        }
        price
    }

    /// Bring the pool's long-term orders up to the current block.
    ///
    /// Anyone may call this — it is how a TWAMM stays current when nobody is trading — and unlike
    /// the hook callbacks it runs outside the pool manager's lock, so it has to take one.
    pub fn execute_virtual_orders(&mut self, key: PoolKey) -> Result<(), Vec<u8>> {
        self.execute_virtual_orders_inner(&key, false)
    }
}

impl TwammHook {
    /// The earnings factor an order that ends at `expiration` settles against: the snapshot taken
    /// when it expired, or the running total if it is still alive.
    fn settled_factor(
        &self,
        pool_id: FixedBytes<32>,
        zero_for_one: bool,
        expiration: U256,
    ) -> U256 {
        let state = self.pools.get(pool_id);
        let pool = if zero_for_one {
            &state.order_pool_0_for_1
        } else {
            &state.order_pool_1_for_0
        };
        if expiration <= state.last_virtual_order_timestamp.get() {
            pool.earnings_factor_at_interval.get(expiration)
        } else {
            pool.earnings_factor_current.get()
        }
    }

    /// Pays out an order's accrued proceeds and marks it settled. Assumes the caller has already
    /// executed virtual orders.
    fn pay_proceeds(
        &mut self,
        key: &PoolKey,
        zero_for_one: bool,
        expiration: U256,
        to: Address,
    ) -> Result<U256, Vec<u8>> {
        let pool_id = key.to_id();
        let id = order_id(to, zero_for_one, expiration);
        let factor = self.settled_factor(pool_id, zero_for_one, expiration);

        let amount = {
            let mut state = self.pools.setter(pool_id);
            let mut order = state.orders.setter(id);
            let rate = order.sell_rate.get();
            let last = order.earnings_factor_last.get();
            order.earnings_factor_last.set(factor);
            factor.saturating_sub(last) * rate / math::wad()
        };

        if !amount.is_zero() {
            // A 0-for-1 order sells token0 and is therefore paid in token1.
            let bought = if zero_for_one {
                key.currency1
            } else {
                key.currency0
            };
            self.push(bought, to, amount)?;
        }
        Ok(amount)
    }

    fn pull(&mut self, token: Address, from: Address, amount: U256) -> Result<(), Vec<u8>> {
        let this = self.vm().contract_address();
        let mut calldata = Vec::with_capacity(100);
        calldata.extend_from_slice(&ERC20_TRANSFER_FROM);
        calldata.extend_from_slice(&address_word(from));
        calldata.extend_from_slice(&address_word(this));
        calldata.extend_from_slice(&word(amount));
        self.erc20_call(token, calldata)
    }

    fn push(&mut self, token: Address, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(68);
        calldata.extend_from_slice(&ERC20_TRANSFER);
        calldata.extend_from_slice(&address_word(to));
        calldata.extend_from_slice(&word(amount));
        self.erc20_call(token, calldata)
    }

    /// Sends one ERC-20 call and insists it succeeded.
    ///
    /// Written out rather than declared with `sol_interface!` because the generic encoder costs
    /// several kilobytes of contract space, and this contract has to fit in one Stylus fragment to
    /// be deployable at a mined address at all.
    ///
    /// A token that returns nothing is accepted, as `SafeERC20` does; one that returns a false
    /// word is not.
    fn erc20_call(&mut self, token: Address, calldata: Vec<u8>) -> Result<(), Vec<u8>> {
        let context = Call::new_mutating(self);
        let returned = call(self.vm(), context, token, &calldata)
            .map_err(|_| TokenCallFailed {}.abi_encode())?;
        if returned.len() >= 32 && returned[..32].iter().all(|b| *b == 0) {
            return Err(TokenCallFailed {}.abi_encode());
        }
        Ok(())
    }

    /// `keccak256(abi.encodePacked(poolId, POOLS_SLOT))`, as `StateLibrary._getPoolStateSlot`.
    fn pool_state_slot(pool_id: FixedBytes<32>) -> U256 {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(pool_id.as_slice());
        buf[63] = POOLS_SLOT;
        U256::from_be_bytes(keccak256(buf).0)
    }

    /// v4 exposes pool state through `extsload` rather than getters, so this reads the two words
    /// the closed form needs: the price out of `slot0`, and the liquidity three slots down.
    fn read_pool(&self, pool_id: FixedBytes<32>) -> Result<(U256, U256), Vec<u8>> {
        let base = Self::pool_state_slot(pool_id);
        let slot0 = self.extsload(FixedBytes::from(base.to_be_bytes::<32>()))?;
        let liquidity_slot = base + U256::from(LIQUIDITY_OFFSET);
        let liquidity = self.extsload(FixedBytes::from(liquidity_slot.to_be_bytes::<32>()))?;

        // slot0 packs sqrtPriceX96 into its low 160 bits, liquidity is a uint128 in its own slot.
        let sqrt_price_x96 =
            U256::from_be_bytes(slot0.0) & ((U256::from(1u8) << 160) - U256::from(1));
        let liquidity =
            U256::from_be_bytes(liquidity.0) & ((U256::from(1u8) << 128) - U256::from(1));
        Ok((sqrt_price_x96, liquidity))
    }

    /// One span of constant sell rates: move the virtual price, and credit both order pools with
    /// what the move earned them.
    fn apply_span(
        &mut self,
        pool_id: FixedBytes<32>,
        sqrt_price: U256,
        liquidity: U256,
        elapsed: U256,
    ) -> U256 {
        let (rate0, rate1) = {
            let state = self.pools.get(pool_id);
            (
                state.order_pool_0_for_1.sell_rate_current.get(),
                state.order_pool_1_for_0.sell_rate_current.get(),
            )
        };
        let (next, earned0, earned1) = math::advance(sqrt_price, liquidity, rate0, rate1, elapsed);

        let mut state = self.pools.setter(pool_id);
        if !rate0.is_zero() {
            let pool = &mut state.order_pool_0_for_1;
            let factor = pool.earnings_factor_current.get();
            pool.earnings_factor_current
                .set(factor + earned0 * math::wad() / rate0);
        }
        if !rate1.is_zero() {
            let pool = &mut state.order_pool_1_for_0;
            let factor = pool.earnings_factor_current.get();
            pool.earnings_factor_current
                .set(factor + earned1 * math::wad() / rate1);
        }
        next
    }

    /// Walks the clock forward one expiry at a time, because the sell rates change at each one and
    /// the closed form is only valid while they hold constant. Every span updates both earnings
    /// factors; the real pool price is pushed once, at the end.
    /// `unlocked` says whether the pool manager's lock is already held. Inside a hook callback it
    /// is — v4 took it before calling us, and taking it again reverts with `AlreadyUnlocked`.
    /// Outside one it is not, and the swap has to happen inside a fresh `unlock`.
    fn execute_virtual_orders_inner(
        &mut self,
        key: &PoolKey,
        unlocked: bool,
    ) -> Result<(), Vec<u8>> {
        let pool_id = key.to_id();
        let mut from = self.pools.get(pool_id).last_virtual_order_timestamp.get();
        if from.is_zero() {
            return Err(PoolNotInitialized {}.abi_encode());
        }
        let to = U256::from(self.vm().block_timestamp());
        if from >= to {
            return Ok(());
        }

        let (rate0, rate1) = {
            let state = self.pools.get(pool_id);
            (
                state.order_pool_0_for_1.sell_rate_current.get(),
                state.order_pool_1_for_0.sell_rate_current.get(),
            )
        };
        // Nothing to sell: move the clock and skip the arithmetic and both pool manager reads.
        if rate0.is_zero() && rate1.is_zero() {
            self.pools
                .setter(pool_id)
                .last_virtual_order_timestamp
                .set(to);
            return Ok(());
        }

        let (sqrt_price_x96, liquidity) = self.read_pool(pool_id)?;
        let mut sqrt_price = math::from_sqrt_x96(sqrt_price_x96);

        let interval = U256::from(EXPIRATION_INTERVAL);
        let mut next = from + interval - (from % interval);
        while next <= to {
            let (ending0, ending1) = {
                let state = self.pools.get(pool_id);
                (
                    state
                        .order_pool_0_for_1
                        .sell_rate_ending_at_interval
                        .get(next),
                    state
                        .order_pool_1_for_0
                        .sell_rate_ending_at_interval
                        .get(next),
                )
            };
            if !ending0.is_zero() || !ending1.is_zero() {
                sqrt_price = self.apply_span(pool_id, sqrt_price, liquidity, next - from);
                from = next;
                // Orders ending here stop selling, and settle against the factor as it stands at
                // this instant however long their owners take to come and claim.
                let mut state = self.pools.setter(pool_id);
                {
                    let pool = &mut state.order_pool_0_for_1;
                    let rate = pool.sell_rate_current.get();
                    pool.sell_rate_current.set(rate - ending0);
                    let factor = pool.earnings_factor_current.get();
                    pool.earnings_factor_at_interval.setter(next).set(factor);
                }
                {
                    let pool = &mut state.order_pool_1_for_0;
                    let rate = pool.sell_rate_current.get();
                    pool.sell_rate_current.set(rate - ending1);
                    let factor = pool.earnings_factor_current.get();
                    pool.earnings_factor_at_interval.setter(next).set(factor);
                }
            }
            next += interval;
        }
        if from < to {
            sqrt_price = self.apply_span(pool_id, sqrt_price, liquidity, to - from);
        }

        // Written before the swap, not after: moving the pool re-enters `before_swap`, and this
        // is what makes the second pass a no-op instead of an infinite recursion.
        self.pools
            .setter(pool_id)
            .last_virtual_order_timestamp
            .set(to);

        let target = math::to_sqrt_x96(sqrt_price);
        if target != sqrt_price_x96 {
            let zero_for_one = target < sqrt_price_x96;
            if unlocked {
                self.move_price(key.clone(), zero_for_one, target)?;
            } else {
                self.unlock(encode_unlock(key, zero_for_one, target))?;
            }
        }
        Ok(())
    }

    /// Moves the real pool to the price the virtual orders arrived at, and squares the books.
    ///
    /// `amount_specified` is deliberately unbounded — the price limit is what stops the swap, which
    /// is how the AMM lands exactly where the closed form said it would.
    fn move_price(
        &mut self,
        key: PoolKey,
        zero_for_one: bool,
        target: U256,
    ) -> Result<(), Vec<u8>> {
        let params = SwapParams {
            zeroForOne: zero_for_one,
            amountSpecified: I256::MAX,
            sqrtPriceLimitX96: U160::from_be_slice(&target.to_be_bytes::<32>()[12..]),
        };
        let delta = self.swap(key.clone(), params, Vec::new())?;
        self.settle_delta(key.currency0, amount0(delta))?;
        self.settle_delta(key.currency1, amount1(delta))
    }

    /// A negative delta is owed to the manager, a positive one is owed to this hook.
    fn settle_delta(&mut self, currency: Address, amount: i128) -> Result<(), Vec<u8>> {
        if amount < 0 {
            let owed = U256::from(amount.unsigned_abs());
            let manager = self.pool_manager();
            self.sync(currency)?;
            self.push(currency, manager, owed)?;
            self.settle()?;
        } else if amount > 0 {
            let this = self.vm().contract_address();
            self.take(currency, this, U256::from(amount as u128))?;
        }
        Ok(())
    }
}

#[public]
impl IUnlockCallback for TwammHook {
    /// Moves the real pool to the price the virtual orders arrived at, and squares the books.
    ///
    /// `amount_specified` is deliberately unbounded — the price limit is what stops the swap.
    fn unlock_callback(&mut self, data: Bytes) -> Result<Bytes, Vec<u8>> {
        if self.vm().msg_sender() != POOL_MANAGER {
            return Err(NotPoolManager {}.abi_encode());
        }
        let (key, zero_for_one, target) =
            decode_unlock(&data.0).ok_or_else(|| MalformedReturn {}.abi_encode())?;
        self.move_price(key, zero_for_one, target)?;
        Ok(Vec::new().into())
    }
}

#[public]
impl IHooks for TwammHook {
    fn before_initialize(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _sqrt_price_x96: U160,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        let now = U256::from(self.vm().block_timestamp());
        self.pools
            .setter(key.to_id())
            .last_virtual_order_timestamp
            .set(now);
        Ok(selector::BEFORE_INITIALIZE)
    }

    fn before_add_liquidity(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: ModifyLiquidityParams,
        _hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        self.execute_virtual_orders_inner(&key, true)?;
        Ok(selector::BEFORE_ADD_LIQUIDITY)
    }

    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        self.execute_virtual_orders_inner(&key, true)?;
        Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    /// Low bits: beforeInitialize | beforeAddLiquidity | beforeSwap.
    const HOOK: Address = Address::new([
        0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe,
        0xef, 0x00, 0x00, 0x28, 0x80,
    ]);

    fn deployed(vm: &TestVM) -> TwammHook {
        vm.set_contract_address(HOOK);
        let mut hook = TwammHook::from(vm);
        hook.constructor(POOL_MANAGER).unwrap();
        hook
    }

    fn d(s: &str) -> U256 {
        U256::from_str_radix(s, 10).unwrap()
    }

    /// The address the constructor accepts has to be the one v4 will read the permissions out of,
    /// so this pins the flags the miner is asked for against the ones the hook declares.
    #[test]
    fn the_mined_address_carries_the_declared_permissions() {
        let vm = TestVM::default();
        let hook = deployed(&vm);
        let flags = hook.permissions().flags();
        assert_eq!(
            U256::from(flags) & U256::from(0x2880u32),
            U256::from(0x2880u32)
        );
    }

    /// `TwammHook.sol` asserts these same numbers. If the two ever diverge the benchmark is
    /// comparing two different algorithms and means nothing.
    #[test]
    fn the_benchmark_kernel_matches_the_solidity_twin() {
        let vm = TestVM::default();
        let hook = deployed(&vm);
        assert_eq!(hook.work_fixed(U256::ZERO), d("1000000000000000000"));
        assert_eq!(hook.work_fixed(U256::from(1)), d("1001197841728773871"));
        assert_eq!(hook.work_fixed(U256::from(2)), d("1002331270278227884"));
        assert_eq!(hook.work_fixed(U256::from(3)), d("1003400248490110669"));
    }

    #[test]
    fn an_order_id_is_the_owner_the_side_and_the_expiry() {
        let vm = TestVM::default();
        let hook = deployed(&vm);
        let a = Address::from([1u8; 20]);
        let b = Address::from([2u8; 20]);
        let t = U256::from(600);
        assert_ne!(hook.order_id(a, true, t), hook.order_id(b, true, t));
        assert_ne!(hook.order_id(a, true, t), hook.order_id(a, false, t));
        assert_ne!(
            hook.order_id(a, true, t),
            hook.order_id(a, true, t + U256::from(60))
        );
        assert_eq!(hook.order_id(a, true, t), hook.order_id(a, true, t));
    }

    #[test]
    fn orders_must_expire_on_the_grid_and_in_the_future() {
        let vm = TestVM::default();
        let mut hook = deployed(&vm);
        let key = key();
        let off_grid = hook.submit_order(key.clone(), true, U256::from(90), U256::from(1000));
        assert_eq!(
            off_grid.unwrap_err(),
            ExpirationNotOnInterval {
                expiration: U256::from(90)
            }
            .abi_encode()
        );

        vm.set_block_timestamp(1_000);
        let past = hook.submit_order(key, true, U256::from(600), U256::from(1000));
        assert_eq!(
            past.unwrap_err(),
            ExpirationInThePast {
                expiration: U256::from(600)
            }
            .abi_encode()
        );
    }

    /// A pool the hook has never seen initialised has no clock to run the orders against.
    #[test]
    fn an_uninitialised_pool_is_refused() {
        let vm = TestVM::default();
        let mut hook = deployed(&vm);
        vm.set_block_timestamp(1_000);
        let err = hook
            .submit_order(key(), true, U256::from(1_200), U256::from(1000))
            .unwrap_err();
        assert_eq!(err, PoolNotInitialized {}.abi_encode());
    }

    fn key() -> PoolKey {
        PoolKey {
            currency0: Address::from([0xa0; 20]),
            currency1: Address::from([0xb0; 20]),
            fee: U24::ZERO,
            tickSpacing: alloy_primitives::aliases::I24::unchecked_from(60),
            hooks: HOOK,
        }
    }

    /// The accounting property the whole contract rests on: a span's proceeds are split between
    /// the orders in a pool strictly in proportion to their sell rates, whatever the price did.
    #[test]
    fn a_span_splits_its_proceeds_by_sell_rate() {
        let vm = TestVM::default();
        let mut hook = deployed(&vm);
        let pool_id = key().to_id();
        let e18 = math::wad();

        // Two sellers of token0, one at 1/s and one at 2/s, against a token1 seller at 5/s.
        let (small, large) = (e18, U256::from(2) * e18);
        {
            let mut state = hook.pools.setter(pool_id);
            state.last_virtual_order_timestamp.set(U256::from(1_000));
            state
                .order_pool_0_for_1
                .sell_rate_current
                .set(small + large);
            state
                .order_pool_1_for_0
                .sell_rate_current
                .set(U256::from(5) * e18);
        }

        let next = hook.apply_span(pool_id, e18, U256::from(1_000_000u64), U256::from(600));
        assert_eq!(
            next,
            d("1001197841728773871"),
            "the span must use the closed form"
        );

        let factor0 = hook.earnings_factor_current(pool_id, true);
        let factor1 = hook.earnings_factor_current(pool_id, false);
        assert!(!factor0.is_zero() && !factor1.is_zero());

        let paid_small = factor0 * small / e18;
        let paid_large = factor0 * large / e18;
        assert_eq!(paid_large, paid_small * U256::from(2));
        // And the two of them together are the whole of what the pool earned, to within the wei
        // that integer division rounds off.
        let total = factor0 * (small + large) / e18;
        assert!(total - (paid_small + paid_large) <= U256::from(1));
    }

    /// With nothing being sold there is no price to move and no reason to touch the pool manager.
    #[test]
    fn an_idle_pool_only_moves_its_clock() {
        let vm = TestVM::default();
        let mut hook = deployed(&vm);
        let key = key();
        let pool_id = key.to_id();
        hook.pools
            .setter(pool_id)
            .last_virtual_order_timestamp
            .set(U256::from(1_000));

        vm.set_block_timestamp(5_000);
        hook.execute_virtual_orders(key).unwrap();
        assert_eq!(
            hook.last_virtual_order_timestamp(pool_id),
            U256::from(5_000)
        );
    }
}
