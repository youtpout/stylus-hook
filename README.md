# stylus-hook

**Uniswap v4 hooks whose state and logic run in Rust/WASM on [Arbitrum Stylus](https://docs.arbitrum.io/stylus/gentle-introduction).**

A v4 hook has to live at an address whose low 14 bits encode its permission flags, and the
`PoolManager` calls it through the `IHooks` Solidity ABI. That address constraint — not the ABI — is
what used to force a Solidity contract into every Stylus hook.

**A hook here can now be written entirely in Rust.** [`stylus/base-hook`](stylus/base-hook) is
`BaseHook.sol`'s counterpart in Stylus, [`stylus/native-counter`](stylus/native-counter) is a hook
built on it with no Solidity anywhere, and [`stylus/hook-miner`](stylus/hook-miner) mines the
CREATE2 salt that lands it on a flag-carrying address.

📖 **[Writing a hook](stylus/base-hook/README.md)** — five steps, with the counter as the worked
example. The documentation site covers the same ground in three pages:
[why Stylus](docs/index.html), [the worked example and deployment](docs/example.html), and
[the benchmarks](docs/benchmark.html).
📝 **[FEEDBACK.md](FEEDBACK.md)** — feedback to Uniswap from porting v4-core, including a failing
input in v4-core's own `TickMath` fuzz test.

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

`Counter.sol` is the same hook written in pure Solidity, so the two can be compared
behaviour-for-behaviour and gas-for-gas.

## Layout

```
uniswap/          Foundry project: the hooks, their pure-Solidity baselines, tests and deploy scripts
  src/            the hooks and their pure-Solidity twins, tokens
  test/           forge tests
  script/         CREATE2 hook mining + deployment
stylus/           Cargo workspace of the Rust side
  base-hook/            the IHooks callbacks, v4 types and permission flags, in Stylus
  base-hook-macros/     #[guarded_hooks]
  native-counter/       a hook with no Solidity at all, built on base-hook
  native-twamm/         a complete TWAMM: orders, expiries, settlement
  native-gaussian/      solstat's Gaussian, ported bit-exactly
  native-compute/       arithmetic sweeps, for the crossover
  hook-miner/           mines the CREATE2 salt for a hook address
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
runs real swaps through the Solidity hooks. Forge cannot execute WASM, so the Rust side is tested with
`TestVM` on the Cargo side, and where the two implement the same maths they are pinned to the same
values — `PmAmmMath.sol` and `stylus/native-gaussian` assert an identical table, to the wei.

The test that matters most for the Rust hook base is
[`selectors_match_uniswap_ihooks`](stylus/base-hook/src/hooks.rs): all ten callback selectors
computed from the Rust ABI equal the ones in the compiled `IHooks.sol`. A hook written in Rust is
only a hook if the `PoolManager`'s calls land on the right methods.

## What it costs

Every benchmark stands up a throwaway Arbitrum Nitro dev node in Docker — the cheapest chain that
runs both EVM bytecode and WASM — deploys both implementations, and reads `gasUsed` off the receipts.

`./bench-counter.bash`, on a hook that only counts callbacks — two hook calls per swap:

| hook | gas per swap | costs |
| --- | ---: | ---: |
| none | 115,065 | — |
| `Counter.sol`, one Solidity contract | 134,003 | +18,938 |
| `native-counter`, cached | 168,941 | +53,876 |

Stylus costs 2.8× what Solidity does here, and that is the workload's fault rather than the port's:
the hook writes a storage slot per callback and computes nothing, and Stylus makes compute cheap, not
storage. This is the floor, and it is here so the wins further down are read against it. Uncached the
same swap costs 199,595 — the hook is entered twice and pays the WASM load each time, which is why
every figure in this repository is the cached one.

`./bench-compute.bash` shows the other side of it, running the same arithmetic in both languages and
sweeping how much of it there is. The interesting column is `mulDiv`, because that is what Uniswap's
own swap math is made of:

| `mulDiv`s per swap | Solidity | Rust |
| ---: | ---: | ---: |
| 0 | 124,362 | 163,869 |
| 50 | 159,158 | 176,341 |
| 200 | 263,258 | **213,469** |
| 1,000 | 818,458 | **411,485** |
| 5,000 | 3,594,426 | **1,401,533** |

694 gas per `mulDiv` in Solidity against 247 in Rust — **2.8× cheaper** — and a Stylus call costs
~39,000 gas more to enter, so the two cross at **89 operations**. Rust is cheaper on every kind of
arithmetic measured, by 1.65× to 10.6×.

One workload does clear it, and it was measured against the TWAMM that is actually deployed —
[`akshatmittal/v4-twamm-hook`](https://github.com/akshatmittal/v4-twamm-hook), by Uniswap Labs and
Zaha Studio, audited by ABDK and Certora, live on Base and Unichain. Both hooks on the same node,
same pool manager, same expiry grid, same order book, with the Stylus program cached:

| | Rust | production Solidity |
| --- | ---: | ---: |
| swap, pool idle | 140,173 | 132,257 |
| swap, one span of virtual orders | **305,104** | 339,612 |
| **per expiry crossed** | **26,924** | **38,834** |

**31 % cheaper per interval, 7,916 gas worse on an idle pool**, and ahead from the first span of
work. The Rust hook is [`stylus/native-twamm`](stylus/native-twamm/src/lib.rs) — orders, order
pools, an expiry grid, earnings factors and settlement against the v4 singleton through
[`PoolManagerCalls`](stylus/base-hook/src/pool_manager.rs), with no Solidity in it.

It is not a clean language comparison: the two implementations differ, mine settles once per catch-up
where theirs settles per interval, and mine is the less finished of the two. `BENCHMARK.md` says so
in more detail.

The pm-AMM was the next candidate, and the one whose arithmetic genuinely cannot be removed — its
invariant is transcendental, so every swap must run a Gaussian solve.
[`stylus/native-gaussian`](stylus/native-gaussian/src/gaussian.rs) is a bit-exact port of
`primitivefinance/solstat`, agreeing with it to the wei on chain:

| | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| Gaussian CDF | 5,137 | **2,082** | 2.47× |
| solve, 8 iterations | 62,425 | **27,659** | 2.26× |

**+26,850 gas per swap in Rust's favour**, on every swap. Getting there took a wasm-opt flag, and it
is the single most useful thing in this repository: without `--llvm-memory-copy-fill-lowering`,
rustc's `opt-level = 2` and `3` emit a `DataCount` section ArbOS refuses to activate, so a Stylus
contract is capped at `"s"` — **compiled for size, on a platform that charges for execution.** The
build succeeds, `cargo stylus check` succeeds, and only activation fails. Every benchmark in this
repository was measuring size-optimised code until that was found, and every ratio roughly doubled
when it was fixed — in proportion to how arithmetic-bound the workload is, and not at all for the
storage-bound ones.

The widest margin, though, is on arithmetic that was in the codebase the whole time: v4-core's own
swap math. [`stylus/v4-math`](stylus/v4-math/src/swap_math.rs) is `SwapMath`, `TickMath`,
`SqrtPriceMath`, `FullMath` and `UnsafeMath` ported function for function and tested against
v4-core's own vectors — every unit test in `test/libraries/` for those five, same inputs, same
expected values. The control side is not a reimplementation:
[`V4MathBench.sol`](uniswap/src/V4MathBench.sol) calls Uniswap's libraries directly.

The workload is `Pool.swap`'s loop without the storage — find the next initialised tick, price the
step, cross, repeat — which is what OpenZeppelin's `AntiSandwichHook` and Uniswap's own
`alf/SwapSimulator` replay on every swap:

| ticks crossed | Solidity | Rust | saving |
| ---: | ---: | ---: | ---: |
| 4 | 42,451 | 46,522 | −4,071 |
| 8 | 58,870 | **48,380** | **+10,490** |
| 32 | 155,342 | **62,055** | **+93,287** |

**4,056 gas per tick against 570 — 7.1×**, `computeSwapStep` alone 4.55× and `getSqrtPriceAtTick`
4.57×. A replay crossing six or more ticks is a net win. This code is bit-twiddling rather than
big-number arithmetic — nineteen shifts and a conditional multiply for a tick's price — and Solidity
pays a 5-gas opcode for every one of them with no cheaper way to write it.

That result also produced the second trap worth knowing. The first run had Rust *losing* 3.6× on
`getSqrtPriceAtTick`, because the nineteen fixed-point factors were `&str` constants parsed with
`from_str_radix` inside the loop. Hoisting them to real `const` values moved the ratio from 0.28× to
4.57× with the algorithm untouched. In Solidity a literal is a literal and this bug cannot be
written; in Rust, building a `U256` from text is the most readable option and about a hundred times
the cost of the arithmetic it feeds.

Everything above is a hook Solidity could write more expensively. The last one it cannot write at
all. [`stylus/native-crypto`](stylus/native-crypto/src/keccak.rs) is Keccak-f[1600], SHAKE256 and
ML-DSA's number-theoretic transform — the two primitives a post-quantum signature check spends its
gas on. Both sides are pinned to NIST's SHAKE256 vectors and XKCP's permutation vector before
anything is timed, and [the Solidity control](uniswap/src/CryptoBench.sol) is unrolled inline
assembly, not the readable version that would have made a strawman of it.

| | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| one 32-byte hash, through the built-in | 184 | 16 | 11.5× |
| one Keccak-f[1600] permutation, by hand | 101,662 | **158** | **643×** |
| SHAKE256, absorb 200 B, squeeze 1088 B | 1,250,263 | **2,559** | **489×** |
| one forward NTT, 256 coefficients | 244,873 | **2,128** | **115×** |

The first row is free money for any hook that hashes: the Stylus host's keccak is 11.5× cheaper than
the EVM opcode. The second is the one that matters. `keccak256` of 32 bytes costs 36 gas of opcode
and performs exactly one permutation internally — but it is Keccak-256 with the `0x01` pad compiled
in, and SHAKE pads with `0x1f`. So SHAKE cannot come from any built-in, in either language, and the
permutation has to be written out. **The EVM can do Keccak, but only through the one door it
provides, and SHAKE is not behind that door.**

Projected onto one ML-DSA-44 verification — about 90 permutations and 9 transforms — that is
**11,353,437 gas in Solidity against 33,372 in Rust**. A swap on Arbitrum costs about 115,000 gas
with no hook. Solidity would spend ninety-nine swaps' worth to check one signature; Rust spends less
than a third of one. A hook that only accepts post-quantum-signed orders is not expensive in
Solidity, it is impossible, and in Rust it is unremarkable.

`BaseHook.sol` is an abstract contract, so its `onlyPoolManager` guard cannot be forgotten. Rust has
no abstract types, so [`#[guarded_hooks]`](stylus/base-hook-macros/src/lib.rs) does the same job with
a procedural macro: it inserts the guards into the callbacks a hook writes, and costs nothing.

The catch is that most hooks barely compute. A cached Stylus hook carries about 7,900 gas per call,
so it needs roughly 12,000 gas of Solidity arithmetic to break even — 11 `rpow` calls, or 45
`mulDiv`s. Hooks that do that much win, and win widely; a hook that only counts swaps does essentially
none and loses. [BENCHMARK.md](BENCHMARK.md) has every sweep and the crossover for each operation.

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

## Licensing

This repository is MIT — see [LICENSE](LICENSE). Two files carry their own terms and say so in their
headers:

| file | licence | why it is here |
| --- | --- | --- |
| [`uniswap/src/vendor/ABDKMathQuad.sol`](uniswap/src/vendor/ABDKMathQuad.sol) | BSD-4-Clause, © ABDK Consulting | IEEE 754 binary128 in software. Vendored to be measured, not used — it is what makes Uniswap's TWAMM cost what it does |
| [`uniswap/script/bench/vendor/StylusDeployer.sol`](uniswap/script/bench/vendor/StylusDeployer.sol) | MIT, Offchain Labs | `cargo stylus` routes CREATE2 deployments through it, and it exists only on real Arbitrum chains |

Both are permissive and impose nothing on the rest of the tree.

Nothing under a copyleft or source-available licence is reproduced here. In particular Uniswap's own
`TwammMath` is marked `UNLICENSED` inside a GPL-2.0 repository, so
[`TwammHook.sol`](uniswap/src/TwammHook.sol) implements the published closed form (Paradigm, 2021)
from the formula instead, and EulerSwap — BUSL-1.1 — is referenced in
[BENCHMARK.md](BENCHMARK.md) for information with none of its code copied.

## Where to look

| What | File |
| --- | --- |
| `IHooks` callbacks implemented in Rust | [`stylus/base-hook/src/hooks.rs`](stylus/base-hook/src/hooks.rs) |
| Permission flags v4 reads from an address | [`stylus/base-hook/src/permissions.rs`](stylus/base-hook/src/permissions.rs) |
| Calling back into the `PoolManager` from Rust | [`stylus/base-hook/src/pool_manager.rs`](stylus/base-hook/src/pool_manager.rs) |
| A hook with no Solidity at all | [`stylus/native-counter/src/lib.rs`](stylus/native-counter/src/lib.rs) |
| End-to-end proof that it works | [`prove-native-hook.bash`](prove-native-hook.bash) |
| CREATE2 salt mining for a hook address | [`stylus/hook-miner/src/lib.rs`](stylus/hook-miner/src/lib.rs) |
| The guard attribute, and why it is a proc macro | [`stylus/base-hook-macros/src/lib.rs`](stylus/base-hook-macros/src/lib.rs) |
| How to write a hook, step by step | [`stylus/base-hook/README.md`](stylus/base-hook/README.md) |
| A complete TWAMM in Rust: orders, expiries, settlement | [`stylus/native-twamm/src/lib.rs`](stylus/native-twamm/src/lib.rs) |
| The workload that wins: solstat's Gaussian, ported bit-exactly | [`stylus/native-gaussian/src/gaussian.rs`](stylus/native-gaussian/src/gaussian.rs) |
| Its Solidity twin, pinned to the same values | [`uniswap/src/PmAmmMath.sol`](uniswap/src/PmAmmMath.sol) |
| Gas benchmarks | [`bench-pmamm.bash`](bench-pmamm.bash), [`bench-twamm.bash`](bench-twamm.bash), [`bench-compute.bash`](bench-compute.bash), [`bench-counter.bash`](bench-counter.bash) |
| Opcode profile of shipping hooks | [`profile-hooks.bash`](profile-hooks.bash) |
| A TWAMM under concurrent order flow | [`bench-twamm-concurrent.bash`](bench-twamm-concurrent.bash) |
| v4-core's swap math in Rust, on Uniswap's own vectors | [`stylus/v4-math/src/swap_math.rs`](stylus/v4-math/src/swap_math.rs), [`tick_math.rs`](stylus/v4-math/src/tick_math.rs), [`sqrt_price_math.rs`](stylus/v4-math/src/sqrt_price_math.rs), [`full_math.rs`](stylus/v4-math/src/full_math.rs) |
| `Pool.swap`'s loop, the workload that wins at 7.1× | [`stylus/native-v4-math/src/lib.rs`](stylus/native-v4-math/src/lib.rs) |
| Its control, calling v4-core's libraries directly | [`uniswap/src/V4MathBench.sol`](uniswap/src/V4MathBench.sol) |
| Keccak-f, SHAKE256 and ML-DSA's NTT, at 643× | [`stylus/native-crypto/src/keccak.rs`](stylus/native-crypto/src/keccak.rs), [`ntt.rs`](stylus/native-crypto/src/ntt.rs) |
| Their Solidity control, in unrolled assembly | [`uniswap/src/CryptoBench.sol`](uniswap/src/CryptoBench.sol) |
| Gas benchmarks, the two newest | [`bench-v4-math.bash`](bench-v4-math.bash), [`bench-crypto.bash`](bench-crypto.bash) |
| Deploying a hook too large for one code fragment | [`bench-lib.bash`](bench-lib.bash#L51) |
| **Feedback to Uniswap, including a bug in v4-core's tests** | [**`FEEDBACK.md`**](FEEDBACK.md) |
