// SPDX-License-Identifier: MIT
//! Calling back into the v4 singleton. A hook returning a non-zero delta has to balance its own
//! books before the lock closes.
//!
//! The calldata is written by hand rather than through `SolCall`, which cost about 8 KB of contract
//! space; the tests below check each encoding byte for byte against what `SolCall` would produce.

use alloc::vec::Vec;

use alloy_primitives::{
    aliases::{I24, U24},
    Address, FixedBytes, I256, U256,
};
use alloy_sol_types::SolError;
use stylus_sdk::{
    call::{call, static_call},
    prelude::*,
};

use crate::hooks::HookConfig;
use crate::types::{BalanceDelta, ModifyLiquidityParams, PoolKey, SwapParams};

alloy_sol_types::sol! {
    /// The pool manager returned data this hook could not decode.
    #[derive(Debug)]
    error MalformedPoolManagerReturn();
}

/// The method IDs v4-core dispatches on, from the compiled `IPoolManager.sol`
/// (`forge inspect IPoolManager methodIdentifiers`).
pub mod selector {
    pub const UNLOCK: [u8; 4] = [0x48, 0xc8, 0x94, 0x91];
    pub const MODIFY_LIQUIDITY: [u8; 4] = [0x5a, 0x6b, 0xcf, 0xda];
    pub const SWAP: [u8; 4] = [0xf3, 0xcd, 0x91, 0x4c];
    pub const DONATE: [u8; 4] = [0x23, 0x42, 0x66, 0xd7];
    pub const SYNC: [u8; 4] = [0xa5, 0x84, 0x11, 0x94];
    pub const TAKE: [u8; 4] = [0x0b, 0x0d, 0x9c, 0x09];
    pub const SETTLE: [u8; 4] = [0x11, 0xda, 0x60, 0xb4];
    pub const SETTLE_FOR: [u8; 4] = [0x3d, 0xd4, 0x5a, 0xdb];
    pub const CLEAR: [u8; 4] = [0x80, 0xf0, 0xb4, 0x4c];
    pub const MINT: [u8; 4] = [0x15, 0x6e, 0x29, 0xf6];
    pub const BURN: [u8; 4] = [0xf5, 0x29, 0x8a, 0xca];
    pub const UPDATE_DYNAMIC_LP_FEE: [u8; 4] = [0x52, 0x75, 0x96, 0x51];
    pub const EXTSLOAD: [u8; 4] = [0x1e, 0x2e, 0xae, 0xaf];
    pub const EXTTLOAD: [u8; 4] = [0xf1, 0x35, 0xba, 0xaa];
}

/// One ABI word.
fn word(value: U256) -> [u8; 32] {
    value.to_be_bytes()
}

/// An `address`, right-aligned in its word.
fn address_word(value: Address) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(value.as_slice());
    out
}

/// A `bool`, as the ABI writes it.
fn bool_word(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = value as u8;
    out
}

/// A `uint24`, right-aligned.
fn u24_word(value: U24) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[29..].copy_from_slice(&value.to_be_bytes::<3>());
    out
}

/// An `int24`, sign-extended across the whole word — the part that is easy to get wrong, since a
/// negative tick spacing is `0xff..ff` down to its last three bytes.
fn i24_word(value: I24) -> [u8; 32] {
    let mut out = if value.is_negative() {
        [0xffu8; 32]
    } else {
        [0u8; 32]
    };
    out[29..].copy_from_slice(&value.into_raw().to_be_bytes::<3>());
    out
}

/// An `int256`, which is already two's complement in its own bytes.
fn i256_word(value: I256) -> [u8; 32] {
    value.to_be_bytes()
}

/// The five static words of a `PoolKey`.
fn push_pool_key(out: &mut Vec<u8>, key: &PoolKey) {
    out.extend_from_slice(&address_word(key.currency0));
    out.extend_from_slice(&address_word(key.currency1));
    out.extend_from_slice(&u24_word(key.fee));
    out.extend_from_slice(&i24_word(key.tickSpacing));
    out.extend_from_slice(&address_word(key.hooks));
}

