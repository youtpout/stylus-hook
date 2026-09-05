// SPDX-License-Identifier: MIT OR Apache-2.0
//! Calling back into the v4 singleton from a hook.
//!
//! A hook that only observes swaps never needs this. A hook that takes a share of one does: v4
//! settles in deltas, so returning a non-zero `BeforeSwapDelta` or `BalanceDelta` obliges the hook
//! to balance its own books with [`PoolManagerCalls::take`], [`PoolManagerCalls::settle`] and
//! friends before the lock closes.
//!
//! Import the trait where the callbacks are implemented; it is blanket-implemented for every hook
//! that declares a [`HookConfig`].

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::{SolCall, SolError};
use stylus_sdk::{
    call::{call, static_call},
    prelude::*,
};

use crate::hooks::HookConfig;
use crate::types::{BalanceDelta, IPoolManager, ModifyLiquidityParams, PoolKey, SwapParams};

alloy_sol_types::sol! {
    /// The pool manager returned data this hook could not decode.
    #[derive(Debug)]
    error MalformedPoolManagerReturn();
}

/// The v4 callbacks a hook may make while the manager is unlocked.
pub trait PoolManagerCalls: HookConfig + HostAccess + TopLevelStorage + Sized {
    /// Sends one state-changing call to the pool manager and decodes what comes back.
    fn call_pool_manager<C: SolCall>(&mut self, request: C) -> Result<C::Return, Vec<u8>> {
        let target = self.pool_manager();
        let calldata = request.abi_encode();
        let context = Call::new_mutating(self);
        let returned = call(self.vm(), context, target, &calldata)?;
        C::abi_decode_returns(&returned).map_err(|_| MalformedPoolManagerReturn {}.abi_encode())
    }

    /// Sends one read-only call to the pool manager and decodes what comes back.
    fn static_call_pool_manager<C: SolCall>(&self, request: C) -> Result<C::Return, Vec<u8>> {
        let calldata = request.abi_encode();
        let returned = static_call(self.vm(), Call::new(), self.pool_manager(), &calldata)?;
        C::abi_decode_returns(&returned).map_err(|_| MalformedPoolManagerReturn {}.abi_encode())
    }

    /// Takes the manager's lock and re-enters through `unlockCallback`.
    ///
    /// Only needed by a hook that acts outside a callback; inside one the manager is already
    /// unlocked.
    fn unlock(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        self.call_pool_manager(IPoolManager::unlockCall { data: data.into() })
            .map(|returned| returned.to_vec())
    }

    /// Records the manager's balance of `currency`, so a later [`settle`] can price the difference.
    ///
    /// [`settle`]: PoolManagerCalls::settle
    fn sync(&mut self, currency: Address) -> Result<(), Vec<u8>> {
        self.call_pool_manager(IPoolManager::syncCall { currency })?;
        Ok(())
    }

    /// Withdraws `amount` of `currency` to `to`, debiting this hook's delta.
    fn take(&mut self, currency: Address, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.call_pool_manager(IPoolManager::takeCall {
            currency,
            to,
            amount,
        })?;
        Ok(())
    }

    /// Pays what this hook owes, crediting its delta. Returns the amount credited.
    fn settle(&mut self) -> Result<U256, Vec<u8>> {
        self.call_pool_manager(IPoolManager::settleCall {})
    }

    /// Pays on `recipient`'s behalf, crediting their delta instead of this hook's.
    fn settle_for(&mut self, recipient: Address) -> Result<U256, Vec<u8>> {
        self.call_pool_manager(IPoolManager::settleForCall { recipient })
    }

    /// Writes off `amount` of a positive delta without withdrawing it — for dust too small to be
    /// worth a transfer.
    fn clear(&mut self, currency: Address, amount: U256) -> Result<(), Vec<u8>> {
        self.call_pool_manager(IPoolManager::clearCall { currency, amount })?;
        Ok(())
    }

    /// Mints ERC-6909 claim tokens against a positive delta, instead of taking the currency out.
    fn mint_claims(&mut self, to: Address, id: U256, amount: U256) -> Result<(), Vec<u8>> {
        self.call_pool_manager(IPoolManager::mintCall { to, id, amount })?;
        Ok(())
    }

