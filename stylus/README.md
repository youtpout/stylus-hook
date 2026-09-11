# stylus — the Rust side

Cargo workspace holding everything written in Rust. See the [root README](../README.md) for the
overall design.

| Crate | What it is | WASM |
| --- | --- | --- |
| [`base-hook`](base-hook/src/lib.rs) | `BaseHook.sol`'s counterpart in Stylus: v4 types, permission flags, the ten `IHooks` callbacks, and the `PoolManager` calls a hook makes | library |
| [`native-counter`](native-counter/src/lib.rs) | a v4 hook with no Solidity at all, built on `base-hook` | 15.7 KB |
| [`hook-miner`](hook-miner/src/lib.rs) | mines the CREATE2 salt that puts a Stylus contract on a hook address | host CLI |

`hook-miner` runs on the host, so it is a workspace member but not a *default* member — the wasm
build skips it. Use `cargo test --workspace` to include its tests.

```bash
cargo test --workspace                                   # unit tests, via stylus_sdk::testing::TestVM
cargo build --target wasm32-unknown-unknown --release    # the contracts
cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc
```

## Writing a hook in Rust

See [`base-hook/README.md`](base-hook/README.md).

## Deploying to a hook address

v4 reads a hook's permissions out of the low 14 bits of its address, so the address has to be mined.
`cargo stylus deploy` goes through the on-chain
[`StylusDeployer`](https://github.com/OffchainLabs/nitro-contracts/blob/main/src/stylus/StylusDeployer.sol),
which uses CREATE2 whenever the salt is non-zero — so no Solidity factory is needed.

```bash
cargo stylus get-initcode --contract stylus-native-counter | tail -1 > initcode.hex
cargo run -p stylus-hook-miner -- \
  --initcode-file initcode.hex \
  --permissions before-swap,after-swap,before-add-liquidity,before-remove-liquidity \
  --constructor-signature "$(cargo stylus constructor --contract stylus-native-counter | tail -1)" \
  --constructor-args 0xFB3e0C6F74eB1a21CC1Da29aeC80D2Dfe6C9a317
```

The miner prints the mined address, the salt, and the `cargo stylus deploy --deployer-salt ...`
command to run. The address is derived from the init code, so rebuilding the contract changes the
salt.
