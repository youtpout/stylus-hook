
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, alloy_primitives::Address, prelude::*};

mod Counter;

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Deploy {
        uint256 toto;
    }
}

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl Deploy {
    /// Gets the number from storage.
    pub fn deploy(&self) -> Record<Address> {
        let raw : RawDeploy = RawDeploy::new();
        raw.salt(3);
        raw.deploy(Counter);
    }

}
