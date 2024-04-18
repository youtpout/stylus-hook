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
        mapping(bytes32 => uint256) before_swap_count;
        mapping(bytes32 => uint256) after_swap_count;
        mapping(bytes32 => uint256) before_add_liquidity_count;
        mapping(bytes32 => uint256) before_remove_liquidity_count;
    }

    
}


/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl Counter {

    /// Gets the number from storage.
    pub fn before_swap_count(&self,  pool_id :FixedBytes<32>) -> U256 {
        self.before_swap_count.get(pool_id)
    }

    pub fn after_swap_count(&self, pool_id :FixedBytes<32>) -> U256 {
        self.after_swap_count.get(pool_id)
    }


    pub fn before_add_liquidity_count(&self, pool_id :FixedBytes<32>) -> U256 {
        self.before_add_liquidity_count.get(pool_id)
    }


    pub fn before_remove_liquidity_count(&self, pool_id :FixedBytes<32>) -> U256 {
        self.before_remove_liquidity_count.get(pool_id)
    }

    pub fn get_hook_permissions(&self) -> (bool,bool,bool,bool,bool,bool,bool,bool,bool,bool) {
        // doesn't support struct in return so use tuple
       let permissions =  (
         false,
         false,
         true,
         false,
         true,
         false,
         true,
         true,
         false,
         false
       );

        permissions
    }
}
