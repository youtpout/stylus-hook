// Bakes the pool manager address into the contract as a compile-time constant.
//
// Solidity hooks hold it in an `immutable` and read it for free. The Stylus SDK has no immutable,
// so a constructor argument can only go to storage and every callback pays a cold SLOAD to check
// its caller. A `const` costs nothing to read — it lives in the WASM code — at the price of being
// fixed at build time rather than deploy time.
//
// That is a fair trade for a hook: the address has to be mined against the init code anyway, so the
// contract is already rebuilt per deployment.
//
//   POOL_MANAGER=0x360E68faCcca8cA495c1B759Fd9EEe466db9FB32 cargo stylus deploy ...
use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-env-changed=POOL_MANAGER");

    let raw = env::var("POOL_MANAGER")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());
    let hex = raw.trim().trim_start_matches("0x");
    assert_eq!(
        hex.len(),
        40,
        "POOL_MANAGER must be a 20-byte hex address, got {raw:?}"
    );

    let bytes: Vec<String> = (0..20)
        .map(|i| {
            let byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .unwrap_or_else(|_| panic!("POOL_MANAGER is not hex: {raw:?}"));
            format!("0x{byte:02x}")
        })
        .collect();

    let src = format!(
        "/// The v4 singleton, fixed at build time. See `build.rs`.\n\
         pub const POOL_MANAGER: Address = Address::new([{}]);\n",
        bytes.join(", ")
    );
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("pool_manager.rs"),
        src,
    )
    .unwrap();
}
