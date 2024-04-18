// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use std::char::from_digit;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, Uint, I256, U256, Signed};
use alloy_sol_types::sol_data::Int;
use hex::FromHex;
use stylus_sdk::prelude::*;
use ux::*;

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Counter {
        mapping(bytes32 => uint256) before_swap_count;
        mapping(bytes32 => uint256) after_swap_count;
        mapping(bytes32 => uint256) before_add_liquidity_count;
        mapping(bytes32 => uint256) before_remove_liquidity_count;
    }
}

// copy from IHooks and BaseHooks
sol! {}

pub type U24 = Uint<24, 1>;
pub type I24 = Signed<24, 1>;

type PoolKey = (Address, Address, U24, U24, Address);
type ModifyLiquidityParams = (I24, I24, I256);

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl Counter {
    /// Gets the number from storage.
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

    pub fn after_swap(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        delta: I256,
        hookData: Vec<u8>,
    ) -> Result<FixedBytes<4>, Vec<u8>> {
        let value = self.after_swap_count.get(key) + U256::from(1);
        self.after_swap_count.replace(key, value);
        let sign = FixedBytes::<4>::from_hex("b47b2fb1");

        return Ok(sign);
    }

    pub fn get_hook_permissions(
        &self,
    ) -> (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) {
        // doesn't support struct in return so use tuple
        let permissions = (
            false, false, true, false, true, false, true, true, false, false,
        );

        permissions
    }
}
