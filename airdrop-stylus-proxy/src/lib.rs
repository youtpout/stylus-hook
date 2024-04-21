// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, U128, U256};
use alloy_sol_types::sol;
use stylus_sdk::{msg, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct AirdropHook {
        mapping(bytes32 => mapping(address => SwapInfo)) total_swap_user;
        mapping(bytes32 => SwapInfo) total_swap;
        mapping(bytes32 => uint256) users_count;
        mapping(bytes32 => mapping(address => bool)) user_exist;
        mapping(bytes32 => address) airdrop_token;
        mapping(bytes32 => mapping(address => bool)) claimed;
        address hook;
    }

    // Airdrop is caculated between total amount swaped by user and number of swap did
    pub struct SwapInfo{
        // amount swapped is profitable for liquidity provider
        uint256 amount0;
        uint256 amount1;
        // number swapped is profitable for network miner
        uint256 counter0;
        uint256 counter1;
    }

}

sol! {
    error NotHook();
    error HookAlreadyDefined();
    error AirdropNotEnd();
    error AlreadyClaimed();
}

#[derive(SolidityError)]
pub enum HookError {
    NotHook(NotHook),
    HookAlreadyDefined(HookAlreadyDefined),
    AirdropNotEnd(AirdropNotEnd),
    AlreadyClaimed(AlreadyClaimed),
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl AirdropHook {
    /// Gets the number from storage.
    // pub fn get_total_swap_user(
    //     &self,
    //     pool_id: FixedBytes<32>,
    //     user: address,
    // ) -> (U256, U256, U256, U256) {
    //     self.total_swap_user.get(pool_id, user)
    // }

    pub fn get_total_swap(&self, pool_id: FixedBytes<32>) -> (U256, U256, U256, U256) {
        let data = self.total_swap.get(pool_id);
        (
            data.amount0.get(),
            data.amount1.get(),
            data.counter0.get(),
            data.counter1.get(),
        )
    }

    // pub fn get_users(&self, pool_id: FixedBytes<32>) -> Vec<Address> {
    //     self.users.get(pool_id)
    // }

    // pub fn get_airdrop_token(&self, pool_id: FixedBytes<32>) -> Address {
    //     self.airdrop_token.get(pool_id)
    // }

    pub fn set_hook(&mut self, value: Address) -> Result<()> {
        let hook: Address = self.hook.get();

        if !hook.is_zero() {
            return Err(HookError::HookAlreadyDefined(HookAlreadyDefined {}));
        }

        self.hook.set(value);
        Ok(())
    }

    pub fn add_after_swap(&mut self, key: FixedBytes<32>) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        //  let value = self.after_swap_count.get(key) + U256::from(1);
        //  self.after_swap_count.replace(key, value);
        return Ok(());
    }
}
