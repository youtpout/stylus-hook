// SPDX-License-Identifier: MIT OR Apache-2.0
//! A Uniswap v4 hook, in pure Rust, that does nothing but arithmetic — with a dial for how much.
//!
//! Stylus buys a lower marginal cost of computation at the price of a fixed cost per call: loading
//! the WASM program, and crossing from the EVM into the WASM runtime. Below some amount of work the
//! fixed cost wins and the same hook is cheaper in Solidity. This hook and its Solidity twin,
//! `uniswap/src/ComputeHook.sol`, run the identical loop so `bench-compute.bash` can sweep the dial
//! and find where the lines cross.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::U64, Address, FixedBytes, U256};
use stylus_sdk::{
    abi::Bytes,
    prelude::*,
    storage::{StorageAddress, StorageU256, StorageU64},
};
use stylus_uniswap_v4::{
    hooks::{selector, HookConfig, HookGuards, IHooks},
    types::{BeforeSwapDelta, PoolKey, SwapParams, U24},
    Permissions, ZERO_DELTA,
};

#[storage]
#[entrypoint]
pub struct ComputeHook {
    pool_manager: StorageAddress,
    rounds: StorageU256,
    last_result: StorageU64,
}

impl HookConfig for ComputeHook {
    fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    fn permissions(&self) -> Permissions {
        Permissions::none().with_before_swap()
    }
}

#[public]
#[implements(IHooks)]
impl ComputeHook {
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        self.pool_manager.set(pool_manager);
        HookGuards::validate_hook_address(self)
    }

    pub fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    pub fn rounds(&self) -> U256 {
        self.rounds.get()
    }

    pub fn set_rounds(&mut self, new_rounds: U256) {
        self.rounds.set(new_rounds);
    }

    pub fn last_result(&self) -> u64 {
        self.last_result.get().to::<u64>()
    }

    /// xorshift64, `n` times. Must agree with `ComputeHook.sol`.
    ///
    /// A `u64` is a native WASM word. The EVM has no such thing: Solidity's `uint64` is a 256-bit
    /// word masked back down after every shift, which is the asymmetry this benchmark is about.
    pub fn work(&self, n: U256) -> u64 {
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut i = U256::ZERO;
        while i < n {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            i += U256::from(1);
        }
        x
    }
}

#[public]
impl IHooks for ComputeHook {
    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        let result = self.work(self.rounds.get());
        self.last_result.set(U64::from(result));
        Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    /// The flags this hook declares: beforeSwap only.
    const HOOK: Address = Address::new([
        0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe,
        0xef, 0x00, 0x00, 0x00, 0x80,
    ]);

    #[test]
    fn xorshift_matches_the_solidity_twin() {
        let vm = TestVM::default();
        vm.set_contract_address(HOOK);
        let mut contract = ComputeHook::from(&vm);
        contract.constructor(Address::new([0x4e; 20])).unwrap();

        // zero rounds leaves the seed untouched
        assert_eq!(contract.work(U256::ZERO), 0x9E37_79B9_7F4A_7C15);
        // reference values, asserted against ComputeHook.sol in uniswap/test/ComputeHook.t.sol
        assert_eq!(contract.work(U256::from(1)), 0xdc1b_77ae_0bf3_4dad);
        assert_eq!(contract.work(U256::from(10)), 0x8f8e_a9d3_4942_8d8e);
        assert_eq!(contract.work(U256::from(100)), 0xab59_17a8_1f0f_b2ae);
    }

    #[test]
    fn only_declares_before_swap() {
        let vm = TestVM::default();
        vm.set_contract_address(HOOK);
        let mut contract = ComputeHook::from(&vm);
        contract.constructor(Address::new([0x4e; 20])).unwrap();
        assert_eq!(
            HookConfig::permissions(&contract).flags(),
            stylus_uniswap_v4::permissions::BEFORE_SWAP_FLAG
        );
    }
}
