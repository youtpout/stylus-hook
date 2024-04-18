// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use alloy_primitives::{Address, uint, U256, FixedBytes};
use alloy_sol_types::{sol};
/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::{
    abi::Bytes, call::Call, contract, msg, prelude::*, storage::StorageAddress
};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Counter {
        mapping(bytes32 => uint256) beforeSwapCount;
        mapping(bytes32 => uint256) afterSwapCount;
        mapping(bytes32 => uint256) beforeAddLiquidityCount;
        mapping(bytes32 => uint256) beforeRemoveLiquidityCount;
    }

    
}

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl Counter {
  /*  pub struct Permissions {
        bool beforeInitialize;
        bool afterInitialize;
        bool beforeAddLiquidity;
        bool afterAddLiquidity;
        bool beforeRemoveLiquidity;
        bool afterRemoveLiquidity;
        bool beforeSwap;
        bool afterSwap;
        bool beforeDonate;
       
        bool afterDonate;
    }*/

    /// Gets the number from storage.
    pub fn beforeSwapCount(&self,  pool_id :FixedBytes<32>) -> U256 {
        self.beforeSwapCount.get(pool_id)
    }

    pub fn afterSwapCount(&self, pool_id :FixedBytes<32>) -> U256 {
        self.afterSwapCount.get(pool_id)
    }


    pub fn beforeAddLiquidityCount(&self, pool_id :FixedBytes<32>) -> U256 {
        self.beforeAddLiquidityCount.get(pool_id)
    }


    pub fn beforeRemoveLiquidityCount(&self, pool_id :FixedBytes<32>) -> U256 {
        self.beforeRemoveLiquidityCount.get(pool_id)
    }

  /*  pub fn getHookPermissions(&self) -> bool {
       let permissions =  Permissions {
        beforeInitialize: false,
        afterInitialize: false,
        beforeAddLiquidity: true,
        afterAddLiquidity: false,
        beforeRemoveLiquidity: true,
        afterRemoveLiquidity: false,
        beforeSwap: true,
        afterSwap: true,
        beforeDonate: false,
        afterDonate: false
    };

    permissions;
    }*/
}
