// Bakes the pool manager address in as a compile-time constant.
//
// The Stylus SDK has no `immutable`, so a constructor argument could only go to storage and every
// callback would pay a cold SLOAD to check its caller. A `const` lives in the WASM and is free, at
// the price of being fixed at build time — which costs nothing, since the address is mined anyway.
use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-env-changed=POOL_MANAGER");
    println!("cargo:rerun-if-env-changed=EXPIRATION_INTERVAL");

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
    // Orders expire on a grid, and the benchmark has to cross several of those boundaries to
    // measure what one costs. There is no way to move a nitro dev node's clock — it has no
    // `evm_increaseTime` — so the only way to cross them is to wait, and the only way to wait a
    // sensible length of time is to build the benchmark with a short grid.
    let interval = env::var("EXPIRATION_INTERVAL").unwrap_or_else(|_| "60".to_string());
    let interval: u64 = interval
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("EXPIRATION_INTERVAL must be a whole number of seconds"));
    assert!(interval > 0, "EXPIRATION_INTERVAL must be positive");

    let src = format!(
        "{src}\n\
         /// Seconds between order expiries, fixed at build time. See `build.rs`.\n\
         pub const EXPIRATION_INTERVAL: u64 = {interval};\n"
    );
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("pool_manager.rs"),
        src,
    )
    .unwrap();
}
