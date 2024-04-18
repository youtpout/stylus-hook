
// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::{string::String, vec, vec::Vec};
use alloy_primitives::{Address, U256, FixedBytes};
use alloy_sol_types::{sol};
use core::{borrow::BorrowMut, marker::PhantomData};
use stylus_sdk::{
    abi::Bytes,
    evm,
    msg,
    prelude::*,
    deploy::*
};
use std::convert::TryInto;

mod hook;

use hook::Counter;

/// Represents the ways methods may fail.
pub enum DeployError {
    ExternalCall(stylus_sdk::call::Error),
}

impl From<stylus_sdk::call::Error> for DeployError {
    fn from(err: stylus_sdk::call::Error) -> Self {
        Self::ExternalCall(err)
    }
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<DeployError> for Vec<u8> {
    fn from(val: DeployError) -> Self {
        match val {
            DeployError::ExternalCall(err) => err.into(),
        }
    }
}


/// Simplifies the result type for the contract's methods.
type Result<T, E = DeployError> = core::result::Result<T, E>;


// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct DeployContract {
       
    }
}

sol! {
    event Deploy(address indexed contract);
}

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl DeployContract {
    /// Gets the number from storage.
    pub fn deploy_deterministic(&self) -> Result<Address,Vec<u8>> {
        let raw : RawDeploy = RawDeploy::new();
        let input = "0x0123456789012345678901234567890123456789012345678901234567890123";
        let decoded = hex::decode(input).expect("Decoding failed");
        let fix = decoded.try_into().unwrap();
        let salt = FixedBytes::<32>::new(fix);
        raw.salt(salt);
        //let code = Counter.self;
        let result = raw.deploy([1001]);
        let newAddress = result.unwrap();
        evm::log(Deploy { contract :  newAddress });

        result
    }

    

}
