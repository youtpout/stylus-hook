//! Example on how to interact with a deployed `stylus-hello-world` program using defaults.
//! This example uses ethers-rs to instantiate the program using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed program is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use alloy_primitives::bytes;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
    types::Bytes,
};
use eyre::eyre;
use std::convert::TryInto;
use std::fs;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;

/// Your private key file path.
const PRIV_KEY: &str = "0x0123456789012345678901234567890123456789012345678901234567890123";

/// Stylus RPC endpoint url.
const RPC_URL: &str = "http://localhost:8547/";

/// Deployed pragram address.
const STYLUS_PROGRAM_ADDRESS: &str = "0x8cDE56336E289c028C8f7CF5c20283fF02272182";

const HOOK_ADDRESS: &str = "0x0105aC59AdaBE5CD26ce684e309C1aa5D9D93874";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let priv_key_path = PRIV_KEY;
    let rpc_url = RPC_URL;
    let program_address = STYLUS_PROGRAM_ADDRESS;
    abigen!(
        Counter,
        r#"[           
            function setHook(address value) external     
            error NotHook()       
            error HookAlreadyDefined()
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = program_address.parse()?;

    //let privkey = read_secret_from_file(&priv_key_path)?;
    let wallet = LocalWallet::from_str(&priv_key_path)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    let counter = Counter::new(address, client);

    let addr: Address = HOOK_ADDRESS.parse()?;
    let deploy = counter.set_hook(addr).send().await?.await?;
    println!("set hook");
    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
