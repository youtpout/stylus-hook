// SPDX-License-Identifier: MIT OR Apache-2.0
//! A Uniswap v4 hook written entirely in Rust — no Solidity anywhere.
//!
//! This is the same hook as `uniswap/src/Counter.sol`, but the contract the `PoolManager` calls
//! *is* the Stylus contract: there is no Solidity shell forwarding callbacks. It counts the four
//! callbacks it enables, per pool.
//!
//! The address still has to encode the permission flags, so this contract must be deployed through
//! a CREATE2 factory with a mined salt; [`Counter::validate`] checks the result.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::{
    abi::Bytes,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256},
};
use stylus_uniswap_v4::{
    hooks::{selector, HookConfig, HookGuards, IHooks},
    types::{BalanceDelta, BeforeSwapDelta, ModifyLiquidityParams, PoolKey, SwapParams, U24},
    Permissions, ZERO_DELTA,
};

#[storage]
#[entrypoint]
pub struct Counter {
    pool_manager: StorageAddress,
    before_swap_count: StorageMap<FixedBytes<32>, StorageU256>,
    after_swap_count: StorageMap<FixedBytes<32>, StorageU256>,
    before_add_liquidity_count: StorageMap<FixedBytes<32>, StorageU256>,
    before_remove_liquidity_count: StorageMap<FixedBytes<32>, StorageU256>,
}

impl HookConfig for Counter {
    fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    fn permissions(&self) -> Permissions {
        Permissions::none()
            .with_before_swap()
            .with_after_swap()
            .with_before_add_liquidity()
            .with_before_remove_liquidity()
    }
}

#[public]
#[implements(IHooks)]
impl Counter {
    /// Reverts unless the deployed address carries exactly the flags [`HookConfig::permissions`]
    /// declares, the same assertion the Solidity `BaseHook` makes in its constructor. Deploy this
    /// through a CREATE2 salt mined by `stylus/hook-miner`.
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        self.pool_manager.set(pool_manager);
        HookGuards::validate_hook_address(self)
    }

    pub fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    /// The low 14 bits this contract's address must carry for v4 to invoke the right callbacks.
    /// Mine a CREATE2 salt against this value.
    pub fn required_hook_flags(&self) -> U256 {
        U256::from(HookConfig::permissions(self).flags())
    }

    pub fn before_swap_count(&self, pool_id: FixedBytes<32>) -> U256 {
        self.before_swap_count.get(pool_id)
    }

    pub fn after_swap_count(&self, pool_id: FixedBytes<32>) -> U256 {
        self.after_swap_count.get(pool_id)
    }

    pub fn before_add_liquidity_count(&self, pool_id: FixedBytes<32>) -> U256 {
        self.before_add_liquidity_count.get(pool_id)
    }

    pub fn before_remove_liquidity_count(&self, pool_id: FixedBytes<32>) -> U256 {
        self.before_remove_liquidity_count.get(pool_id)
    }
}

#[public]
impl IHooks for Counter {
    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        Self::bump(&mut self.before_swap_count, key.to_id());
        Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
    }

    fn after_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _delta: BalanceDelta,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, i128), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        Self::bump(&mut self.after_swap_count, key.to_id());
        Ok((selector::AFTER_SWAP, 0))
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
        Self::bump(&mut self.before_add_liquidity_count, key.to_id());
        Ok(selector::BEFORE_ADD_LIQUIDITY)
    }

    fn before_remove_liquidity(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: ModifyLiquidityParams,
        _hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        Self::bump(&mut self.before_remove_liquidity_count, key.to_id());
        Ok(selector::BEFORE_REMOVE_LIQUIDITY)
    }
}

