// SPDX-License-Identifier: MIT OR Apache-2.0
//! Per-pool hook-callback counters for a Uniswap v4 hook, running on Arbitrum Stylus.
//!
//! `CounterProxy.sol` is the Solidity shell whose address carries the v4 permission flags; it
//! forwards every callback here. `Counter.sol` is the equivalent pure-Solidity hook, so the two
//! can be gas-compared. The ABI is mirrored in `uniswap/src/ICounter.sol`.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;
use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256},
};

#[storage]
#[entrypoint]
pub struct Counter {
    before_swap_count: StorageMap<FixedBytes<32>, StorageU256>,
    after_swap_count: StorageMap<FixedBytes<32>, StorageU256>,
    before_add_liquidity_count: StorageMap<FixedBytes<32>, StorageU256>,
    before_remove_liquidity_count: StorageMap<FixedBytes<32>, StorageU256>,
    hook: StorageAddress,
}

sol! {
    #[derive(Debug)]
    error NotHook();
    #[derive(Debug)]
    error HookAlreadyDefined();
}

#[derive(SolidityError, Debug)]
pub enum CounterError {
    NotHook(NotHook),
    HookAlreadyDefined(HookAlreadyDefined),
}

type Result<T, E = CounterError> = core::result::Result<T, E>;

#[public]
impl Counter {
    /// The v4 hook contract allowed to write into this contract.
    pub fn hook(&self) -> Address {
        self.hook.get()
    }

    /// One-shot binding of the hook contract. Reverts once set.
    pub fn set_hook(&mut self, value: Address) -> Result<()> {
        if !self.hook.get().is_zero() {
            return Err(HookAlreadyDefined {}.into());
        }
        self.hook.set(value);
        Ok(())
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

    pub fn add_before_swap(&mut self, pool_id: FixedBytes<32>) -> Result<()> {
        self.only_hook()?;
        Self::bump(&mut self.before_swap_count, pool_id);
        Ok(())
    }

    pub fn add_after_swap(&mut self, pool_id: FixedBytes<32>) -> Result<()> {
        self.only_hook()?;
        Self::bump(&mut self.after_swap_count, pool_id);
        Ok(())
    }

    pub fn add_before_add_liquidity(&mut self, pool_id: FixedBytes<32>) -> Result<()> {
        self.only_hook()?;
        Self::bump(&mut self.before_add_liquidity_count, pool_id);
        Ok(())
    }

    pub fn add_before_remove_liquidity(&mut self, pool_id: FixedBytes<32>) -> Result<()> {
        self.only_hook()?;
        Self::bump(&mut self.before_remove_liquidity_count, pool_id);
        Ok(())
    }
}

impl Counter {
    fn only_hook(&self) -> Result<()> {
        if self.vm().msg_sender() != self.hook.get() {
            return Err(NotHook {}.into());
        }
        Ok(())
    }

    fn bump(counters: &mut StorageMap<FixedBytes<32>, StorageU256>, pool_id: FixedBytes<32>) {
        let mut slot = counters.setter(pool_id);
        let current = slot.get();
        slot.set(current + U256::from(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    fn pool_id(byte: u8) -> FixedBytes<32> {
        FixedBytes::from([byte; 32])
    }

    #[test]
    fn counts_per_pool() {
        let vm = TestVM::default();
        let mut contract = Counter::from(&vm);

        let hook = Address::from([0x11; 20]);
        contract.set_hook(hook).unwrap();
        vm.set_sender(hook);

        let pool_a = pool_id(1);
        let pool_b = pool_id(2);

        contract.add_before_swap(pool_a).unwrap();
        contract.add_after_swap(pool_a).unwrap();
        contract.add_after_swap(pool_a).unwrap();
        contract.add_before_add_liquidity(pool_a).unwrap();
        contract.add_before_remove_liquidity(pool_b).unwrap();

        assert_eq!(contract.before_swap_count(pool_a), U256::from(1));
        assert_eq!(contract.after_swap_count(pool_a), U256::from(2));
        assert_eq!(contract.before_add_liquidity_count(pool_a), U256::from(1));
        assert_eq!(contract.before_remove_liquidity_count(pool_a), U256::ZERO);

        // counters are namespaced per pool
        assert_eq!(contract.after_swap_count(pool_b), U256::ZERO);
        assert_eq!(
            contract.before_remove_liquidity_count(pool_b),
            U256::from(1)
        );
    }

    #[test]
    fn hook_binds_once() {
        let vm = TestVM::default();
        let mut contract = Counter::from(&vm);

        contract.set_hook(Address::from([0x11; 20])).unwrap();
        assert!(matches!(
            contract.set_hook(Address::from([0x22; 20])),
            Err(CounterError::HookAlreadyDefined(_))
        ));
    }

    #[test]
    fn only_the_hook_can_write() {
        let vm = TestVM::default();
        let mut contract = Counter::from(&vm);

        contract.set_hook(Address::from([0x11; 20])).unwrap();
        vm.set_sender(Address::from([0x99; 20]));

        assert!(matches!(
            contract.add_after_swap(pool_id(1)),
            Err(CounterError::NotHook(_))
        ));
        assert_eq!(contract.after_swap_count(pool_id(1)), U256::ZERO);
    }
}
