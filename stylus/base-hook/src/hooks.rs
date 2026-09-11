// SPDX-License-Identifier: MIT OR Apache-2.0
//! The `IHooks` entry points, as a Stylus trait with `BaseHook.sol` semantics.
//!
//! A hook implements [`HookConfig`] to declare its pool manager and permissions, then implements
//! only the callbacks it enables — every other callback keeps its default body, which reverts with
//! `HookNotImplemented`, exactly as the Solidity `BaseHook` does.
//!
//! # Guards
//!
//! Solidity's `BaseHook` makes its entry points `external onlyPoolManager`, so a hook author cannot
//! forget the check. Rust has no abstract types to inherit that from, so
//! [`guarded_hooks`](stylus_uniswap_v4_macros::guarded_hooks) inserts it instead — put it above
//! `#[public]` on the `impl IHooks` block and every callback gains
//! [`HookGuards::require_pool_manager`], plus [`HookGuards::require_valid_pool`] where it takes a
//! `PoolKey`.
//!
//! Only on that block, though. A hook's own entry points — an order book, a claim — must *not*
//! require the pool manager.

use alloc::vec::Vec;

use alloy_primitives::{
    aliases::{I24, U160, U24},
    Address, FixedBytes, I256, U256,
};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{abi::Bytes, prelude::*};

use crate::permissions::{is_valid_hook_address, Permissions};
use crate::types::{
    BalanceDelta, BeforeSwapDelta, ModifyLiquidityParams, PoolKey, SwapParams, ZERO_DELTA,
};

sol! {
    /// The callback was invoked but the hook does not implement it.
    #[derive(Debug)]
    error HookNotImplemented();
    /// The caller is not the pool manager.
    #[derive(Debug)]
    error NotPoolManager();
    /// The deployed address does not encode the declared permissions.
    #[derive(Debug)]
    error HookAddressNotValid(address hooks);
    /// The pool is not configured to use this hook.
    #[derive(Debug)]
    error InvalidPool();
}

/// The `IHooks` selectors a callback must echo back, as v4-core checks them.
pub mod selector {
    use alloy_primitives::FixedBytes;

    macro_rules! selector {
        ($name:ident, $bytes:expr) => {
            pub const $name: FixedBytes<4> = FixedBytes($bytes);
        };
    }

    selector!(BEFORE_INITIALIZE, [0xdc, 0x98, 0x35, 0x4e]);
    selector!(AFTER_INITIALIZE, [0x6f, 0xe7, 0xe6, 0xeb]);
    selector!(BEFORE_ADD_LIQUIDITY, [0x25, 0x99, 0x82, 0xe5]);
    selector!(AFTER_ADD_LIQUIDITY, [0x9f, 0x06, 0x3e, 0xfc]);
    selector!(BEFORE_REMOVE_LIQUIDITY, [0x21, 0xd0, 0xee, 0x70]);
    selector!(AFTER_REMOVE_LIQUIDITY, [0x6c, 0x2b, 0xbe, 0x7e]);
    selector!(BEFORE_SWAP, [0x57, 0x5e, 0x24, 0xb4]);
    selector!(AFTER_SWAP, [0xb4, 0x7b, 0x2f, 0xb1]);
    selector!(BEFORE_DONATE, [0xb6, 0xa8, 0xb0, 0xfa]);
    selector!(AFTER_DONATE, [0xe1, 0xb4, 0xaf, 0x69]);
}

/// What every hook must declare: who may call it, and which callbacks it implements.
///
/// Both methods take `&self` so that [`IHooks`] stays dyn-compatible, which `#[implements]`
/// requires.
pub trait HookConfig {
    /// The v4 `PoolManager` singleton — the only address allowed to invoke the callbacks.
    fn pool_manager(&self) -> Address;

    /// The callbacks this hook implements. Must agree with the low 14 bits of its address.
    fn permissions(&self) -> Permissions;
}

