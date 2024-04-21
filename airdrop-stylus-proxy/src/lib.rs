// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;
use stylus_sdk::{msg, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct AirdropHook {
        mapping(bytes32 => uint256) before_swap_count;
        mapping(bytes32 => uint256) after_swap_count;
        mapping(bytes32 => uint256) before_add_liquidity_count;
        mapping(bytes32 => uint256) before_remove_liquidity_count;
        mapping(bytes32 => mapping(address => SwapInfo)) total_swap_user;
        mapping(bytes32 => SwapInfo) total_swap;
        mapping(bytes32 => address[]) users;
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
        uint128 counter0;
        uint128 counter1;
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

    pub fn set_hook(&mut self, value: Address) -> Result<()> {
        let hook: Address = self.hook.get();

        if !hook.is_zero() {
            return Err(HookError::HookAlreadyDefined(HookAlreadyDefined {}));
        }

        self.hook.set(value);
        Ok(())
    }

    pub fn add_before_swap(&mut self, key: FixedBytes<32>) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }
        let value = self.before_swap_count.get(key) + U256::from(1);
        self.before_swap_count.replace(key, value);
        return Ok(());
    }

    pub fn add_after_swap(&mut self, key: FixedBytes<32>) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        let value = self.after_swap_count.get(key) + U256::from(1);
        self.after_swap_count.replace(key, value);
        return Ok(());
    }

    pub fn add_before_add_liquidity(&mut self, key: FixedBytes<32>) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        let value = self.before_add_liquidity_count.get(key) + U256::from(1);
        self.before_add_liquidity_count.replace(key, value);
        return Ok(());
    }

    pub fn add_before_remove_liquidity(&mut self, key: FixedBytes<32>) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        let value = self.before_remove_liquidity_count.get(key) + U256::from(1);
        self.before_remove_liquidity_count.replace(key, value);
        return Ok(());
    }
}