/// A trailing `bytes` argument: the offset goes in the head, the payload after it.
fn push_tail_bytes(out: &mut Vec<u8>, head_words: usize, data: &[u8]) {
    out.extend_from_slice(&word(U256::from(head_words * 32)));
    out.extend_from_slice(&word(U256::from(data.len())));
    out.extend_from_slice(data);
    let padding = (32 - data.len() % 32) % 32;
    out.extend_from_slice(&[0u8; 32][..padding]);
}

fn malformed() -> Vec<u8> {
    MalformedPoolManagerReturn {}.abi_encode()
}

/// The first word of a return, as a `uint256`.
fn first_word(returned: &[u8]) -> Result<U256, Vec<u8>> {
    if returned.len() < 32 {
        return Err(malformed());
    }
    Ok(U256::from_be_slice(&returned[..32]))
}

/// The v4 callbacks a hook may make while the manager is unlocked.
pub trait PoolManagerCalls: HookConfig + HostAccess + TopLevelStorage + Sized {
    /// Sends one state-changing call to the pool manager and hands back its raw return data.
    fn call_pool_manager(&mut self, calldata: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        let target = self.pool_manager();
        let context = Call::new_mutating(self);
        call(self.vm(), context, target, &calldata).map_err(Vec::from)
    }

    /// Sends one read-only call to the pool manager and hands back its raw return data.
    fn static_call_pool_manager(&self, calldata: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        static_call(self.vm(), Call::new(), self.pool_manager(), &calldata).map_err(Vec::from)
    }

    /// Takes the manager's lock and re-enters through `unlockCallback`.
    ///
    /// Only needed by a hook that acts outside a callback; inside one the manager is already
    /// unlocked.
    fn unlock(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        let mut calldata = Vec::with_capacity(100 + data.len());
        calldata.extend_from_slice(&selector::UNLOCK);
        push_tail_bytes(&mut calldata, 1, &data);
        let returned = self.call_pool_manager(calldata)?;
        // `bytes memory`: an offset, then a length, then the payload.
        if returned.len() < 64 {
            return Err(malformed());
        }
        let len = U256::from_be_slice(&returned[32..64]);
        let len: usize = len.try_into().map_err(|_| malformed())?;
        returned
            .get(64..64 + len)
            .map(<[u8]>::to_vec)
            .ok_or_else(malformed)
    }