    /// Burns ERC-6909 claim tokens to pay a negative delta.
    fn burn_claims(&mut self, from: Address, id: U256, amount: U256) -> Result<(), Vec<u8>> {
        self.call_pool_manager(IPoolManager::burnCall { from, id, amount })?;
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
        self.call_pool_manager(IPoolManager::donateCall {
            key,
            amount0,
            amount1,
            hookData: hook_data.into(),
        })
    }

    /// Adds or removes liquidity on behalf of this hook.
    fn modify_liquidity(
        &mut self,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Vec<u8>,
    ) -> Result<(BalanceDelta, BalanceDelta), Vec<u8>> {
        let returns = self.call_pool_manager(IPoolManager::modifyLiquidityCall {
            key,
            params,
            hookData: hook_data.into(),
        })?;
        Ok((returns.callerDelta, returns.feesAccrued))
    }

    /// Swaps on behalf of this hook. Re-entering the pool the hook is attached to is rejected by
    /// v4, so this is for routing through *other* pools.
    fn swap(
        &mut self,
        key: PoolKey,
        params: SwapParams,
        hook_data: Vec<u8>,
    ) -> Result<BalanceDelta, Vec<u8>> {
        self.call_pool_manager(IPoolManager::swapCall {
            key,
            params,
            hookData: hook_data.into(),
        })
    }

    /// Sets the LP fee of a pool that was initialised with the dynamic-fee flag.
    fn update_dynamic_lp_fee(
        &mut self,
        key: PoolKey,
        new_fee: crate::types::U24,
    ) -> Result<(), Vec<u8>> {
        self.call_pool_manager(IPoolManager::updateDynamicLPFeeCall {
            key,
            newDynamicLPFee: new_fee,
        })?;
        Ok(())
    }

    /// Reads one storage slot of the pool manager — v4 exposes its state this way rather than
    /// through getters.
    fn extsload(&self, slot: FixedBytes<32>) -> Result<FixedBytes<32>, Vec<u8>> {
        self.static_call_pool_manager(IPoolManager::extsloadCall { slot })
    }

    /// Reads one transient storage slot of the pool manager.
    fn exttload(&self, slot: FixedBytes<32>) -> Result<FixedBytes<32>, Vec<u8>> {
        self.static_call_pool_manager(IPoolManager::exttloadCall { slot })
    }
}

impl<T: HookConfig + HostAccess + TopLevelStorage + Sized> PoolManagerCalls for T {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The selectors v4-core dispatches on, from the compiled `IPoolManager.sol`
    /// (`forge inspect IPoolManager methodIdentifiers`).
    #[test]
    fn selectors_match_uniswap_ipoolmanager() {
        assert_eq!(IPoolManager::unlockCall::SELECTOR, [0x48, 0xc8, 0x94, 0x91]);
        assert_eq!(
            IPoolManager::initializeCall::SELECTOR,
            [0x62, 0x76, 0xcb, 0xbe]
        );
        assert_eq!(
            IPoolManager::modifyLiquidityCall::SELECTOR,
            [0x5a, 0x6b, 0xcf, 0xda]
        );
        assert_eq!(IPoolManager::swapCall::SELECTOR, [0xf3, 0xcd, 0x91, 0x4c]);
        assert_eq!(IPoolManager::donateCall::SELECTOR, [0x23, 0x42, 0x66, 0xd7]);
        assert_eq!(IPoolManager::syncCall::SELECTOR, [0xa5, 0x84, 0x11, 0x94]);
        assert_eq!(IPoolManager::takeCall::SELECTOR, [0x0b, 0x0d, 0x9c, 0x09]);
        assert_eq!(IPoolManager::settleCall::SELECTOR, [0x11, 0xda, 0x60, 0xb4]);
        assert_eq!(
            IPoolManager::settleForCall::SELECTOR,
            [0x3d, 0xd4, 0x5a, 0xdb]
        );
        assert_eq!(IPoolManager::clearCall::SELECTOR, [0x80, 0xf0, 0xb4, 0x4c]);
        assert_eq!(IPoolManager::mintCall::SELECTOR, [0x15, 0x6e, 0x29, 0xf6]);
        assert_eq!(IPoolManager::burnCall::SELECTOR, [0xf5, 0x29, 0x8a, 0xca]);
        assert_eq!(
            IPoolManager::extsloadCall::SELECTOR,
            [0x1e, 0x2e, 0xae, 0xaf]
        );
        assert_eq!(
            IPoolManager::exttloadCall::SELECTOR,
            [0xf1, 0x35, 0xba, 0xaa]
        );
    }
}
