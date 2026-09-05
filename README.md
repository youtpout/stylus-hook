# stylus-hook

**Uniswap v4 hooks whose state and logic run in Rust/WASM on [Arbitrum Stylus](https://docs.arbitrum.io/stylus/gentle-introduction).**

A v4 hook has to live at an address whose low 14 bits encode its permission flags, and the
`PoolManager` calls it through the `IHooks` Solidity ABI. That address constraint — not the ABI — is
what used to force a Solidity contract into every Stylus hook.

**A hook here can now be written entirely in Rust.** [`stylus/base-hook`](stylus/base-hook) is
`BaseHook.sol`'s counterpart in Stylus, [`stylus/native-counter`](stylus/native-counter) is a hook
built on it with no Solidity anywhere, and [`stylus/hook-miner`](stylus/hook-miner) mines the
CREATE2 salt that lands it on a flag-carrying address.

`./prove-native-hook.bash` is the receipt: it stands up a chain, mines the address, deploys the Rust
hook to it and has a real `PoolManager` drive every callback the hook declares.

```
ok  landed on the mined address
ok  the code at that address is an activated Stylus program
ok  its constructor ran and validated the address
ok  beforeAddLiquidity reached the Rust hook
ok  beforeSwap reached the Rust hook
ok  afterSwap reached the Rust hook
ok  beforeRemoveLiquidity reached the Rust hook
ok  beforeDonate reverts with HookNotImplemented
```

The repository also keeps the earlier **split design**, where a stateless Solidity shell owns the
address and forwards callbacks to Stylus:

| Piece | Language | Role |
| --- | --- | --- |
| `AirdropHookProxy` / `CounterProxy` | Solidity | CREATE2-mined address carrying the permission flags; forwards every callback |
| `stylus/airdrop` / `stylus/counter` | Rust → WASM | all of the hook's storage and logic |

and the pure-Solidity equivalents `AirdropHook.sol` and `Counter.sol`, so the three approaches can
be compared behaviour-for-behaviour and gas-for-gas.

## Layout

```
uniswap/          Foundry project: the hooks, their pure-Solidity baselines, tests and deploy scripts
  src/            Counter.sol, CounterProxy.sol, AirdropHook.sol, AirdropHookProxy.sol, tokens
  test/           forge tests, including the Solidity replica of the Stylus contract
  script/         CREATE2 hook mining + deployment
stylus/           Cargo workspace of the Rust side
  base-hook/      the IHooks callbacks, v4 types and permission flags, in Stylus
  native-counter/ a hook with no Solidity at all, built on base-hook (15.7 KB WASM)
  hook-miner/     mines the CREATE2 salt for a hook address
  airdrop/        airdrop accounting behind AirdropHookProxy.sol (14.7 KB WASM)
  counter/        callback counters behind CounterProxy.sol (7.5 KB WASM)
```

## Versions

| Dependency | Version |
| --- | --- |
| Uniswap v4-core | `v4.0.0` (via `@openzeppelin/uniswap-hooks`) |
| Hook base contract | `@openzeppelin/uniswap-hooks` `BaseHook` |
| Local v4 stack for tests | [`hookmate`](https://github.com/akshatmittal/hookmate) |
| Stylus SDK / `cargo-stylus` | `0.10.9` |
| Solidity | `0.8.30`, `evm_version = "cancun"` |
| Rust | `1.91.0`, `wasm32-unknown-unknown` |

Arbitrum has supported transient storage since ArbOS 32, so this runs against **upstream v4-core** —
the 2024 version of this repo had to vendor a `no-cancun` fork.

## Requirements

```bash
curl -L https://foundry.paradigm.xyz | bash && foundryup
cargo install --locked cargo-stylus
rustup target add wasm32-unknown-unknown
git submodule update --init --recursive
```

## Test

```bash
cd uniswap && forge test
```

```bash
cd stylus && cargo test
```

The Foundry suite spins up a full local v4 stack (PoolManager, PositionManager, Permit2, router) and
runs real swaps through the hooks. Forge cannot execute WASM, so `AirdropHookProxy` is tested
against [`MockStylusAirdrop.sol`](uniswap/test/mocks/MockStylusAirdrop.sol), a Solidity replica of
the Rust contract. The Rust suite checks the same scenarios with `TestVM`, and both assert the same
airdrop amounts down to the wei.

The test that matters most for the Rust hook base is
[`selectors_match_uniswap_ihooks`](stylus/base-hook/src/hooks.rs): all ten callback selectors
computed from the Rust ABI equal the ones in the compiled `IHooks.sol`. A hook written in Rust is
only a hook if the `PoolManager`'s calls land on the right methods.

## What it costs

`./bench.bash` measures the airdrop hook on a throwaway Arbitrum Nitro dev node — the cheapest chain
that runs both EVM bytecode and WASM — by swapping through four pools and reading `gasUsed` off the
receipts.

| hook | gas per swap | costs |
| --- | ---: | ---: |
| none | 124,590 | — |
| one Solidity contract | 160,276 | +35,686 |
| two Solidity contracts | 166,835 | +42,245 |
| Solidity shell + Stylus | 201,121 | +76,531 |

Stylus costs 2.14× what Solidity does here. That is the workload's fault, not the port's: `afterSwap`
is six `SLOAD`s and six `SSTORE`s with almost no arithmetic, and Stylus makes compute cheap, not
storage.

`./bench-compute.bash` shows the other side of it, running the same arithmetic loop in both
languages and sweeping how much of it there is:

| rounds of work | Solidity | Rust |
| ---: | ---: | ---: |
| 0 | 129,910 | 162,554 |
| 200 | 158,482 | 164,703 |
| 500 | 201,410 | **167,995** |
| 5,000 | 844,878 | **216,929** |

Computation is **13× cheaper** in Stylus — 10.9 gas per round against 143 — but a Stylus call costs
32,644 gas more to enter, most of it loading the WASM program. The two cross at roughly 250 rounds.
[BENCHMARK.md](BENCHMARK.md) has the full breakdown.

## Deploy a hook with no Solidity (Arbitrum Sepolia)

`cargo stylus deploy` routes through the on-chain `StylusDeployer`, which uses CREATE2 when handed a
non-zero salt. Mine a salt for the permissions the hook declares, then deploy with it:

```bash
cd stylus
cargo stylus get-initcode --contract stylus-native-counter | tail -1 > initcode.hex
cargo run -p stylus-hook-miner -- \
  --initcode-file initcode.hex \
  --permissions before-swap,after-swap,before-add-liquidity,before-remove-liquidity \
  --constructor-signature 'constructor(address pool_manager)' \
  --constructor-args 0xFB3e0C6F74eB1a21CC1Da29aeC80D2Dfe6C9a317
```

It prints the mined address, the salt and the `cargo stylus deploy --deployer-salt ...` command to
run. The address is fixed by the init code, so rebuilding the contract changes the salt.

## Deploy the split design (Arbitrum Sepolia)

Uniswap v4 and Stylus are both live on Arbitrum Sepolia, so no local node is needed.

```bash
cd stylus
cargo stylus deploy --contract stylus-airdrop-hook \
  --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key $PRIVATE_KEY
```

Then mine the hook address, deploy the shell and bind it to the Stylus contract:

```bash
cd uniswap && STYLUS_AIRDROP=0x... forge script script/01_DeployStylusAirdropHook.s.sol \
  --rpc-url arbitrum_sepolia --broadcast
```

`./deploy.bash` runs both steps. The counter hook follows the same two commands with
`stylus-counter-hook` / `02_DeployStylusCounterHook.s.sol`, and
`00_DeployAirdropHook.s.sol` deploys the pure-Solidity baseline for comparison.

## Where to look

| What | File |
| --- | --- |
| `IHooks` callbacks implemented in Rust | [`stylus/base-hook/src/hooks.rs`](stylus/base-hook/src/hooks.rs) |
| Permission flags v4 reads from an address | [`stylus/base-hook/src/permissions.rs`](stylus/base-hook/src/permissions.rs) |
| Calling back into the `PoolManager` from Rust | [`stylus/base-hook/src/pool_manager.rs`](stylus/base-hook/src/pool_manager.rs) |
| A hook with no Solidity at all | [`stylus/native-counter/src/lib.rs`](stylus/native-counter/src/lib.rs) |
| End-to-end proof that it works | [`prove-native-hook.bash`](prove-native-hook.bash) |
| CREATE2 salt mining for a hook address | [`stylus/hook-miner/src/lib.rs`](stylus/hook-miner/src/lib.rs) |
| Gas benchmarks | [`bench.bash`](bench.bash), [`bench-counter.bash`](bench-counter.bash), [`bench-compute.bash`](bench-compute.bash) |
| Hook forwarding v4 callbacks to Stylus | [`uniswap/src/AirdropHookProxy.sol`](uniswap/src/AirdropHookProxy.sol) |
| The same hook in pure Solidity | [`uniswap/src/AirdropHook.sol`](uniswap/src/AirdropHook.sol) |
| Hook state and logic in Rust | [`stylus/airdrop/src/lib.rs`](stylus/airdrop/src/lib.rs) |
| ABI boundary between the two | [`uniswap/src/interfaces/IAirdropHook.sol`](uniswap/src/interfaces/IAirdropHook.sol) |
| CREATE2 mining + binding | [`uniswap/script/01_DeployStylusAirdropHook.s.sol`](uniswap/script/01_DeployStylusAirdropHook.s.sol) |