    /// Records the manager's balance of `currency`, so a later [`settle`] can price the difference.
    ///
    /// [`settle`]: PoolManagerCalls::settle
    fn sync(&mut self, currency: Address) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(36);
        calldata.extend_from_slice(&selector::SYNC);
        calldata.extend_from_slice(&address_word(currency));
        self.call_pool_manager(calldata)?;
        Ok(())
    }

    /// Withdraws `amount` of `currency` to `to`, debiting this hook's delta.
    fn take(&mut self, currency: Address, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(100);
        calldata.extend_from_slice(&selector::TAKE);
        calldata.extend_from_slice(&address_word(currency));
        calldata.extend_from_slice(&address_word(to));
        calldata.extend_from_slice(&word(amount));
        self.call_pool_manager(calldata)?;
        Ok(())
    }

    /// Pays what this hook owes, crediting its delta. Returns the amount credited.
    fn settle(&mut self) -> Result<U256, Vec<u8>> {
        let returned = self.call_pool_manager(selector::SETTLE.to_vec())?;
        first_word(&returned)
    }

    /// Pays on `recipient`'s behalf, crediting their delta instead of this hook's.
    fn settle_for(&mut self, recipient: Address) -> Result<U256, Vec<u8>> {
        let mut calldata = Vec::with_capacity(36);
        calldata.extend_from_slice(&selector::SETTLE_FOR);
        calldata.extend_from_slice(&address_word(recipient));
        let returned = self.call_pool_manager(calldata)?;
        first_word(&returned)
    }

    /// Writes off `amount` of a positive delta without withdrawing it — for dust too small to be
    /// worth a transfer.
    fn clear(&mut self, currency: Address, amount: U256) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(68);
        calldata.extend_from_slice(&selector::CLEAR);
        calldata.extend_from_slice(&address_word(currency));
        calldata.extend_from_slice(&word(amount));
        self.call_pool_manager(calldata)?;
        Ok(())
    }

    /// Mints ERC-6909 claim tokens against a positive delta, instead of taking the currency out.
    fn mint_claims(&mut self, to: Address, id: U256, amount: U256) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(100);
        calldata.extend_from_slice(&selector::MINT);
        calldata.extend_from_slice(&address_word(to));
        calldata.extend_from_slice(&word(id));
        calldata.extend_from_slice(&word(amount));
        self.call_pool_manager(calldata)?;
        Ok(())
    }

    /// Burns ERC-6909 claim tokens to pay a negative delta.
    fn burn_claims(&mut self, from: Address, id: U256, amount: U256) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(100);
        calldata.extend_from_slice(&selector::BURN);
        calldata.extend_from_slice(&address_word(from));
        calldata.extend_from_slice(&word(id));
        calldata.extend_from_slice(&word(amount));
        self.call_pool_manager(calldata)?;
        Ok(())
    }

    /// Donates to the in-range liquidity providers of a pool.
    fn donate(
        &mut self,
        key: PoolKey,
        amount0: U256,
        amount1: U256,
        hook_data: Vec<u8>,
    ) -> Result<BalanceDelta, Vec<u8>> {
        let mut calldata = Vec::with_capacity(324 + hook_data.len());
        calldata.extend_from_slice(&selector::DONATE);
        push_pool_key(&mut calldata, &key);
        calldata.extend_from_slice(&word(amount0));
        calldata.extend_from_slice(&word(amount1));
        push_tail_bytes(&mut calldata, 8, &hook_data);
        let returned = self.call_pool_manager(calldata)?;
        Ok(I256::from_raw(first_word(&returned)?))
    }

    /// Adds or removes liquidity on behalf of this hook.
    fn modify_liquidity(
        &mut self,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Vec<u8>,
    ) -> Result<(BalanceDelta, BalanceDelta), Vec<u8>> {
        let mut calldata = Vec::with_capacity(388 + hook_data.len());
        calldata.extend_from_slice(&selector::MODIFY_LIQUIDITY);
        push_pool_key(&mut calldata, &key);
        calldata.extend_from_slice(&i24_word(params.tickLower));
        calldata.extend_from_slice(&i24_word(params.tickUpper));
        calldata.extend_from_slice(&i256_word(params.liquidityDelta));
        calldata.extend_from_slice(params.salt.as_slice());
        push_tail_bytes(&mut calldata, 10, &hook_data);
        let returned = self.call_pool_manager(calldata)?;
        if returned.len() < 64 {
            return Err(malformed());
        }
        Ok((
            I256::from_raw(U256::from_be_slice(&returned[..32])),
            I256::from_raw(U256::from_be_slice(&returned[32..64])),
        ))
    }

    /// Swaps on behalf of this hook. Re-entering the pool the hook is attached to is rejected by
    /// v4, so this is for routing through *other* pools — or, from inside `unlockCallback`, for
    /// moving a pool the hook manages to a price it has already computed.
    fn swap(
        &mut self,
        key: PoolKey,
        params: SwapParams,
        hook_data: Vec<u8>,
    ) -> Result<BalanceDelta, Vec<u8>> {
        let mut calldata = Vec::with_capacity(356 + hook_data.len());
        calldata.extend_from_slice(&selector::SWAP);
        push_pool_key(&mut calldata, &key);
        calldata.extend_from_slice(&bool_word(params.zeroForOne));
        calldata.extend_from_slice(&i256_word(params.amountSpecified));
        calldata.extend_from_slice(&word(U256::from(params.sqrtPriceLimitX96)));
        push_tail_bytes(&mut calldata, 9, &hook_data);
        let returned = self.call_pool_manager(calldata)?;
        Ok(I256::from_raw(first_word(&returned)?))
    }

    /// Sets the LP fee of a pool that was initialised with the dynamic-fee flag.
    fn update_dynamic_lp_fee(&mut self, key: PoolKey, new_fee: U24) -> Result<(), Vec<u8>> {
        let mut calldata = Vec::with_capacity(196);
        calldata.extend_from_slice(&selector::UPDATE_DYNAMIC_LP_FEE);
        push_pool_key(&mut calldata, &key);
        calldata.extend_from_slice(&u24_word(new_fee));
        self.call_pool_manager(calldata)?;
        Ok(())
    }

    /// Reads one storage slot of the pool manager — v4 exposes its state this way rather than
    /// through getters.
    fn extsload(&self, slot: FixedBytes<32>) -> Result<FixedBytes<32>, Vec<u8>> {
        let mut calldata = Vec::with_capacity(36);
        calldata.extend_from_slice(&selector::EXTSLOAD);
        calldata.extend_from_slice(slot.as_slice());
        let returned = self.static_call_pool_manager(calldata)?;
        Ok(FixedBytes::from(first_word(&returned)?.to_be_bytes::<32>()))
    }

    /// Reads one transient storage slot of the pool manager.
    ///
    /// Note that a Stylus contract can read the manager's transient storage but cannot keep any of
    /// its own: the Stylus host interface exposes no `TSTORE`/`TLOAD` at all.
    fn exttload(&self, slot: FixedBytes<32>) -> Result<FixedBytes<32>, Vec<u8>> {
        let mut calldata = Vec::with_capacity(36);
        calldata.extend_from_slice(&selector::EXTTLOAD);
        calldata.extend_from_slice(slot.as_slice());
        let returned = self.static_call_pool_manager(calldata)?;
        Ok(FixedBytes::from(first_word(&returned)?.to_be_bytes::<32>()))
    }
}

