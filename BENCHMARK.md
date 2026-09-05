# What a hook costs per swap

## Method

Every figure here comes from a transaction on an Arbitrum Nitro dev node — the cheapest chain that
executes both EVM bytecode and WASM — with one swap per transaction, so the cost is read off the
receipt rather than estimated.

Both sides are optimised. Solidity is compiled with `optimizer = true, optimizer_runs = 200`, which
is what OpenZeppelin's `uniswap-hooks` ships with; Rust is built in release with LTO and the pinned
`wasm-opt -Oz` that `Stylus.toml` declares. An earlier revision of this file reported numbers taken
with the Solidity optimiser off, which flattered Stylus — those have been remeasured.

The benchmarks also set `ArbOwner.setL1PricePerUnit(0)`. The L1 data-posting component is identical
across the variants being compared — same calldata, same swap — and on a real chain it dwarfs the
L2 execution gas the comparison is about.

# What the airdrop hook costs per swap

Measured on an Arbitrum Nitro dev node (ArbOS 59, Stylus v3), the cheapest chain that can execute
both EVM bytecode and WASM. Reproduce with:

```bash
./bench.bash
```

It stands up a throwaway node in Docker, deploys a full Uniswap v4 stack and four pools onto it,
then swaps through each in its own transaction and reads `gasUsed` off the receipts.

## The four pools

| Pool | Hook |
| --- | --- |
| control | none |
| solidity | `AirdropHook.sol` — one contract, Solidity |
| replica | `AirdropHookProxy.sol` → `MockStylusAirdrop.sol` — two contracts, both Solidity |
| stylus | `AirdropHookProxy.sol` → `stylus/airdrop` — two contracts, the second one Rust/WASM |

`replica` is the control that makes the comparison fair. It has the same two-contract shape as
`stylus` and runs the identical accounting, so the gap between the two is the cost of Stylus itself
rather than the cost of the extra call.

## Results

Gas per swap, warm (the first swap on a pool pays zero-to-non-zero `SSTORE` prices and is discarded):

| | gas per swap | hook costs |
| --- | ---: | ---: |
| no hook | 119,562 | — |
| hook in one Solidity contract | 153,107 | +33,545 |
| hook split over two Solidity contracts | 158,688 | +39,126 |
| **hook with its state in Stylus** | **194,049** | **+74,487** |
| hook with its state in Stylus, if cached | 176,822 (projected) | +57,260 |

Breaking down the 74,487:

| | gas |
| --- | ---: |
| the accounting itself, as Solidity would do it | 33,545 |
| the extra call the split design needs | 5,581 |
| Stylus, over the identical Solidity callee | 35,361 |
| — of which loading the WASM program, uncached | 20,439 |
| — the same, once the contract is cached | 3,212 |

**Stylus costs 2.22× what Solidity does for this hook, or 1.71× once the contract is cached.**

The dev node has no `CacheManager`, so the cached row is computed from
`ArbWasm.programInitGas(address)`, which reports both figures, rather than measured directly.

## Why Stylus loses here

Stylus makes *compute* roughly an order of magnitude cheaper. It does not make *storage* cheaper:
`SLOAD` and `SSTORE` are host calls priced in EVM gas either way.

`afterSwap` on this hook does about six `SLOAD`s and six `SSTORE`s and almost no arithmetic — six
warm `SSTORE`s alone are 17,400 gas of the 33,545 the Solidity version costs. There is no compute
for Stylus to win back, so all that is left is what it adds: ~20k to load the program on a cold
call, plus the cross-VM call itself.

This is a property of the workload, not of the implementation. Nothing in
[`stylus/airdrop/src/lib.rs`](stylus/airdrop/src/lib.rs) is doing more storage work than
[`AirdropHook.sol`](uniswap/src/AirdropHook.sol) — they write the same six slots, and both suites
assert the same airdrop amounts to the wei.

## The counter, three ways

`./bench-counter.bash` asks the follow-up question directly: does dropping the Solidity shell
recover what it costs? Same hook, three implementations, one swap per transaction. A swap hits this
hook twice — `beforeSwap` and `afterSwap` — so every figure below is two hook calls.

| | gas per swap | hook costs |
| --- | ---: | ---: |
| no hook | 115,125 | — |
| `Counter.sol`, one Solidity contract | 134,063 | +18,938 |
| `CounterProxy.sol` → Stylus | 199,968 | +84,843 |
| **`native-counter`, no Solidity in the hook** | **199,353** | **+84,228** |
| the same, if the contract is cached | 184,256 (projected) | +69,131 |

Going fully native saves **615 gas** — a rounding error — and only because of Binaryen. `-Oz` cuts
the native hook's init gas from 21,274 to 17,751, and this hook is entered twice per swap, so
without it the shell would win by several thousand:

| | compiled size | init gas, uncached / cached |
| --- | ---: | ---: |
| `native-counter`, no `wasm-opt` | 1,044,480 | 21,274 / 3,585 |
| `native-counter`, `-Oz` | 862,208 | 17,751 / 2,654 |
| the contract behind `CounterProxy` | 779,264 | 15,010 / 1,977 |

Two things pull against going native, and they nearly cancel the win:

- The native hook carries the whole `IHooks` router and the ABI decoders for `PoolKey` and
  `SwapParams`, so its program is bigger — and `ArbWasm` prices `programInitGas` off compiled size,
  charged on every call.
- v4 hands a hook the full `PoolKey`, `SwapParams` and `hookData`. The native hook decodes all of
  that in WASM and hashes the key there too. The Solidity shell does both in the EVM, where this
  shape is cheap, and passes Stylus a single `bytes32`.

So the shell is not only overhead: it doubles as a calldata decoder. Removing it is still the right
call — one less contract, one less trust assumption, and it no longer costs anything — but on gas
alone it is a wash, and it will only pay for a hook whose own work is worth more than its front
door.

Against pure Solidity the native hook still costs 65,290 gas more per swap, 4.4× the Solidity hook.
Same conclusion as the airdrop: this hook writes one storage slot per callback and computes
nothing.

## Where Rust starts winning

The two hooks above both lose to Solidity, which invites the wrong conclusion. Stylus is not slower;
it is *differently priced*. It costs more to enter and far less to run. `./bench-compute.bash`
measures both halves of that: `ComputeHook.sol` and `stylus/native-compute` run the identical
xorshift64 loop in `beforeSwap`, and the only thing that changes between rows is how many rounds.

### xorshift64 on a `u64`

| rounds | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 124,342 | 162,918 | +38,576 |
| 50 | 130,064 | 163,434 | +33,370 |
| 200 | 147,314 | 165,066 | +17,752 |
| 500 | 181,842 | **168,359** | −13,483 |
| 1,000 | 239,378 | **173,835** | −65,543 |
| 5,000 | 699,310 | **217,293** | −482,017 |

115 gas per round in Solidity, 10.9 in Rust: **10.6× cheaper**. The crossover is at ~371 rounds.

### `mulDiv` on 256-bit words

This is the one that matters, because it is what Uniswap's own swap math is made of.
`SwapMath.computeSwapStep`, `SqrtPriceMath` and every tick-walking simulation are mostly chains of
`FullMath.mulDiv`.

| rounds | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 124,362 | 163,869 | +39,507 |
| 50 | 159,158 | 176,341 | +17,183 |
| 200 | 263,258 | **213,469** | −49,789 |
| 500 | 471,394 | **287,661** | −183,733 |
| 1,000 | 818,458 | **411,485** | −406,973 |
| 5,000 | 3,594,426 | **1,401,533** | −2,192,893 |

694 gas per `mulDiv` in Solidity, 247 in Rust: **2.8× cheaper**, and the crossover is **89
operations** — about 62,000 gas of Solidity-side arithmetic.

That Stylus wins here at all is the opposite of what the 256-bit word size suggests, and the reason
is worth stating. A bare `MUL` or `DIV` *is* a single 5-gas EVM opcode, and Stylus would lose that
comparison. But Uniswap does not use bare `mul` and `div` for pool math — it uses `FullMath.mulDiv`,
which needs the full 512-bit product, and the EVM has no 512-bit anything. Solidity gets there with
Remco Bloemen's long-division routine: `mulmod`, a modular inverse built by Newton iteration, and
several dozen opcodes. Rust gets there with a `U512` multiply and divide over limbs. The EVM's
256-bit word is an advantage right up to the point where you need 257 bits, which is most of
Uniswap's math.

### `rpow`, the operation Bunni actually runs

`LibGeometricDistribution` calls `alphaX96.rpow(length, Q96)` several times per Liquidity Density
Function query, and BunniHook queries the LDF on every swap. One `rpow` is a chain of
full-precision `mulDiv`s, so it is the right unit for pricing a hook that replaces the AMM curve.

| `rpow(·, 100)` per swap | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 124,560 | 164,255 | +39,695 |
| 10 | 155,172 | 182,795 | +27,623 |
| 25 | 201,106 | 210,621 | +9,515 |
| 50 | 277,492 | **256,827** | −20,665 |
| 100 | 430,328 | **349,303** | −81,025 |

3,058 gas per `rpow` in Solidity, 1,850 in Rust: **1.65× cheaper**, crossing at **33 calls**.