/// Guards every hook needs, available on any [`HookConfig`] that is a Stylus contract.
///
/// Deliberately *not* a supertrait of [`IHooks`]: it borrows the host, which would drag
/// `HostAccess`'s associated type into the `IHooks` bound and break `#[implements]`. Import it
/// where you implement the callbacks.
pub trait HookGuards: HookConfig + HostAccess {
    /// Reverts unless the caller is the pool manager. Every callback does this first.
    fn require_pool_manager(&self) -> Result<(), Vec<u8>> {
        if self.vm().msg_sender() != self.pool_manager() {
            return Err(NotPoolManager {}.abi_encode());
        }
        Ok(())
    }

    /// Reverts unless this contract's own address encodes exactly [`HookConfig::permissions`].
    ///
    /// The Solidity `BaseHook` asserts this in its constructor and so should a Stylus hook: call it
    /// from `#[constructor]`. A hook at an address that does not carry its flags is not a hook —
    /// v4 will simply never invoke the callbacks it thinks it implements.
    ///
    /// This is also what makes a hook undeployable by a plain `cargo stylus deploy`: the address
    /// has to be mined first. See `stylus/hook-miner`.
    fn validate_hook_address(&self) -> Result<(), Vec<u8>> {
        let address = self.vm().contract_address();
        if !is_valid_hook_address(address, &self.permissions()) {
            return Err(HookAddressNotValid { hooks: address }.abi_encode());
        }
        Ok(())
    }

    /// Reverts unless `key` names this contract as its hook.
    ///
    /// The pool manager passes whatever `PoolKey` the caller supplied, so a callback that trusts it
    /// blindly can be driven with a key belonging to some other pool. `BaseHook`'s
    /// `onlyValidPools` modifier in the OpenZeppelin hooks library does the same check.
    fn require_valid_pool(&self, key: &PoolKey) -> Result<(), Vec<u8>> {
        if key.hooks != self.vm().contract_address() {
            return Err(InvalidPool {}.abi_encode());
        }
        Ok(())
    }
}

impl<T: HookConfig + HostAccess> HookGuards for T {}

