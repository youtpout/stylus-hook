// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, Signed, U256};
use alloy_sol_types::{sol, SolError};

use stylus_sdk::{crypto::keccak, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct LimitOrder {
        uint256 epoch_next;
        address hook;
        mapping(bytes32 => int32) tick_lower_lasts;
        mapping(bytes32 => uint256) epochs;
        mapping(uint256 => EpochInfo) epoch_infos;
        address pool_manager;
    }

    pub struct EpochInfo {
        bool filled;
        address currency0;
        address currency1;
        uint256 token0_total;
        uint256 token1_total;
        uint128 liquidity_total;
        mapping(address => uint128) liquidity;
    }

}

sol! {
    error ZeroLiquidity();
    error InRange();
    error CrossedRange();
    error Filled();
    error NotFilled();
    error NotPoolManagerToken();
}

sol_interface! {
    interface IPoolManager {
        function getSlot0(bytes32 id) external view returns (uint160 sqrtPriceX96, int24 tick, uint16 protocolFee, uint24 swapFee);
    }
}

#[derive(SolidityError)]
pub enum HookError {
    ZeroLiquidity(ZeroLiquidity),
    InRange(InRange),
    CrossedRange(CrossedRange),
    Filled(Filled),
    NotFilled(NotFilled),
    NotPoolManagerToken(NotPoolManagerToken)
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl LimitOrder {
    pub fn set_pool_manager(&mut self, value: Address) -> Result<()> {
        self.pool_manager.set(value);
        Ok(())
    }

    pub fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    pub fn tick_lower_lasts(&self, pool_id: FixedBytes<32>) -> i32 {
        self.tick_lower_lasts.get(pool_id).as_i32()
    }

    pub fn get_tick_lower_last(&self, pool_id: FixedBytes<32>) -> i32 {
        self.tick_lower_lasts.get(pool_id).as_i32()
    }

    fn set_tick_lower_lasts(&mut self, pool_id: FixedBytes<32>, tick_lower: i32) -> Result<()> {
        // why I need cas between i32 and Signed<32,i> is boring
        self.tick_lower_lasts
            .replace(pool_id, Signed::<32, 1>::unchecked_from(tick_lower));
        return Ok(());
    }

    pub fn epochs(&self, pool_id: FixedBytes<32>) -> U256 {
        self.epochs.get(pool_id)
    }

    pub fn get_epoch_liquidity(&self, epoch: U256, owner: Address) -> u128 {
        let epoch_info = self.epoch_infos.get(epoch);
        epoch_info.liquidity.get(owner).to::<u128>()
    }

    pub fn epoch(&self, id: FixedBytes<32>) -> U256 {
        self.epochs.get(id)
    }

    pub fn get_epoch(&self, pool_id: FixedBytes<32>, tick_lower: i32, zero_for_one: bool) -> U256 {
        let hash = keccak(
            &[
                pool_id,
                keccak(tick_lower.to_be_bytes()),
                keccak([u8::from(zero_for_one)]),
            ]
            .concat(),
        );
        self.epochs.get(hash)
    }

    fn set_epoch(
        &mut self,
        pool_id: FixedBytes<32>,
        tick_lower: i32,
        zero_for_one: bool,
        epoch: U256,
    ) -> Result<()> {
        let hash = keccak(
            &[
                pool_id,
                keccak(tick_lower.to_be_bytes()),
                keccak([u8::from(zero_for_one)]),
            ]
            .concat(),
        );
        self.epochs.replace(hash, epoch);
        return Ok(());
    }

    pub fn epoch_next(&self) -> U256 {
        self.epoch_next.get()
    }

    pub fn after_initialize(
        &mut self,
        pool_id: FixedBytes<32>,
        tick: i32,
        tick_spacing: i32,
    ) -> Result<()> {
        let last = self.get_tick_lower(tick, tick_spacing);
        let converted: Signed<32, 1> = Signed::<32, 1>::unchecked_from(last);
        self.tick_lower_lasts.replace(pool_id, converted);
        return Ok(());
    }

    pub fn after_swap(
        &mut self,
        pool_id: FixedBytes<32>,
        tick: i32,
        tick_spacing: i32,
    ) -> Result<()> {
        let crossed_ticks: (i32, i32, i32) = self.get_crossed_ticks(pool_id, tick_spacing)?;
        if (crossed_ticks.1 > crossed_ticks.2) {
            return Ok(());
        }

        // note that a zeroForOne swap means that the pool is actually gaining token0, so limit
        // order fills are the opposite of swap fills, hence the inversion below
        return Ok(());
    }

    /*fn fill_epoch(  &mut self,
        pool_id: FixedBytes<32>,
        tick: i32,
        tick_spacing: i32) -> Result<()> {
            // the calc is different from original limit order to match stylus type
            keccak([pool_id, lower ])
        let epoch = getEpoch(key, lower, zeroForOne);
        if (!epoch.equals(EPOCH_DEFAULT)) {
            EpochInfo storage epochInfo = epochInfos[epoch];

            epochInfo.filled = true;

            (uint256 amount0, uint256 amount1) =
                _lockAcquiredFill(key, lower, -int256(uint256(epochInfo.liquidityTotal)));

            unchecked {
                epochInfo.token0Total += amount0;
                epochInfo.token1Total += amount1;
            }

            setEpoch(key, lower, zeroForOne, EPOCH_DEFAULT);

            emit Fill(epoch, key, lower, zeroForOne);
        }
    }*/

    fn get_crossed_ticks(
        &self,
        pool_id: FixedBytes<32>,
        tick_spacing: i32,
    ) -> Result<(i32, i32, i32)> {
        //let tick_lower =
        let tick = self.get_tick(pool_id)?;
        let tick_lower = self.get_tick_lower(tick, tick_spacing);
        let tick_lower_last = self.get_tick_lower_last(pool_id);
        let lower;
        let upper;
        if tick_lower_last < tick_lower_last {
            lower = tick_lower + tick_spacing;
            upper = tick_lower_last;
        } else {
            lower = tick_lower_last;
            upper = tick_lower - tick_spacing;
        }
        return Ok((tick_lower, lower, upper));
    }

    fn get_tick(&self, pool_id: FixedBytes<32>) -> Result<i32, HookError> {
        //let tick_lower =
        let pool_slot = IPoolManager::new(self.pool_manager.get()).get_slot_0(self, pool_id).ok().unwrap();
        return Ok(pool_slot.1);
    }

    fn get_tick_lower(&self, tick: i32, tick_spacing: i32) -> i32 {
        let mut compressed = tick / tick_spacing;
        if tick < 0 && tick % tick_spacing != 0 {
            compressed -= 1; // round towards negative infinity
        }
        return compressed * tick_spacing;
    }
}