That is *less* than the 2.8× measured for `mulDiv`, which looks contradictory until you look at what
`FullMath.mulDiv` actually does. It has two paths. When the 512-bit product needs all 512 bits it
runs Remco Bloemen's long division — dozens of opcodes, and Stylus wins 2.8×. When the product fits
in 256 bits it is a single `DIV`, which costs 5 gas and which Stylus cannot beat.

Bunni's LDF math is in Q96, so its products fit and it takes the cheap path almost everywhere.
**How much Stylus buys depends on the magnitudes a hook works with, not just on how much arithmetic
it does.** Anything using the full 256-bit range benefits about twice as much as fixed-point Q96
work.

### Integer `sqrt`, the operation EulerSwap runs

EulerSwap inverts its curve with the quadratic formula, so every swap takes the square root of a
discriminant its own source comments put in the 255-bit range.

| `sqrt` per swap | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 123,694 | 164,819 | +41,125 |
| 25 | 174,584 | 176,654 | +2,070 |
| 50 | 224,709 | **187,724** | −36,985 |
| 100 | 324,931 | **209,837** | −115,094 |
| 200 | 525,459 | **254,147** | −271,312 |

2,009 gas per `sqrt` in Solidity, 447 in Rust: **4.5× cheaper**, crossing at **26 calls** — the best
ratio measured anywhere in this document.

Two reasons, and both generalise. The values span the full 256-bit range, so the `mulDiv`s inside
the iteration take the expensive path rather than the single-`DIV` one. And the Babylonian seed
needs the operand's bit length: the EVM has no instruction for it, so Solidity spends a cascade of
comparisons or a de Bruijn table, while `leading_zeros` in Rust lowers to WASM's `i64.clz`.

> The algorithm here is written from the published description of what EulerSwap's curve does, not
> copied from it. EulerSwap is licensed BUSL-1.1, which restricts production use; it is referenced
> in this document for information only.

### Building a hook to clear the bar, and failing

Every shipping hook profiled here is dominated by storage, so the obvious move was to build one that
is not. `StableSwapHook.sol` and `stylus/native-stableswap` price a swap on a StableSwap curve:
solving the invariant `D` takes about four Newton iterations and the output reserve `y` about eight
more, so twelve iterations of 256-bit arithmetic per swap, with no storage beyond two reserves.
`./bench-stableswap.bash` runs both, and the benchmark checks they quote the same output before
measuring.

| | gas per swap | over baseline |
| --- | ---: | ---: |
| no hook | 115,061 | — |
| StableSwap in Solidity | 149,112 | +34,051 |
| **StableSwap in Rust** | **174,417** | **+59,356** |

**Rust loses by 25,305 gas.** So the next question is where that goes, and the answer is not where
it first appears to be. Calling the pure functions directly, with no hook and no storage in the way:

| | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| 100 × `a * b / c`, no overflow | 27,834 | 9,390 | 2.96× |
| 400 × | 111,365 | 37,534 | 2.97× |
| 1,000 × | 279,387 | 92,622 | 3.02× |
| one StableSwap `getY` | 8,629 | below the estimate's resolution | — |

Plain 256-bit multiply-and-divide is **three times cheaper in Rust**, flat across three orders of
magnitude. The arithmetic is not the problem.

The problem is that there is so little of it. **One `getY` — twelve Newton iterations — is 8,629
gas.** Three times cheaper saves under 6,000, against a fixed cost of 19,724 to load the WASM
program. That is nearly the whole 25,305, and it says the curve is
nowhere near the ~62,000 gas of arithmetic the crossover needs. Twelve Newton iterations sounds like
a lot and is not: the bar is closer to 220 plain multiply-divides, or seven `getY` calls, per swap.

### A structural tax on Stylus hooks, and how to avoid it

Solidity hooks hold the pool manager in an `immutable`, which costs nothing to read. The Stylus SDK
has no equivalent, so a constructor argument can only go to storage and every callback pays a cold
`SLOAD` — 2,100 gas — purely to check that its caller is the pool manager.

There is a way out. A Rust `const` lives in the WASM code and costs nothing to read, and
[`stylus/native-stableswap/build.rs`](stylus/native-stableswap/build.rs) bakes the address in from
`$POOL_MANAGER` at build time:

```bash
POOL_MANAGER=0x360E68faCcca8cA495c1B759Fd9EEe466db9FB32 cargo stylus deploy ...
```

Measured, the hook goes from 176,519 gas per swap to **174,417** — 2,102 saved, which is the cold
`SLOAD` to the byte.

The trade is that the address is fixed at build time rather than deploy time. For a hook that costs
nothing: the address has to be mined against the init code anyway, so the contract is already
rebuilt per deployment. `native-counter` and `native-compute` still read theirs from storage, so
their figures elsewhere in this document carry the 2,100 — twice per swap in the counter's case,
which is called on both `beforeSwap` and `afterSwap`.