/// The ten v4 callbacks, ABI-identical to `IHooks.sol`.
///
/// Defaults revert with `HookNotImplemented`; override only what the permissions enable.
#[public]
pub trait IHooks: HookConfig {
    fn before_initialize(
        &mut self,
        sender: Address,
        key: PoolKey,
        sqrt_price_x96: U160,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let _ = (sender, key, sqrt_price_x96);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn after_initialize(
        &mut self,
        sender: Address,
        key: PoolKey,
        sqrt_price_x96: U160,
        tick: I24,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let _ = (sender, key, sqrt_price_x96, tick);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn before_add_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let _ = (sender, key, params, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn after_add_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        delta: BalanceDelta,
        fees_accrued: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BalanceDelta), Vec<u8>> {
        let _ = (sender, key, params, delta, fees_accrued, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn before_remove_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let _ = (sender, key, params, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn after_remove_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        delta: BalanceDelta,
        fees_accrued: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BalanceDelta), Vec<u8>> {
        let _ = (sender, key, params, delta, fees_accrued, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn before_swap(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: SwapParams,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        let _ = (sender, key, params, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn after_swap(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: SwapParams,
        delta: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, i128), Vec<u8>> {
        let _ = (sender, key, params, delta, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn before_donate(
        &mut self,
        sender: Address,
        key: PoolKey,
        amount0: U256,
        amount1: U256,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let _ = (sender, key, amount0, amount1, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }

    fn after_donate(
        &mut self,
        sender: Address,
        key: PoolKey,
        amount0: U256,
        amount1: U256,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let _ = (sender, key, amount0, amount1, hook_data);
        Err(HookNotImplemented {}.abi_encode())
    }
}

/// The zero deltas a callback returns when it does not take a share of the swap.
pub const NO_DELTA: I256 = ZERO_DELTA;

/// The callback the pool manager makes on whoever called [`unlock`].
///
/// Only a hook that unlocks the manager itself needs this — inside a swap or liquidity callback the
/// manager is already unlocked. Implement it alongside [`IHooks`] and list it in `#[implements]`.
///
/// [`unlock`]: crate::pool_manager::PoolManagerCalls::unlock
#[public]
pub trait IUnlockCallback: HookConfig {
    fn unlock_callback(&mut self, data: Bytes) -> Result<Bytes, Vec<u8>> {
        let _ = data;
        Err(HookNotImplemented {}.abi_encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::function_selector;

    /// The selectors v4-core dispatches on, taken from the compiled `IHooks.sol`
    /// (`forge inspect IHooks methodIdentifiers`).
    ///
    /// This is the load-bearing property of the whole crate: a hook written in Rust is only a hook
    /// if the `PoolManager`'s calls land on the right methods, so the Rust ABI must hash to exactly
    /// the same four bytes as the Solidity interface.
    #[test]
    fn selectors_match_uniswap_ihooks() {
        assert_eq!(
            function_selector!("beforeInitialize", Address, PoolKey, U160),
            [0xdc, 0x98, 0x35, 0x4e]
        );
        assert_eq!(
            function_selector!("afterInitialize", Address, PoolKey, U160, I24),
            [0x6f, 0xe7, 0xe6, 0xeb]
        );
        assert_eq!(
            function_selector!(
                "beforeAddLiquidity",
                Address,
                PoolKey,
                ModifyLiquidityParams,
                Bytes
            ),
            [0x25, 0x99, 0x82, 0xe5]
        );
        assert_eq!(
            function_selector!(
                "afterAddLiquidity",
                Address,
                PoolKey,
                ModifyLiquidityParams,
                I256,
                I256,
                Bytes
            ),
            [0x9f, 0x06, 0x3e, 0xfc]
        );
        assert_eq!(
            function_selector!(
                "beforeRemoveLiquidity",
                Address,
                PoolKey,
                ModifyLiquidityParams,
                Bytes
            ),
            [0x21, 0xd0, 0xee, 0x70]
        );
        assert_eq!(
            function_selector!(
                "afterRemoveLiquidity",
                Address,
                PoolKey,
                ModifyLiquidityParams,
                I256,
                I256,
                Bytes
            ),
            [0x6c, 0x2b, 0xbe, 0x7e]
        );
        assert_eq!(
            function_selector!("beforeSwap", Address, PoolKey, SwapParams, Bytes),
            [0x57, 0x5e, 0x24, 0xb4]
        );
        assert_eq!(
            function_selector!("afterSwap", Address, PoolKey, SwapParams, I256, Bytes),
            [0xb4, 0x7b, 0x2f, 0xb1]
        );
        assert_eq!(
            function_selector!("beforeDonate", Address, PoolKey, U256, U256, Bytes),
            [0xb6, 0xa8, 0xb0, 0xfa]
        );
        assert_eq!(
            function_selector!("afterDonate", Address, PoolKey, U256, U256, Bytes),
            [0xe1, 0xb4, 0xaf, 0x69]
        );
    }

    /// The constants a callback echoes back are the selectors of the callbacks themselves.
    #[test]
    fn returned_selectors_match_the_callbacks() {
        assert_eq!(
            selector::BEFORE_SWAP.0,
            function_selector!("beforeSwap", Address, PoolKey, SwapParams, Bytes)
        );
        assert_eq!(
            selector::AFTER_SWAP.0,
            function_selector!("afterSwap", Address, PoolKey, SwapParams, I256, Bytes)
        );
        assert_eq!(
            selector::BEFORE_ADD_LIQUIDITY.0,
            function_selector!(
                "beforeAddLiquidity",
                Address,
                PoolKey,
                ModifyLiquidityParams,
                Bytes
            )
        );
        assert_eq!(
            selector::BEFORE_REMOVE_LIQUIDITY.0,
            function_selector!(
                "beforeRemoveLiquidity",
                Address,
                PoolKey,
                ModifyLiquidityParams,
                Bytes
            )
        );
    }

    /// `PoolKey` must ABI-encode as `(address,address,uint24,int24,address)`.
    #[test]
    fn pool_key_encodes_as_the_solidity_tuple() {
        use stylus_sdk::abi::AbiType;
        assert_eq!(
            PoolKey::SELECTOR_ABI.as_str(),
            "(address,address,uint24,int24,address)"
        );
        assert_eq!(
            ModifyLiquidityParams::SELECTOR_ABI.as_str(),
            "(int24,int24,int256,bytes32)"
        );
        assert_eq!(SwapParams::SELECTOR_ABI.as_str(), "(bool,int256,uint160)");
    }
}
