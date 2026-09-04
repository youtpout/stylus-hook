// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mines the CREATE2 salt that puts a Stylus contract on a Uniswap v4 hook address.
//!
//! ```text
//! cargo stylus get-initcode --contract stylus-native-counter > initcode.hex
//! cargo run -p stylus-hook-miner -- \
//!     --initcode-file initcode.hex \
//!     --permissions before-swap,after-swap,before-add-liquidity,before-remove-liquidity \
//!     --constructor-signature 'stylus_constructor(address)' \
//!     --constructor-args 0x360E68faCcca8cA495c1B759Fd9EEe466db9FB32
//! ```
//!
//! It prints the `cargo stylus deploy --deployer-salt ...` command to run.

use std::{fs, process::ExitCode};

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{keccak256, Address};
use clap::Parser;
use stylus_hook_miner::{init_data, mine, MinedSalt, STYLUS_DEPLOYER};
use stylus_uniswap_v4::Permissions;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Hex init code, as printed by `cargo stylus get-initcode`.
    #[arg(long, conflicts_with = "initcode_file")]
    initcode: Option<String>,

    /// File holding the hex init code.
    #[arg(long)]
    initcode_file: Option<String>,

    /// Comma-separated hook callbacks, e.g. `before-swap,after-swap`.
    #[arg(long, value_delimiter = ',')]
    permissions: Vec<String>,

    /// The contract's Stylus constructor, as printed by `cargo stylus constructor`.
    #[arg(long)]
    constructor_signature: Option<String>,

    /// Constructor arguments, in the order the signature declares them.
    #[arg(long, num_args = 0.., value_delimiter = ' ')]
    constructor_args: Vec<String>,

    /// The `StylusDeployer` that will perform the CREATE2.
    #[arg(long, default_value_t = STYLUS_DEPLOYER)]
    deployer: Address,

    /// Give up after this many salts.
    #[arg(long, default_value_t = 2_000_000)]
    max_attempts: u64,
}

fn parse_permissions(names: &[String]) -> Result<Permissions, String> {
    let mut permissions = Permissions::none();
    for name in names {
        permissions = match name.trim() {
            "before-initialize" => permissions.with_before_initialize(),
            "after-initialize" => permissions.with_after_initialize(),
            "before-add-liquidity" => permissions.with_before_add_liquidity(),
            "after-add-liquidity" => permissions.with_after_add_liquidity(),
            "before-remove-liquidity" => permissions.with_before_remove_liquidity(),
            "after-remove-liquidity" => permissions.with_after_remove_liquidity(),
            "before-swap" => permissions.with_before_swap(),
            "after-swap" => permissions.with_after_swap(),
            "before-donate" => permissions.with_before_donate(),
            "after-donate" => permissions.with_after_donate(),
            "before-swap-return-delta" => permissions.with_before_swap_return_delta(),
            "after-swap-return-delta" => permissions.with_after_swap_return_delta(),
            "after-add-liquidity-return-delta" => {
                permissions.with_after_add_liquidity_return_delta()
            }
            "after-remove-liquidity-return-delta" => {
                permissions.with_after_remove_liquidity_return_delta()
            }
            other => return Err(format!("unknown hook callback: {other}")),
        };
    }
    Ok(permissions)
}

/// ABI-encodes the constructor arguments the same way `cargo stylus deploy` does.
fn encode_constructor_args(signature: &str, args: &[String]) -> Result<Vec<u8>, String> {
    let open = signature
        .find('(')
        .ok_or_else(|| format!("malformed constructor signature: {signature}"))?;
    let close = signature
        .rfind(')')
        .ok_or_else(|| format!("malformed constructor signature: {signature}"))?;
    // `cargo stylus constructor` prints parameter names too, e.g. `constructor(address owner)`
    let params: Vec<&str> = signature[open + 1..close]
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| p.split_whitespace().next().unwrap_or(p))
        .collect();

    if params.len() != args.len() {
        return Err(format!(
            "constructor takes {} argument(s) but {} were given",
            params.len(),
            args.len()
        ));
    }
    if params.is_empty() {
        return Ok(Vec::new());
    }

    let mut values = Vec::with_capacity(params.len());
    for (param, arg) in params.iter().zip(args) {
        let ty: DynSolType = param
            .parse()
            .map_err(|e| format!("could not resolve constructor arg `{param}`: {e}"))?;
        let value = ty
            .coerce_str(arg)
            .map_err(|e| format!("could not parse constructor arg `{arg}`: {e}"))?;
        values.push(value);
    }
    Ok(DynSolValue::Tuple(values).abi_encode_params())
}

fn run(args: Args) -> Result<(), String> {
    let initcode_hex = match (&args.initcode, &args.initcode_file) {
        (Some(hex), _) => hex.clone(),
        (None, Some(path)) => {
            fs::read_to_string(path).map_err(|e| format!("could not read {path}: {e}"))?
        }
        (None, None) => return Err("pass --initcode or --initcode-file".into()),
    };
    let initcode = hex::decode(initcode_hex.trim().trim_start_matches("0x"))
        .map_err(|e| format!("init code is not valid hex: {e}"))?;
    if initcode.is_empty() {
        return Err("init code is empty".into());
    }

    let permissions = parse_permissions(&args.permissions)?;
    let flags = permissions.flags();
    if flags == 0 {
        return Err("pass --permissions: a hook with no callbacks needs no mined address".into());
    }

    let encoded_args = match &args.constructor_signature {
        Some(signature) => encode_constructor_args(signature, &args.constructor_args)?,
        None if args.constructor_args.is_empty() => Vec::new(),
        None => return Err("--constructor-args needs --constructor-signature".into()),
    };
    let init_data = init_data(&encoded_args);
    let init_code_hash = keccak256(&initcode);

    let MinedSalt {
        salt,
        address,
        attempts,
    } = mine(
        args.deployer,
        &init_data,
        init_code_hash,
        flags,
        args.max_attempts,
    )
    .ok_or_else(|| format!("no salt found in {} attempts", args.max_attempts))?;

    println!("hook address:  {address}");
    println!("required flags: {flags:#06x}");
    println!("salt:          {salt}");
    println!("found after:   {attempts} attempt(s)");
    println!();
    println!("deploy with:");
    println!("  cargo stylus deploy \\");
    println!("    --deployer-salt {salt} \\");
    if let Some(signature) = &args.constructor_signature {
        println!("    --constructor-signature '{signature}' \\");
        if !args.constructor_args.is_empty() {
            println!(
                "    --constructor-args {} \\",
                args.constructor_args.join(" ")
            );
        }
    }
    println!("    --endpoint $RPC --private-key $PRIVATE_KEY");
    println!();
    println!(
        "The init code is what fixes this address: rebuild the contract and the salt changes."
    );

    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