### The rule this all adds up to

Five measurements of the same question:

| operation | what it needs | Solidity | Rust | ratio |
| --- | --- | ---: | ---: | ---: |
| plain `a * b / c` | one `MUL`, one `DIV`, checked | 278 | 93 | 3.0× |
| `rpow` (Bunni's LDF) | `mulDiv`, Q96, fits in 256 bits | 3,058 | 1,850 | 1.65× |
| `mulDiv` | 512-bit intermediate | 694 | 247 | 2.8× |
| `sqrt` | 512-bit intermediate + bit length | 2,009 | 447 | 4.5× |
| xorshift64 | 64-bit words | 115 | 10.9 | 10.6× |

Rust is cheaper in every row, by between 1.65× and 10.6×. How much cheaper tracks how badly the work
fits a 256-bit word: `rpow` in Q96 gains least because that is exactly the shape the EVM is built
for, and 64-bit words gain most because the EVM pays for a word it cannot use.

But the ratio was never the binding constraint. **The amount of arithmetic is.** Against a fixed
cost of roughly 20,000 gas to enter a Stylus contract, a 3× saving needs about 30,000 gas of
Solidity arithmetic just to break even, and 62,000 to be worth the trouble. Nothing measured in this
document gets there: AntiSandwichHook has 21,000, a StableSwap curve 8,600, the counter and airdrop
hooks essentially none.

That is the finding. Not that Stylus computes slowly — it does not — but that **hooks do not compute
enough** for it to matter.

### Writing storage

There is no such thing as "Stylus storage" as distinct from "Solidity storage". A Stylus contract
writes the same 32-byte slots in the same account trie, and ArbOS charges EVM prices for reaching
them. This mode checks that rather than assuming it — each round writes one mapping slot.

| slots written | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 124,329 | 162,887 | +38,558 |
| 5 | 136,314 | 174,620 | +38,306 |
| 25 | 184,318 | 221,742 | +37,424 |
| 100 | 364,033 | 398,312 | +34,279 |

2,397 gas per slot in Solidity, 2,354 in Rust. Stylus is **1.8 % cheaper**, not 2.8× and not 10×,
and that margin is not the store itself: a cold slot costs 2,100 to touch and 100 to write the value
it already holds, identically in both. What Stylus shaves is the arithmetic wrapped around the
access — hashing the mapping key, and the loop. Paying off the entry fee on storage alone would take
about 900 writes per call.

So moving state into a Stylus contract does not make it cheaper to store. It makes the code around
it cheaper.

### The fixed cost

At zero rounds the Rust hook still costs ~38,500 gas more, of which 22,142 is loading its WASM
program on every call — 5,432 once the contract is cached. That figure is higher here than for
`native-counter` because this hook carries both workloads and `U512` arithmetic; the program is
bigger, and `ArbWasm` charges by compiled size.

## Pricing a real hook: AntiSandwichHook

`./bench-antisandwich.bash` measures OpenZeppelin's
[`AntiSandwichHook`](https://github.com/OpenZeppelin/uniswap-hooks/blob/master/src/general/AntiSandwichHook.sol),
the most compute-looking hook in the library the official v4-template depends on. It checkpoints the
pool at the top of each block and, for `zeroForOne == false` swaps, replays `Pool.swap` against that
checkpoint so a trade cannot get a better price than the block started with. `zeroForOne == true`
swaps skip the replay, so the gap between the two directions isolates the simulation.

| | gas per swap | over baseline |
| --- | ---: | ---: |
| no hook, 0→1 | 120,315 | — |
| no hook, 1→0 | 114,520 | — |
| AntiSandwichHook, 0→1 (no replay) | 168,425 | +48,110 |
| AntiSandwichHook, 1→0 (with replay) | 184,090 | +69,570 |
| **the `Pool.swap` replay alone** | | **21,460** |

So the arithmetic is **31 %** of what this hook costs. The other 48,110 is checkpointing pool state
into the hook's own storage — `extsload` calls to the pool manager, then `SSTORE`s — and settling
an ERC-6909 fee. None of that gets cheaper in Stylus.

Projecting the port from the rates measured above: the replay at 2.8× would drop from 21,460 to
about 7,700, saving ~13,800, against a Stylus entry fee of ~22,000 uncached or ~5,400 cached.
**Porting this hook loses money uncached and saves perhaps 4 % cached.** Not the demonstration it
looks like from the outside.

The bar set by the `mulDiv` sweep is 89 operations, about 62,000 gas of Solidity arithmetic per
call. AntiSandwichHook has ~21,000 — a third of it — and it is the *best* candidate in that
library.

Two caveats. The measured pool holds a single full-range position, so the replay crosses no ticks;
a pool with concentrated liquidity would walk further and the replay would grow. And Uniswap's own
snapshots put `swapSimulator_before_singleTick` at 28,803 against `multiTick` at 28,899, which
suggests tick crossing adds much less than one might hope.

### `block.number` does not mean what this hook thinks on Arbitrum

The hook would not run at all until it was fixed. It keys its checkpoint on `block.number`, and on
Arbitrum the `NUMBER` opcode returns an estimate of the **L1** block number, not the L2 one. On the
dev node used here it reports `0` while the L2 chain is at block 3:

```
solidity block.number      0
ArbSys.arbBlockNumber()    3
eth_blockNumber            3
```

Since the checkpoint starts at block 0, `_lastCheckpoint.blockNumber != currentBlock` is never true,
the checkpoint is never taken, and `Pool.swap` runs against an empty state — the first
`zeroForOne == false` swap reverts with `InvalidPrice()`.

On Arbitrum One the number does advance, so the hook runs; but the beginning-of-block price is then
frozen for an entire L1 block, spanning many L2 blocks, which is a much wider window than intended.
`_getBlockNumber` is `virtual` precisely so this can be fixed, and
[`ArbAntiSandwichMock`](uniswap/script/bench/ArbAntiSandwichMock.sol) overrides it onto
`ArbSys.arbBlockNumber()` — a one-line change, but one nothing in the hook tells you to make.

## Which shipping hooks are worth porting

`./profile-hooks.bash` answers that generically. It swaps through one pool per hook and traces the
transaction opcode by opcode, summing gas by class, then subtracts the same swap through a pool with
no hook. Only the compute column moves in Stylus.

A call opcode's `gasCost` in the trace is the gas handed to the callee, and the callee's own opcodes
are logged too, so calls are counted rather than summed — the numbers below are the work itself.

| | compute | storage | keccak | calls | total gas |
| --- | ---: | ---: | ---: | ---: | ---: |
| no hook | 26,656 | 48,100 | 684 | 11 | 114,608 |
| AntiSandwich | 50,272 | 86,500 | 1,350 | 19 | 181,405 |
| LimitOrder | 31,433 | 50,300 | 828 | 13 | 124,441 |
| PanopticOracle | 32,150 | 72,700 | 912 | 13 | 147,642 |

What each one adds over the hookless swap:

| hook | compute | storage | keccak | compute share |
| --- | ---: | ---: | ---: | ---: |
| **AntiSandwich** | **23,616** | 38,400 | 666 | 37 % |
| LimitOrder | 4,777 | 2,200 | 144 | 67 % |
| PanopticOracle | 5,494 | 24,600 | 228 | 18 % |

The profiler and the direction trick agree on AntiSandwich — 23,616 against 21,460 — which is the
main reason to trust either.

None of them clears the 62,000-gas bar. The best is at a third of it. And note the first row: a
plain v4 swap is itself 26,656 of compute against 48,100 of storage, so a hook would have to compute
more than twice what the AMM does to be worth moving.

Two caveats on this table. `LimitOrderHook` was profiled with no orders resting, so it shows its
framework cost and not a fill; its 67 % compute share is of a very small number. And the heaviest
hooks Uniswap publishes — `NativeBookHook` at 155k–221k, `ALFMultiplexer` at 235k, `DualPoolHook` at
506k in their own snapshots — each need vaults, ladders and makers configured before their expensive
path runs at all, so they are not in this table. `ALFMultiplexer` is the one worth setting up: it
runs a `SwapSimulator` pass per routing candidate, and a simulation is roughly an AntiSandwich
replay, so three or more candidates would clear the bar.

### Candidates, ranked by protocol rather than by library

`Uniswap/hooklist` indexes what is deployed, but not what is used. Cross-checking the names against
DefiLlama changes the picture:

| protocol | TVL | its hook | maths per swap |
| --- | ---: | --- | --- |
| **Euler** | **$802M** | EulerSwap — the AMM *is* the hook | curve inverted by the quadratic formula: a 255-bit `sqrt`, full-range `mulDiv` chains |
| Bunni V2 | $215k | BunniHook | Liquidity Density Function in Q96: repeated `rpow` |
| flaunch | $1.8M | Flaunch PositionManager | fee routing, little arithmetic |
| Clanker, Zora, launchpads | — | many | bonding curves, a handful of multiplications |

Bunni looked like the answer and is not: its TVL collapsed to $215k after a ~$2.3M exploit of
`BunniHub` in September 2025, and its Q96 arithmetic sits in the *worst* regime for Stylus at 1.65×.

EulerSwap is the better target on every axis. It is a real, used protocol; its AMM is the hook
rather than an accessory to one; and its curve inversion is full-range `sqrt` and `mulDiv`, the 4.5×
and 2.8× regimes. It is absent from the registry because it deploys one hook instance per pool
rather than a shared singleton.

Two caveats stand. EulerSwap is BUSL-1.1, so it is referenced here for information and its code is
not reproduced. And its compute *share* per swap is still unmeasured — the curve maths sits
alongside Euler vault calls and their storage, which do not move.

### Looking further: the hook registry

Uniswap keeps a public registry of deployed v4 hooks at
[`Uniswap/hooklist`](https://github.com/Uniswap/hooklist) — 1,472 entries at the time of writing,
each with its address, chain and all fourteen permission bits. That last part makes it searchable
for the profile that matters here: `beforeSwapReturnsDelta` means the hook prices the swap itself
rather than letting the pool do it, which is the flag a hook can only set if it is doing real math.

Filtering to Arbitrum, the chain Stylus runs on, leaves 33 hooks, 17 of which price their own swaps.
The ones whose descriptions and code size suggest heavy arithmetic:

| hook | address | deployed bytecode | what the registry says it does |
| --- | --- | ---: | --- |
| **BunniHook** | `0x0000fe59…1888` | 23.5 KB | computes all swap math internally from a configurable Liquidity Density Function, with TWAP-based and surge fees |
| **TokiHook** | `0x916bc355…1888` | 23.9 KB | custom-curve hook implementing a Pendle-style fixed-rate AMM |
| Spot | `0xb4f4949e…10cc` | 11.2 KB | full-range AMM with a truncated geometric-mean oracle driving dynamic fees |
| Clanker Dynamic Fee | `0xfd213be7…68cc` | — | fees adjusted from swap volatility via a tick accumulator |

Bytecode sizes are measured from Arbitrum One; the descriptions are the registry's own. Both
BunniHook and TokiHook are within a few hundred bytes of the 24 KB contract limit, which is itself a
signal — a hook that fits comfortably is not doing much.

BunniHook is the strongest candidate found anywhere, and its own repository backs that up. Bunni
publishes gas snapshots: a swap through it costs **437,000–505,000 gas**, against roughly 115,000
for a swap with no hook. So the hook adds 320,000–390,000 gas per swap — five times what
AntiSandwichHook adds, and far into the range where arithmetic could plausibly dominate.

Its swap path runs `rpow` repeatedly, which is why that operation is benchmarked above. But the
answer that comes back is sobering: `rpow` at Q96 precision is only **1.65× cheaper** in Rust,
because Q96 products fit in 256 bits and take `FullMath.mulDiv`'s single-`DIV` fast path.

What is still missing is Bunni's compute share. Its 320,000–390,000 gas per swap mixes LDF
arithmetic with vault accounting, TWAP observation writes and rebalance bookkeeping, and only the
first of those moves. At 1.65×, a hook whose cost were 40 % arithmetic would save about 6 % overall
after the Stylus entry fee; at 70 % arithmetic, about 15 %. Worth having, not the headline.

Measuring that share needs a live Bunni pool, and pool discovery on Arbitrum turned out to be the
hard part: the hub is an `internal immutable` with no getter, so it cannot simply be read off the
hook.

## What this means for the project## Pricing a real hook: AntiSandwichHook

`./bench-antisandwich.bash` measures OpenZeppelin's
[`AntiSandwichHook`](https://github.com/OpenZeppelin/uniswap-hooks/blob/master/src/general/AntiSandwichHook.sol),
the most compute-looking hook in the library the official v4-template depends on. It checkpoints the
pool at the top of each block and, for `zeroForOne == false` swaps, replays `Pool.swap` against that
checkpoint so a trade cannot get a better price than the block started with. `zeroForOne == true`
swaps skip the replay, so the gap between the two directions isolates the simulation.

| | gas per swap | over baseline |
| --- | ---: | ---: |
| no hook, 0→1 | 120,315 | — |
| no hook, 1→0 | 114,520 | — |
| AntiSandwichHook, 0→1 (no replay) | 168,425 | +48,110 |
| AntiSandwichHook, 1→0 (with replay) | 184,090 | +69,570 |
| **the `Pool.swap` replay alone** | | **21,460** |

So the arithmetic is **31 %** of what this hook costs. The other 48,110 is checkpointing pool state
into the hook's own storage — `extsload` calls to the pool manager, then `SSTORE`s — and settling
an ERC-6909 fee. None of that gets cheaper in Stylus.

Projecting the port from the rates measured above: the replay at 2.8× would drop from 21,460 to
about 7,700, saving ~13,800, against a Stylus entry fee of ~22,000 uncached or ~5,400 cached.
**Porting this hook loses money uncached and saves perhaps 4 % cached.** Not the demonstration it
looks like from the outside.

The bar set by the `mulDiv` sweep is 89 operations, about 62,000 gas of Solidity arithmetic per
call. AntiSandwichHook has ~21,000 — a third of it — and it is the *best* candidate in that
library.

Two caveats. The measured pool holds a single full-range position, so the replay crosses no ticks;
a pool with concentrated liquidity would walk further and the replay would grow. And Uniswap's own
snapshots put `swapSimulator_before_singleTick` at 28,803 against `multiTick` at 28,899, which
suggests tick crossing adds much less than one might hope.

### `block.number` does not mean what this hook thinks on Arbitrum

The hook would not run at all until it was fixed. It keys its checkpoint on `block.number`, and on
Arbitrum the `NUMBER` opcode returns an estimate of the **L1** block number, not the L2 one. On the
dev node used here it reports `0` while the L2 chain is at block 3:

```
solidity block.number      0
ArbSys.arbBlockNumber()    3
eth_blockNumber            3
```

Since the checkpoint starts at block 0, `_lastCheckpoint.blockNumber != currentBlock` is never true,
the checkpoint is never taken, and `Pool.swap` runs against an empty state — the first
`zeroForOne == false` swap reverts with `InvalidPrice()`.

On Arbitrum One the number does advance, so the hook runs; but the beginning-of-block price is then
frozen for an entire L1 block, spanning many L2 blocks, which is a much wider window than intended.
`_getBlockNumber` is `virtual` precisely so this can be fixed, and
[`ArbAntiSandwichMock`](uniswap/script/bench/ArbAntiSandwichMock.sol) overrides it onto
`ArbSys.arbBlockNumber()` — a one-line change, but one nothing in the hook tells you to make.

## Which shipping hooks are worth porting

`./profile-hooks.bash` answers that generically. It swaps through one pool per hook and traces the
transaction opcode by opcode, summing gas by class, then subtracts the same swap through a pool with
no hook. Only the compute column moves in Stylus.

A call opcode's `gasCost` in the trace is the gas handed to the callee, and the callee's own opcodes
are logged too, so calls are counted rather than summed — the numbers below are the work itself.

| | compute | storage | keccak | calls | total gas |
| --- | ---: | ---: | ---: | ---: | ---: |
| no hook | 26,656 | 48,100 | 684 | 11 | 114,608 |
| AntiSandwich | 50,272 | 86,500 | 1,350 | 19 | 181,405 |
| LimitOrder | 31,433 | 50,300 | 828 | 13 | 124,441 |
| PanopticOracle | 32,150 | 72,700 | 912 | 13 | 147,642 |

What each one adds over the hookless swap:

| hook | compute | storage | keccak | compute share |
| --- | ---: | ---: | ---: | ---: |
| **AntiSandwich** | **23,616** | 38,400 | 666 | 37 % |
| LimitOrder | 4,777 | 2,200 | 144 | 67 % |
| PanopticOracle | 5,494 | 24,600 | 228 | 18 % |

The profiler and the direction trick agree on AntiSandwich — 23,616 against 21,460 — which is the
main reason to trust either.

None of them clears the 62,000-gas bar. The best is at a third of it. And note the first row: a
plain v4 swap is itself 26,656 of compute against 48,100 of storage, so a hook would have to compute
more than twice what the AMM does to be worth moving.

Two caveats on this table. `LimitOrderHook` was profiled with no orders resting, so it shows its
framework cost and not a fill; its 67 % compute share is of a very small number. And the heaviest
hooks Uniswap publishes — `NativeBookHook` at 155k–221k, `ALFMultiplexer` at 235k, `DualPoolHook` at
506k in their own snapshots — each need vaults, ladders and makers configured before their expensive
path runs at all, so they are not in this table. `ALFMultiplexer` is the one worth setting up: it
runs a `SwapSimulator` pass per routing candidate, and a simulation is roughly an AntiSandwich
replay, so three or more candidates would clear the bar.

### Candidates, ranked by protocol rather than by library

`Uniswap/hooklist` indexes what is deployed, but not what is used. Cross-checking the names against
DefiLlama changes the picture:

| protocol | TVL | its hook | maths per swap |
| --- | ---: | --- | --- |
| **Euler** | **$802M** | EulerSwap — the AMM *is* the hook | curve inverted by the quadratic formula: a 255-bit `sqrt`, full-range `mulDiv` chains |
| Bunni V2 | $215k | BunniHook | Liquidity Density Function in Q96: repeated `rpow` |
| flaunch | $1.8M | Flaunch PositionManager | fee routing, little arithmetic |
| Clanker, Zora, launchpads | — | many | bonding curves, a handful of multiplications |

Bunni looked like the answer and is not: its TVL collapsed to $215k after a ~$2.3M exploit of
`BunniHub` in September 2025, and its Q96 arithmetic sits in the *worst* regime for Stylus at 1.65×.

EulerSwap is the better target on every axis. It is a real, used protocol; its AMM is the hook
rather than an accessory to one; and its curve inversion is full-range `sqrt` and `mulDiv`, the 4.5×
and 2.8× regimes. It is absent from the registry because it deploys one hook instance per pool
rather than a shared singleton.

Two caveats stand. EulerSwap is BUSL-1.1, so it is referenced here for information and its code is
not reproduced. And its compute *share* per swap is still unmeasured — the curve maths sits
alongside Euler vault calls and their storage, which do not move.

### Looking further: the hook registry

Uniswap keeps a public registry of deployed v4 hooks at
[`Uniswap/hooklist`](https://github.com/Uniswap/hooklist) — 1,472 entries at the time of writing,
each with its address, chain and all fourteen permission bits. That last part makes it searchable
for the profile that matters here: `beforeSwapReturnsDelta` means the hook prices the swap itself
rather than letting the pool do it, which is the flag a hook can only set if it is doing real math.

Filtering to Arbitrum, the chain Stylus runs on, leaves 33 hooks, 17 of which price their own swaps.
The ones whose descriptions and code size suggest heavy arithmetic:

| hook | address | deployed bytecode | what the registry says it does |
| --- | --- | ---: | --- |
| **BunniHook** | `0x0000fe59…1888` | 23.5 KB | computes all swap math internally from a configurable Liquidity Density Function, with TWAP-based and surge fees |
| **TokiHook** | `0x916bc355…1888` | 23.9 KB | custom-curve hook implementing a Pendle-style fixed-rate AMM |
| Spot | `0xb4f4949e…10cc` | 11.2 KB | full-range AMM with a truncated geometric-mean oracle driving dynamic fees |
| Clanker Dynamic Fee | `0xfd213be7…68cc` | — | fees adjusted from swap volatility via a tick accumulator |

Bytecode sizes are measured from Arbitrum One; the descriptions are the registry's own. Both
BunniHook and TokiHook are within a few hundred bytes of the 24 KB contract limit, which is itself a
signal — a hook that fits comfortably is not doing much.

BunniHook is the strongest candidate found anywhere, and its own repository backs that up. Bunni
publishes gas snapshots: a swap through it costs **437,000–505,000 gas**, against roughly 115,000
for a swap with no hook. So the hook adds 320,000–390,000 gas per swap — five times what
AntiSandwichHook adds, and far into the range where arithmetic could plausibly dominate.

Its swap path runs `rpow` repeatedly, which is why that operation is benchmarked above. But the
answer that comes back is sobering: `rpow` at Q96 precision is only **1.65× cheaper** in Rust,
because Q96 products fit in 256 bits and take `FullMath.mulDiv`'s single-`DIV` fast path.

What is still missing is Bunni's compute share. Its 320,000–390,000 gas per swap mixes LDF
arithmetic with vault accounting, TWAP observation writes and rebalance bookkeeping, and only the
first of those moves. At 1.65×, a hook whose cost were 40 % arithmetic would save about 6 % overall
after the Stylus entry fee; at 70 % arithmetic, about 15 %. Worth having, not the headline.

Measuring that share needs a live Bunni pool, and pool discovery on Arbitrum turned out to be the
hard part: the hub is an `internal immutable` with no getter, so it cannot simply be read off the
hook.

## What this means for the project

The airdrop hook was the wrong thing to port. It is a bookkeeping hook: read a counter, add to it,
write it back. That is the exact shape of hook where Stylus has nothing to offer.

Stylus pays off when a hook has to *think* on every swap — past the crossover measured above, which
is ~371 rounds of small-word arithmetic or 89 `mulDiv`s. In practice:

- on-chain math: TWAP and volatility oracles, curve solvers, Newton iterations
- dynamic fees computed from a model rather than looked up
- anything scanning or sorting a batch, where the loop dominates
- signature or proof verification inside the callback

Those are the hooks worth writing in Rust, and they are what
[`stylus/base-hook`](stylus/base-hook) exists to make writable end-to-end.

## A caveat on the numbers

Gas here is L2 execution gas only. The benchmark sets the L1 data-posting price to zero
(`ArbOwner.setL1PricePerUnit(0)`) because that component is identical across all four pools — same
calldata, same swap — and on a real chain it dwarfs the differences being measured.

[`uniswap/test/Gas.t.sol`](uniswap/test/Gas.t.sol) runs the same comparison inside `forge test`, but
forge cannot execute WASM, so its "stylus" row is really the `replica` row. Its numbers
(33,545 / 39,126 hook cost measured on-chain vs 31,732 / 37,309 in forge) track each other within
about 5 % for the Solidity variants, which is what makes it useful as a fast check.