impl<T: HookConfig + HostAccess + TopLevelStorage + Sized> PoolManagerCalls for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IPoolManager;
    use alloy_sol_types::SolCall;

    fn key() -> PoolKey {
        PoolKey {
            currency0: Address::from([0x11; 20]),
            currency1: Address::from([0x22; 20]),
            fee: U24::from(3000),
            // Negative on purpose: the sign extension of an `int24` is the one part of this
            // encoding that a hand-rolled version can plausibly get wrong.
            tickSpacing: I24::unchecked_from(-60),
            hooks: Address::from([0x33; 20]),
        }
    }

    /// The selectors v4-core dispatches on, from the compiled `IPoolManager.sol`
    /// (`forge inspect IPoolManager methodIdentifiers`).
    #[test]
    fn selectors_match_uniswap_ipoolmanager() {
        assert_eq!(selector::UNLOCK, [0x48, 0xc8, 0x94, 0x91]);
        assert_eq!(selector::MODIFY_LIQUIDITY, [0x5a, 0x6b, 0xcf, 0xda]);
        assert_eq!(selector::SWAP, [0xf3, 0xcd, 0x91, 0x4c]);
        assert_eq!(selector::DONATE, [0x23, 0x42, 0x66, 0xd7]);
        assert_eq!(selector::SYNC, [0xa5, 0x84, 0x11, 0x94]);
        assert_eq!(selector::TAKE, [0x0b, 0x0d, 0x9c, 0x09]);
        assert_eq!(selector::SETTLE, [0x11, 0xda, 0x60, 0xb4]);
        assert_eq!(selector::SETTLE_FOR, [0x3d, 0xd4, 0x5a, 0xdb]);
        assert_eq!(selector::CLEAR, [0x80, 0xf0, 0xb4, 0x4c]);
        assert_eq!(selector::MINT, [0x15, 0x6e, 0x29, 0xf6]);
        assert_eq!(selector::BURN, [0xf5, 0x29, 0x8a, 0xca]);
        assert_eq!(selector::EXTSLOAD, [0x1e, 0x2e, 0xae, 0xaf]);
        assert_eq!(selector::EXTTLOAD, [0xf1, 0x35, 0xba, 0xaa]);
        // Not in the original list; taken from the same `forge inspect` output.
        assert_eq!(
            selector::UPDATE_DYNAMIC_LP_FEE,
            IPoolManager::updateDynamicLPFeeCall::SELECTOR
        );
    }

    /// The load-bearing test of this module: what the hand-rolled encoders produce has to be, byte
    /// for byte, what `alloy_sol_types` would have produced from the same interface.
    #[test]
    fn the_hand_rolled_calldata_matches_alloy() {
        let hook_data = b"hook data that is not a multiple of thirty-two bytes".to_vec();

        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::UNLOCK);
        push_tail_bytes(&mut mine, 1, &hook_data);
        assert_eq!(
            mine,
            IPoolManager::unlockCall {
                data: hook_data.clone().into()
            }
            .abi_encode()
        );

        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::TAKE);
        mine.extend_from_slice(&address_word(Address::from([0xaa; 20])));
        mine.extend_from_slice(&address_word(Address::from([0xbb; 20])));
        mine.extend_from_slice(&word(U256::from(12345)));
        assert_eq!(
            mine,
            IPoolManager::takeCall {
                currency: Address::from([0xaa; 20]),
                to: Address::from([0xbb; 20]),
                amount: U256::from(12345),
            }
            .abi_encode()
        );

        let params = SwapParams {
            zeroForOne: true,
            amountSpecified: I256::unchecked_from(-1_000_000i64),
            sqrtPriceLimitX96: alloy_primitives::aliases::U160::from(
                79228162514264337593543950336u128,
            ),
        };
        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::SWAP);
        push_pool_key(&mut mine, &key());
        mine.extend_from_slice(&bool_word(params.zeroForOne));
        mine.extend_from_slice(&i256_word(params.amountSpecified));
        mine.extend_from_slice(&word(U256::from(params.sqrtPriceLimitX96)));
        push_tail_bytes(&mut mine, 9, &hook_data);
        assert_eq!(
            mine,
            IPoolManager::swapCall {
                key: key(),
                params: params.clone(),
                hookData: hook_data.clone().into(),
            }
            .abi_encode()
        );

        let liquidity = ModifyLiquidityParams {
            tickLower: I24::unchecked_from(-887220),
            tickUpper: I24::unchecked_from(887220),
            liquidityDelta: I256::unchecked_from(-42i64),
            salt: FixedBytes::from([0x77; 32]),
        };
        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::MODIFY_LIQUIDITY);
        push_pool_key(&mut mine, &key());
        mine.extend_from_slice(&i24_word(liquidity.tickLower));
        mine.extend_from_slice(&i24_word(liquidity.tickUpper));
        mine.extend_from_slice(&i256_word(liquidity.liquidityDelta));
        mine.extend_from_slice(liquidity.salt.as_slice());
        push_tail_bytes(&mut mine, 10, &hook_data);
        assert_eq!(
            mine,
            IPoolManager::modifyLiquidityCall {
                key: key(),
                params: liquidity.clone(),
                hookData: hook_data.clone().into(),
            }
            .abi_encode()
        );

        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::DONATE);
        push_pool_key(&mut mine, &key());
        mine.extend_from_slice(&word(U256::from(1)));
        mine.extend_from_slice(&word(U256::from(2)));
        push_tail_bytes(&mut mine, 8, &hook_data);
        assert_eq!(
            mine,
            IPoolManager::donateCall {
                key: key(),
                amount0: U256::from(1),
                amount1: U256::from(2),
                hookData: hook_data.into(),
            }
            .abi_encode()
        );

        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::UPDATE_DYNAMIC_LP_FEE);
        push_pool_key(&mut mine, &key());
        mine.extend_from_slice(&u24_word(U24::from(500)));
        assert_eq!(
            mine,
            IPoolManager::updateDynamicLPFeeCall {
                key: key(),
                newDynamicLPFee: U24::from(500),
            }
            .abi_encode()
        );

        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::EXTSLOAD);
        mine.extend_from_slice(FixedBytes::<32>::from([0x99; 32]).as_slice());
        assert_eq!(
            mine,
            IPoolManager::extsloadCall {
                slot: FixedBytes::from([0x99; 32])
            }
            .abi_encode()
        );

        assert_eq!(
            selector::SETTLE.to_vec(),
            IPoolManager::settleCall {}.abi_encode()
        );
    }

    /// An empty `bytes` is the common case for a hook that passes no data on, and it is also the
    /// case where the padding arithmetic can slip and append a stray word.
    #[test]
    fn an_empty_bytes_argument_is_encoded_without_padding() {
        let mut mine = Vec::new();
        mine.extend_from_slice(&selector::UNLOCK);
        push_tail_bytes(&mut mine, 1, &[]);
        assert_eq!(mine.len(), 4 + 64);
        assert_eq!(
            mine,
            IPoolManager::unlockCall {
                data: Vec::new().into()
            }
            .abi_encode()
        );
    }
}
