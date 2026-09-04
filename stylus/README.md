# stylus — the Rust side

Cargo workspace holding the Stylus contracts that back the Uniswap v4 hooks. See the
[root README](../README.md) for the overall design.

| Crate | Contract | WASM |
| --- | --- | --- |
| [`airdrop`](airdrop/src/lib.rs) | airdrop accounting, driven by `AirdropHookProxy.sol` | 14.7 KB |
| [`counter`](counter/src/lib.rs) | per-pool callback counters, driven by `CounterProxy.sol` | 7.5 KB |

Both contracts only accept writes from the v4 hook bound through `setHook`, which can be set once.

```bash
cargo test                                        # unit tests against stylus_sdk::testing::TestVM
cargo stylus check -e https://sepolia-rollup.arbitrum.io/rpc
cargo run -p stylus-airdrop-hook --features export-abi   # Solidity interface for the hook side
```

`export-abi` output must stay in sync with `uniswap/src/interfaces/IAirdropHook.sol` and
`uniswap/src/ICounter.sol` — those are what the Solidity hooks call through.

## Deploy

```bash
cargo stylus deploy --contract stylus-airdrop-hook \
  --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key $PRIVATE_KEY
```

Then deploy and bind the hook shell with `uniswap/script/01_DeployStylusAirdropHook.s.sol`.