impl Counter {
    fn bump(counters: &mut StorageMap<FixedBytes<32>, StorageU256>, pool_id: FixedBytes<32>) {
        let mut slot = counters.setter(pool_id);
        let current = slot.get();
        slot.set(current + U256::from(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{I256, U256 as Uint};
    use stylus_sdk::testing::*;
    use stylus_uniswap_v4::types::{I24, U160};

    const POOL_MANAGER: Address = Address::new([0x4e; 20]);

    /// An address whose low 14 bits are 0x0ac0 — beforeSwap | afterSwap | beforeAddLiquidity |
    /// beforeRemoveLiquidity, exactly what this hook declares.
    const HOOK: Address = Address::new([
        0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe,
        0xef, 0x00, 0x00, 0x0a, 0xc0,
    ]);

    fn pool_key() -> PoolKey {
        PoolKey {
            currency0: Address::new([0xc0; 20]),
            currency1: Address::new([0xc1; 20]),
            fee: U24::from(3000u32),
            tickSpacing: I24::try_from(60i32).unwrap(),
            hooks: HOOK,
        }
    }

    fn swap_params() -> SwapParams {
        SwapParams {
            zeroForOne: true,
            amountSpecified: I256::try_from(-1_000_000i64).unwrap(),
            sqrtPriceLimitX96: U160::from(1u64),
        }
    }

    fn liquidity_params() -> ModifyLiquidityParams {
        ModifyLiquidityParams {
            tickLower: I24::try_from(-60i32).unwrap(),
            tickUpper: I24::try_from(60i32).unwrap(),
            liquidityDelta: I256::try_from(1_000i64).unwrap(),
            salt: FixedBytes::ZERO,
        }
    }

    fn deployed(vm: &TestVM) -> Counter {
        vm.set_contract_address(HOOK);
        let mut contract = Counter::from(vm);
        contract
            .constructor(POOL_MANAGER)
            .expect("HOOK carries the declared flags");
        vm.set_sender(POOL_MANAGER);
        contract
    }

    #[test]
    fn counts_the_callbacks_per_pool() {
        let vm = TestVM::default();
        let mut contract = deployed(&vm);
        let key = pool_key();
        let id = key.to_id();
        let empty: Bytes = Vec::new().into();

        assert_eq!(
            contract
                .before_swap(POOL_MANAGER, key.clone(), swap_params(), empty.clone())
                .unwrap()
                .0,
            selector::BEFORE_SWAP
        );
        contract
            .after_swap(
                POOL_MANAGER,
                key.clone(),
                swap_params(),
                I256::ZERO,
                empty.clone(),
            )
            .unwrap();
        contract
            .after_swap(
                POOL_MANAGER,
                key.clone(),
                swap_params(),
                I256::ZERO,
                empty.clone(),
            )
            .unwrap();
        contract
            .before_add_liquidity(POOL_MANAGER, key.clone(), liquidity_params(), empty.clone())
            .unwrap();

        assert_eq!(contract.before_swap_count(id), Uint::from(1));
        assert_eq!(contract.after_swap_count(id), Uint::from(2));
        assert_eq!(contract.before_add_liquidity_count(id), Uint::from(1));
        assert_eq!(contract.before_remove_liquidity_count(id), Uint::ZERO);

        // a different pool has its own counters
        let mut other = pool_key();
        other.fee = U24::from(500u32);
        assert_eq!(contract.after_swap_count(other.to_id()), Uint::ZERO);
    }

    #[test]
    fn rejects_callers_other_than_the_pool_manager() {
        let vm = TestVM::default();
        let mut contract = deployed(&vm);
        vm.set_sender(Address::new([0x99; 20]));

        let key = pool_key();
        assert!(contract
            .after_swap(
                POOL_MANAGER,
                key.clone(),
                swap_params(),
                I256::ZERO,
                Vec::new().into()
            )
            .is_err());
        assert_eq!(contract.after_swap_count(key.to_id()), Uint::ZERO);
    }

    #[test]
    fn unimplemented_callbacks_revert() {
        let vm = TestVM::default();
        let mut contract = deployed(&vm);

        // `beforeDonate` is not in the declared permissions, so it keeps the base implementation
        assert!(contract
            .before_donate(
                POOL_MANAGER,
                pool_key(),
                Uint::ZERO,
                Uint::ZERO,
                Vec::new().into()
            )
            .is_err());
    }

    #[test]
    fn declares_the_flags_its_address_must_carry() {
        let vm = TestVM::default();
        let contract = deployed(&vm);
        // beforeSwap | afterSwap | beforeAddLiquidity | beforeRemoveLiquidity
        assert_eq!(contract.required_hook_flags(), Uint::from(0x0ac0));
    }

    #[test]
    fn refuses_to_deploy_at_an_address_without_the_flags() {
        let vm = TestVM::default();
        vm.set_contract_address(Address::new([0x11; 20]));
        let mut contract = Counter::from(&vm);
        assert!(
            contract.constructor(POOL_MANAGER).is_err(),
            "a hook at an address v4 will never call is not a hook"
        );
    }

    #[test]
    fn rejects_a_pool_that_names_another_hook() {
        let vm = TestVM::default();
        let mut contract = deployed(&vm);

        let mut foreign = pool_key();
        foreign.hooks = Address::new([0x77; 20]);

        assert!(contract
            .after_swap(
                POOL_MANAGER,
                foreign.clone(),
                swap_params(),
                I256::ZERO,
                Vec::new().into()
            )
            .is_err());
        assert_eq!(contract.after_swap_count(foreign.to_id()), Uint::ZERO);
    }
}
