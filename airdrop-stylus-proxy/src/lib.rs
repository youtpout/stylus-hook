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

    pub fn add_after_swap(
        &mut self,
        pool_id: FixedBytes<32>,
        sender: Address,
        zero_for_one: bool,
        amount_specified: U256,
    ) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        // if the token for airdrop if define, aidrop calculation is finished
        if !self.airdrop_token.get(pool_id).is_zero() {
            return Ok(());
        }

        let exist: bool = self.user_exist.get(pool_id).get(sender);
        if !exist {
            self.user_exist.setter(pool_id).insert(sender, true);
            let mut count = self.users_count.setter(pool_id);
            let oldCount = count.get();
            count.set(oldCount + U256::from(1));
        }

        let mut swap_pool = self.total_swap_user.setter(pool_id);
        let mut swap_user = swap_pool.setter(sender);
        let mut swap_total = self.total_swap.setter(pool_id);
        if zero_for_one {
            let amount_total1: U256 = swap_total.amount1.get();
            let counter_total1: U256 = swap_total.counter1.get();
            let amount_user1: U256 = swap_user.amount1.get();
            let counter_user1: U256 = swap_user.counter1.get();
            swap_user.amount1.set(amount_specified + amount_user1);
            swap_total.amount1.set(amount_specified + amount_total1);
            swap_user.counter1.set(counter_user1 + U256::from(1));
            swap_total.counter1.set(counter_total1 + U256::from(1));
        } else {
            let amount_total0: U256 = swap_total.amount0.get();
            let counter_total0: U256 = swap_total.counter0.get();
            let amount_user0: U256 = swap_user.amount0.get();
            let counter_user0: U256 = swap_user.counter0.get();
            swap_user.amount0.set(amount_specified + amount_user0);
            swap_total.amount0.set(amount_specified + amount_total0);
            swap_user.counter0.set(counter_user0 + U256::from(1));
            swap_total.counter0.set(counter_total0 + U256::from(1));
        }

        //  let value = self.after_swap_count.get(key) + U256::from(1);
        //  self.after_swap_count.replace(key, value);
        return Ok(());
    }
}
