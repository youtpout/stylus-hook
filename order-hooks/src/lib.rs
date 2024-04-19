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
    pub struct LimitOrder {
        uint256 epochNext;
        address hook;
        mapping(bytes32 => uint32) tickLowerLasts;
    }

    pub struct EpochInfo {
        bool filled;
        address currency0;
        address currency1;
        uint256 token0Total;
        uint256 token1Total;
        uint128 liquidityTotal;
        mapping(address => uint128) liquidity;
    }

}

sol! {
    error ZeroLiquidity();
    error InRange();
    error CrossedRange();
    error Filled();
    error NotFilled();
    error NotPoolManagerToken();
}

#[derive(SolidityError)]
pub enum HookError {
    ZeroLiquidity(ZeroLiquidity),
    InRange(InRange),
    CrossedRange(CrossedRange),
    Filled(Filled),
    NotFilled(NotFilled),
    NotPoolManagerToken(NotPoolManagerToken),
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl LimitOrder {
    pub fn tickLowerLasts(&self, pool_id: FixedBytes<32>) -> U256 {
        self.tickLowerLasts(pool_id)
    }
}
