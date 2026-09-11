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

# What the hooks cost per swap

## The counter: the floor

`./bench-counter.bash` runs the same hook two ways, one swap per transaction. A swap hits this hook
twice — `beforeSwap` and `afterSwap` — so every figure is two hook calls.

| | gas per swap | hook costs |
| --- | ---: | ---: |
| no hook | 115,065 | — |
| `Counter.sol`, one Solidity contract | 134,003 | +18,938 |
| `native-counter`, uncached | 199,595 | +84,530 |
| **`native-counter`, cached** | **168,941** | **+53,876** |

**The native hook costs 34,938 gas more per swap than the Solidity one**, and that is this document's
conclusion stated at its worst: the hook writes a storage slot per callback and computes nothing, so
there is nothing for Stylus to win back. Every figure below is read against this one.

The gap between the two native rows is the whole reason the rest of this document reports cached
figures. `ArbWasm` prices `programInitGas` off compiled size, the hook is entered twice per swap, and
caching takes that from 17,482 to 2,187 per entry:

| | asm size | init gas, uncached / cached |
| --- | ---: | ---: |
| `native-counter` | 857,088 | 17,482 / 2,187 |

2 × (17,482 − 2,187) = 30,590, against the 30,654 the swap actually dropped by. Caching is a one-off
bid and an uncached program pays the full WASM load on every call forever, so the cached row is the
state any hook with users would be in — and on a dev node with no `CacheManager`, the chain owner
appoints itself one so the figure is measured rather than quoted out of `programInitGas`.

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
| **StableSwap in Rust** | **174,656** | **+59,595** |

**Rust loses by 25,544 gas.** So the next question is where that goes, and the answer is not where
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
program. That is nearly the whole 25,544, and it says the curve is
nowhere near the ~62,000 gas of arithmetic the crossover needs. Twelve Newton iterations sounds like
a lot and is not: the bar is closer to 220 plain multiply-divides, or seven `getY` calls, per swap.

### A structural tax on Stylus hooks, and how to avoid it

Solidity hooks hold the pool manager in an `immutable`, which costs nothing to read. The Stylus SDK
has no equivalent, so a constructor argument can only go to storage and every callback pays a cold
`SLOAD` — 2,100 gas — purely to check that its caller is the pool manager.

There is a way out, and all four Rust hooks here now take it. A Rust `const` lives in the WASM code
and costs nothing to read, and each hook's `build.rs` bakes the address in from `$POOL_MANAGER` at
build time:

```bash
POOL_MANAGER=0x360E68faCcca8cA495c1B759Fd9EEe466db9FB32 cargo stylus deploy ...
```

Measured on StableSwap, the hook goes from 176,519 gas per swap to **174,417** — 2,102 saved, which
is the cold `SLOAD` to the byte. On the counter, which gets two callbacks per swap and so paid a cold
`SLOAD` and a warm one, 202,163 to **199,595** — 2,568.

Two things a storage slot cannot match. A `static` is not an alternative: it compiles, and a Stylus
program is instantiated per call, so anything written to one is silently gone by the next call —
measured, writing 42 and reading back the initial 777. And the constant cannot be changed after
deployment at all, because it is part of the code whose hash the mined CREATE2 address commits to; a
different pool manager is a different hook address. Solidity's `immutable` is only guaranteed by the
deploy transaction.

A constant has a failure mode a storage slot does not: build against the wrong `$POOL_MANAGER` and
you get a hook that compiles, deploys, mines a valid address and then silently rejects every call
the pool manager makes. So the constructor still takes the address, purely to compare it with what
was compiled in and revert if they differ. The argument is never stored; callbacks still read the
constant. That check costs **239 gas** per swap — not the constructor, which runs once, but the
slightly larger program, which `ArbWasm` prices by size. Cheap for turning a silent
misconfiguration into a failed deployment.

The remaining trade is that the address is fixed at build time rather than deploy time, which costs
nothing here: the address has to be mined against the init code anyway, so the contract is already
rebuilt per deployment. `native-counter` and `native-compute` still read theirs from storage, so
their figures elsewhere in this document carry the 2,100 — twice per swap in the counter's case,
which is called on both `beforeSwap` and `afterSwap`.

### The rule this all adds up to

Five measurements of the same question, all remeasured with the contracts built for speed (see
*The flag that changes every number in this document*, below — the first pass of this table was
compiled for size and every ratio in it was roughly half of what follows):

| operation | what it needs | Solidity | Rust | ratio |
| --- | --- | ---: | ---: | ---: |
| plain `a * b / c` | one `MUL`, one `DIV`, checked | 278 | 51 | **5.5×** |
| `rpow` (Bunni's LDF) | `mulDiv`, Q96, fits in 256 bits | 3,058 | 1,019 | **3.0×** |
| `mulDiv` | 512-bit intermediate | 694 | 204 | **3.4×** |
| `sqrt` | 512-bit intermediate + bit length | 2,009 | 362 | **5.6×** |
| xorshift64 | 64-bit words | 115 | 2.8 | **41.6×** |
| storage write | a host operation either way | 2,397 | 2,265 | 1.06× |

Rust is cheaper in every row, by between 3.0× and 41.6×, and storage — a host operation on both
sides — is the row that does not move. How much cheaper tracks how badly the work fits a 256-bit
word: `rpow` in Q96 gains least because that is exactly the shape the EVM is built for, and 64-bit
words gain most because the EVM pays for a word it cannot use.

The width effect is worth measuring directly. The same expression, 100 times, changing only how many
of a `U256`'s four limbs are non-zero:

| 100 × `a*b/c` | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| operands ~2^32 (one limb) | 9,599 | 2,285 | **4.20×** |
| operands ~2^96 (two limbs) | 9,599 | 2,825 | **3.40×** |
| operands ~2^128 (four limbs) | 9,599 | 4,083 | **2.35×** |

**The EVM's column is flat: `MUL` and `DIV` cost 5 gas whatever the operands are.** A `U256` in WASM
is four 64-bit limbs and `ruint` only pays for the ones that are non-zero, so Rust's advantage
narrows as the words fill up — but it does not run out. A hook lives in Q96 and WAD fixed point,
which is the middle row.

Even so, the ratio was never the binding constraint. **The amount of arithmetic is.** Against the
fixed cost of entering a Stylus contract — 7,916 gas for a cached hook, measured against the
production TWAMM below — a 3× saving needs about 12,000 gas of Solidity arithmetic to break even.
AntiSandwichHook has 21,000, a StableSwap curve 8,600, the counter hook essentially
none, and the pm-AMM's Gaussian solve has 62,000.

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
Arbitrum the `NUMBER` opcode does not return the block number — it returns the **L1** block number.
Measured on Arbitrum One itself, not on a fork (a fork replays Arbitrum's state through a vanilla
EVM and would not reproduce this):

| | |
| --- | ---: |
| Solidity `block.number`, via `Multicall3.getBlockNumber()` | 25,912,325 |
| Ethereum L1 height, read at the same moment | 25,912,326 |
| `NodeInterface.blockL1Num(502057926)` | 25,912,325 |
| Arbitrum L2 block, `eth_blockNumber` | 502,057,928 |

Two consequences, one per environment.

On a dev node there is no L1, so `block.number` is `0` while the L2 chain runs. The checkpoint also
starts at 0, `_lastCheckpoint.blockNumber != currentBlock` is never true, the checkpoint is never
taken, and `Pool.swap` runs against an empty state — the first `zeroForOne == false` swap reverts
with `InvalidPrice()`. That is what `bench-antisandwich.bash` hit.

On Arbitrum One the number does advance, so the hook runs, but it advances once per **L1** block.
Arbitrum produces L2 blocks roughly every 250 ms, so the "beginning-of-block" reference price is
held for about forty-eight L2 blocks rather than one. The protection is not absent — it is applied
over a twelve-second window, during which honest price movement is also refused. That is a different
economic instrument from the one the hook describes.

`_getBlockNumber` is `virtual` precisely so this can be fixed, and
[`ArbAntiSandwichMock`](uniswap/script/bench/ArbAntiSandwichMock.sol) overrides it onto
`ArbSys.arbBlockNumber()`. A one-line change, but nothing in the hook tells you to make it.

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

EulerSwap is BUSL-1.1, so it is referenced here for information and its code is not reproduced.

### Measuring it

Two EulerSwap v2 pools are live on Arbitrum. The pool exposes `computeQuote`, `getLimits` and
`getReserves` separately, and Arbitrum's `NodeInterface.gasEstimateComponents` splits an estimate
into its L2 and L1 halves, so the swap path can be taken apart on the real chain without a trace:

| | L2 gas |
| --- | ---: |
| `getStaticParams()` — a `pure` call, the floor | 25,865 |
| `getReserves()` | 26,819 |
| **`getLimits()` — queries the Euler vaults** | **153,526** |
| `computeQuote()` — the curve plus those limits | 156,773 |
| `quoteExactInput()` — the whole path | 162,603 |

`computeQuote` is `getLimits` plus about 3,200 gas. **The curve — the quadratic-formula inversion
with its 255-bit square root — costs roughly 3,200 gas. The vault queries cost 127,700.**

A second measurement says the same thing independently: the cost barely moves with trade size.
Quoting 1,000 units costs 156,773 and quoting a thousand times more costs 156,825, a difference of
52 gas. An iterative curve solve that were doing real work would not be that flat.

So the protocol that looked most compute-heavy in the whole registry — its AMM *is* the hook, and it
inverts its curve with a square root — spends about **2 % of its gas on arithmetic** and the rest
talking to lending vaults. At the 4.5× measured for `sqrt`, porting the curve would save around
2,300 gas against a ~20,000 gas entry fee.

That is the search concluded. Every candidate, from a counter to a live $800M protocol, lands in the
same place.

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

## Against the official guidance

Arbitrum publishes [gas optimization best practices](https://docs.arbitrum.io/stylus/best-practices/gas-optimization)
for Stylus. It says up front that its multipliers are directional and that you should benchmark your
own contract, which is what this document is. Four of its claims are checkable against the
measurements here, and they do not all hold.

| the docs say | measured here |
| --- | --- |
| compute-heavy loops: **~50–100×** | **10.6×** at best, on 64-bit xorshift. 256-bit work runs 1.65× to 4.5×. |
| storage operations: **none (1×)** | 1.8 % cheaper. Agrees, for every practical purpose. |
| set `opt-level = "z"` for smaller binaries | makes them **bigger**: 18,555 → 18,928 bytes, because the SDK's own pinned `wasm-opt -Oz` already runs afterwards. `"s"` wins. |
| `ecrecover`: 3,000 gas → **~300 gas, ~10×** | no mechanism for this is visible. `stylus_sdk::crypto` exposes exactly one primitive, `native_keccak256`; there is no signature hostio. A Stylus contract can only call the `0x01` precompile at its EVM price, or implement secp256k1 in Rust, and neither lands near 300. |

The 50–100× figure is the one that matters most, because it is the number a team would use to decide
whether to port. Nothing measured here — five kinds of arithmetic, across four orders of magnitude of
loop length, on a current dev node — comes within a factor of five of it. If it is reachable, it is
on a workload shape this document did not find, and the docs do not say which.

Two pieces of the guidance the work here follows independently: cache storage reads rather than
re-reading in a loop, and measure on a live endpoint because `TestVM` has no gas meter.

## The candidate: Uniswap's own TWAMM

The search above looked at what is *deployed*. It missed what Uniswap *published*. Their v4-periphery
carried a set of example hooks — `TWAMM`, `FullRange`, `GeomeanOracle`, `LimitOrder`,
`VolatilityOracle` — removed from the tree in December 2024 but still in its history, together with
the gas snapshots their own tests recorded:

| | gas |
| --- | ---: |
| `executTWAMMOrders`, 1 interval | 489,927 |
| 2 intervals | 595,404 |
| 3 intervals | 692,853 |
| `executTWAMMOrders singleSell`, 1 interval | 262,033 |
| 2 intervals | 294,134 |
| `FullRangeSwap`, an ordinary swap for scale | 81,970 |

An extra interval costs about **100,000 gas**, and that is marginal cost — the incremental work of one
more iteration, with the fixed overhead already paid.

What that hundred thousand buys is arithmetic. `TwammMath` runs on `ABDKMathQuad`: **IEEE 754
quadruple-precision floating point, emulated in Solidity over `bytes16`**. One pass performs 20
multiplies, 19 divides, 21 conversions, 7 square roots, 4 additions, 8 subtractions — and **two
exponentials and a logarithm**. The EVM has no floating point at all, no `exp`, no `ln`, and no
instruction for the bit-scan every normalisation needs.

**This is the first workload in this document that clears the 62,000-gas bar**, and it clears it by
60 % on marginal cost alone. It is also, by the rule the five sweeps establish, the profile where
Stylus should gain most rather than least: quad floats are built from 64-bit limbs, which is WASM's
native word and the 10.6× regime, and their normalisation needs a bit-scan, which is `i64.clz` and
the 4.5× regime.

That it was never deployed is the argument, not a counterargument. TWAMM is a well-known design that
Uniswap wrote, benchmarked, and shipped as an example — and half a million gas per execution is why
nobody runs one. It is precisely the hook that is too expensive to exist in Solidity.

### Why the port cannot be written

Stylus has no floating point. A contract that so much as converts an integer to an `f64` is refused
at activation:

```
program activation failed: failed to build user module
No implementation for floating point operation ConvertIntOp(F64, I64, false) in user
```

The SDK's own README says the same: "we may add … floating point and SIMD, which the Stylus VM does
not yet support". Stylus is at version 3 on both Arbitrum One and Sepolia.

That takes the argument apart. TWAMM is expensive because ABDK emulates IEEE 754 binary128 in
software over 256-bit words. The reason that looked like Stylus's best case was the assumption that
WASM would run those floats natively — and it will not, because Stylus forbids them. Neither
machine has floating point.

So a Rust TWAMM has two options, and neither is the experiment it appeared to be. Emulate binary128
in Rust as well, which measures one software float implementation against another rather than one
language against another. Or drop floats for integer fixed point — but a Solidity TWAMM could do
that too, and would get most of the same saving. That is an algorithmic change wearing a language
change's clothes.

So the port was written in fixed point, and — because that would otherwise compare an algorithm
rather than a language — `TwammHook.sol` carries the identical fixed-point form alongside the
quad-float one. All three agree: the two fixed-point implementations are identical to the wei, and
both track the quad-float original to two parts in 10^18. The quadruple precision was buying
nothing.

`./bench-twamm.bash`, gas for the arithmetic alone, called directly with no hook and no storage:

| intervals | Solidity, quad floats | Solidity, fixed point | Rust, fixed point |
| ---: | ---: | ---: | ---: |
| 1 | 23,587 | 14,011 | **2,368** |
| 2 | 47,521 | 27,842 | **4,723** |
| 4 | 94,458 | 55,950 | **9,435** |
| 8 | 189,590 | 111,385 | **18,860** |

Per interval: **23,715 gas in Solidity as written, 13,911 in Solidity done differently, 2,356 in
Rust.**

That splits cleanly into the two changes it is made of:

- **1.7× from the algorithm.** Dropping ABDK's software binary128 for fixed point is worth that much
  without leaving Solidity at all, and it costs two parts in 10^18 of precision.
- **6.0× from the language.** That is the largest gain measured anywhere in this document on
  arithmetic that a real hook actually runs — larger than `sqrt` at 4.5×, and approaching the 10.6×
  of the synthetic 64-bit loop.

Together, 10.1×. The Rust side is a complete hook rather than a kernel — orders, order pools,
expiries, settlement — so it is 37 KB and pays 30,289 gas to be loaded uncached, or 5,025 once
cached. Against 11,555 saved per interval that is **2.6 intervals to break even cold, and under
half an interval warm**; against the quad-float form Uniswap actually ships, 1.4 intervals cold.

### Is there a Solidity hook that is genuinely expensive? Yes, and it is a trap

Measured by running each project's own test suite under `--gas-report`, rather than estimated.

| hook | per swap | how it was obtained |
| --- | ---: | --- |
| [`robertleifke/forex-swap`][fx] | **4,266,005 avg, 11,283,644 max** | its own `ForexSwap.t.sol` |
| [`Gnome101/Pm-AMM-Hook`][pm] | ~401,100 of arithmetic | 100 bisection steps × measured `cdf`+`pdf` |
| [`akshatmittal/v4-twamm-hook`][tw] | 339,609 (one span) | `bench-twamm.bash` |
| `AntiSandwichHook` (OpenZeppelin) | 168,429 | `bench-antisandwich.bash` |
| baseline swap, no hook | 115,065 | every benchmark here |

[fx]: https://github.com/robertleifke/forex-swap
[pm]: https://github.com/Gnome101/Pm-AMM-Hook
[tw]: https://github.com/akshatmittal/v4-twamm-hook

`forex-swap` is the most expensive v4 hook I have found by an order of magnitude: a single swap eats
4.3 million gas on average and up to 11.3 million, which is a third of an Ethereum block. Its own
repository carries a runtime-baseline file with a guardrail that says *"stop and isolate pathological
path"*, which is a fair description.

**And it is exactly the wrong thing to port.** Its `Gaussian.cdf` computes the CDF by *bisecting on
its own inverse CDF*, calling `ppf` `CDF_STEPS` times, each of which calls `_ppfRaw` three times.
Measured, that costs **1,280,509 gas for one CDF evaluation**. `solstat`'s `erfc`-based CDF, which is
the same function to the same precision, costs **3,165** — the algorithm choice is a **405× penalty**,
and three CDF evaluations account for about 90 % of a swap.

So the honest reading: yes, expensive Solidity hooks exist. No, this one is not an argument for
Stylus, because rewriting its CDF in Solidity would recover roughly 4 million of the 4.3 million and
a Stylus port would then be worth the 1.42×–2.47× measured above. Benchmarking against it would
repeat the mistake this document already made once with TWAMM: **the gain would be the algorithm
wearing a language's clothes.**

The pm-AMM is the one that survives the test, because its 401,100 gas is a *lazy* implementation of
arithmetic that is irreducible — fixing the laziness (Newton instead of 100-step bisection) still
leaves 27,659 gas of Gaussian solve that no rewriting removes, and that is the figure the benchmark
uses.

### Uniswap's own published hooks

[`Uniswap/v4-hooks-public`][pub] is where the canonical `base/BaseHook.sol` lives — the contract the
Stylus port targets — and it collects eleven hooks. Surveyed for arithmetic:

| | what it computes per swap |
| --- | --- |
| `stable/StablePairHook.sol` | **a dynamic fee, not a stable curve.** A price ratio against a reference, a boundary interpolation, and a `fastPow` for per-block fee decay. Well below the bar. |
| `aggregator-hooks/*` (10 of them, two of which wrap Curve StableSwap) | nothing — the curve is an external call, `pool.get_dy(...)`. The same I/O-bound shape as EulerSwap. |
| `alf/*` | the heaviest by far. `SwapSimulator.simulateSwapToPrice` replays v4's own tick-crossing swap loop on chain, and `NativeBookHook` and `ALFMultiplexer` walk bins and ladders per swap. |

Only the last is a candidate, and it is the pattern already priced in this document: replaying
`Pool.swap` is what `AntiSandwichHook` does, measured at **21,460 gas** of `mulDiv` and `sqrt` — above
the bar, and the part of that hook worth porting.

Worth stating plainly because it is easy to assume otherwise: **the StableSwap hook benchmarked in
this document is not one of these.** `StableSwapHook.sol` was written here, from Curve's published
formula, specifically to be arithmetic-heavy enough to clear the bar — and it still lost.

[pub]: https://github.com/Uniswap/v4-hooks-public

### Where to look next: the two filters that rule out most cryptography

The obvious next thought is that a hook doing cryptography — a ZK verifier, a hash, a signature check
— must be the ideal Stylus workload, because that is expensive in Solidity. Two filters cut most of
that down, and both are measured rather than assumed.

**Filter one: precompiles.** A precompile is native code; Stylus can call it but never beat it.
Checked against Arbitrum One by return length:

| | on Arbitrum One | consequence |
| --- | --- | --- |
| `ecrecover`, SHA-256, RIPEMD, MODEXP, BLAKE2F | present | out |
| BN254 add / mul / pairing (`0x06`–`0x08`) | **present** | Groth16 and PLONK over BN254: out |
| BLS12-381, EIP-2537 (`0x0b`–`0x11`) | **present** | BLS signature verification: out |
| KZG point evaluation (`0x0a`) | absent | but it is BLS pairings, which are not |
| **P-256 / secp256r1, RIP-7212 (`0x100`)** | **absent** | open |

That BLS12-381 is live is worth knowing on its own: aggregate-signature hooks are a popular idea and
the EVM already does them natively here.

**Filter two: `MULMOD`.** The EVM has modular multiplication as a single opcode at 8 gas. WASM has
nothing of the sort. Measured, gas per modular multiplication:

| field | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| BN254 scalar (~254-bit), `ruint::mul_mod` | 88 | 94 | **0.93×** |
| BN254 scalar (~254-bit), Montgomery (CIOS) | 88 | **62** | **1.42×** |
| Goldilocks (64-bit) | 89 | **19** | **4.50×** |

Two things fall out. The naive answer — a 512-bit product and a division — *loses to the opcode*, so
how the field is implemented decides the result, not the language. And Solidity's cost is the same in
both fields, 88 against 89, because the EVM pays for a 256-bit word whether the field needs one or
not; Rust goes from 62 to 19.

So the thresholds, against the 7,916 gas a cached Stylus hook carries:

| field | saving per op | ops per swap to break even |
| --- | ---: | ---: |
| BN254 in Montgomery form | 26 | **304** |
| Goldilocks / BabyBear | 70 | **113** |

Both are low. One MiMC permutation is on the order of 660 field multiplications; a Poseidon2
permutation over Goldilocks is 8 full and 22 partial rounds; a P-256 verification is a few thousand.
Anything in that territory clears the bar several times over.

**And the primitives are already written.** `openzeppelin-crypto` 0.3.0, OpenZeppelin's Stylus crypto
crate, ships a Montgomery prime field plus Poseidon2 instanced over BN256, BLS12, Pallas, Vesta and —
the interesting ones — **Goldilocks and BabyBear**; twisted-Edwards curves including curve25519,
Jubjub, Baby Jubjub and Bandersnatch; Pedersen hashing; and a full **Ed25519** implementation with
signature verification. None of those has an EVM precompile.

Ranked by what the measurements above imply, not by anything measured end to end yet:

1. **Ed25519 verification.** No precompile on any EVM, so Solidity implementations run to hundreds of
   thousands of gas. `openzeppelin_crypto::eddsa` is already there.
2. **Poseidon2 or a STARK verifier over Goldilocks or BabyBear.** The 4.50× field, and verifiers do
   field operations by the hundred thousand.
3. **A Tornado-style privacy hook.** `ChinmayGopal931/UniStorm` is one, and its
   `MerkleTreeWithHistory` runs `MiMCSponge` on chain — thousands of BN254 multiplications per
   deposit, in the 1.42× field.
4. **P-256 / WebAuthn.** Not precompiled on Arbitrum, a few thousand multiplications per check.

What is *not* worth porting, and would have looked like the best idea: a Groth16 or PLONK verifier
over BN254. The pairing is a precompile and the field layer only gains 1.42×.

### The flag that changes every number in this document

Every benchmark here was, for months, measuring code compiled for size rather than speed — and not
by choice.

`opt-level = 2` or `3` produces a wasm ArbOS refuses to activate: *"program activation failed: failed
to parse wasm — unsupported section type DataCountSection"*. rustc emits `memory.copy` at the speed
levels, that brings a `DataCount` section, and the ArbOS prover does not support it. Nothing warns
you: the build succeeds, `cargo stylus check` succeeds, and the failure appears only at activation.
So the default a Stylus developer can actually reach is `"s"` or `"z"`.

Adding `--llvm-memory-copy-fill-lowering` to the wasm-opt flags lowers those bulk-memory ops back to
loops, drops the section, and makes the speed levels activatable, for a couple of hundred extra
bytes. Every crate's `Stylus.toml` here now passes it and the workspace builds at `opt-level = 3`.

What it is worth, by workload:

| | built for size | built for speed | change |
| --- | ---: | ---: | ---: |
| xorshift64, 64-bit | 10.6× | **41.6×** | ×3.9 |
| `a*b/c`, 256-bit | 3.0× | **5.5×** | ×1.8 |
| `rpow` Q96 | 1.65× | **3.0×** | ×1.8 |
| pm-AMM Gaussian solve | 1.13× | **2.26×** | ×2.0 |
| TWAMM, per expiry crossed | 1.44× | **1.47×** | ×1.02 |
| storage write | 1.02× | **1.06×** | ×1.04 |

**It buys arithmetic and nothing else**, in proportion to how much of a workload is arithmetic. The
two storage-bound rows do not move. That is a useful sanity check on the whole document: the
improvement lands exactly where the theory says it should.

It also reverses one conclusion. At `"s"`, the pm-AMM's Gaussian solve saved 7,395 gas against a
7,916 handicap and therefore lost. At `3` it saves 34,766 and wins by a wide margin.

### The pm-AMM, and the rule that explains every result above

TWAMM stopped being the interesting case once the deployed version turned out to use plain integer
`mulDiv`. The pm-AMM looked like a better bet, for one reason: unlike TWAMM's, its arithmetic cannot
be removed. With `z = (y-x)/L`, Paradigm's invariant is

```text
f(y)  = (y - x)·Phi(z) + L·phi(z) - y
f'(y) = Phi(z) - 1
```

transcendental in `y`, so a numerical solve is mandatory and every iteration needs a Gaussian CDF and
PDF. `Gnome101/Pm-AMM-Hook`, the one v4 hook that implements one, solves it by 100-step bisection —
200 Gaussian evaluations per swap, 401,100 gas. That is not what is measured here. Newton is the
honest floor, and `./bench-pmamm.bash` measures it: `uniswap/src/PmAmmMath.sol` runs
`primitivefinance/solstat`, `stylus/native-gaussian` is a bit-exact port of the same library —
Solmate's `expWad` included — and both test suites pin the same table of values, so the two agree to
the wei on chain before anything is timed.

**It wins, but only once the contract is compiled for speed — and getting there took a flag.**

The first measurement said it lost, at `opt-level = "s"`. Trying `2` or `3` instead produced a wasm
ArbOS flatly refuses to activate: *"unsupported section type DataCountSection"*. rustc emits
`memory.copy` at the speed levels, that brings a `DataCount` section, and the ArbOS prover does not
support it. **So a Stylus contract is capped at `"s"` or `"z"` by default — compiled for size, on a
platform that charges for execution.** Adding `--llvm-memory-copy-fill-lowering` to the wasm-opt
flags lowers those bulk-memory ops back to loops, drops the section, and unlocks the speed levels for
about 200 extra bytes.

What that is worth, on this contract, cached and net of call overhead:

| | Solidity | Rust, built for size | Rust, built for speed | ratio |
| --- | ---: | ---: | ---: | ---: |
| `expWad` | 451 | 2,569 | 1,246 | 0.36× |
| `cdf` | 5,137 | 3,828 | **2,082** | **2.47×** |
| Newton solve, 1 iteration | 7,996 | 6,539 | **3,536** | **2.26×** |
| Newton solve, 8 iterations | 62,425 | 55,031 | **27,659** | **2.26×** |

Bit-exactness survives the change: the two implementations still agree to the wei on 32 values and
four solves, checked on chain at `opt-level = 3`.

So a pm-AMM solve saves **34,766 gas**, against the 7,916 a cached Stylus hook carries — **+26,850
gas per swap in Rust's favour**, and paid on every swap rather than only when intervals are crossed.
That is the first hook workload in this document that clears the bar for a reason that survives
scrutiny.

`expWad` still loses, and that is the interesting residue: it converts into a 2^96 basis on purpose,
for precision, so every multiply and shift in it is full-width, and full-width 256-bit arithmetic is
where the EVM's single opcodes are hardest to beat.

Operand width is measured in the summary table near the top of this document, and it is the reason
`expWad` is the one row Rust loses: it converts into a 2^96 basis on purpose, for precision, so every
multiply and shift in it is full-width.

### Against the TWAMM that is actually in production

Everything above times the arithmetic on its own, called directly with no storage, no pool and no
orders. That is the right way to compare two languages and the wrong way to answer "what does an
interval cost", so `./bench-twamm.bash` also drives both hooks for real — and the control is not a
strawman of my own making. It is [`akshatmittal/v4-twamm-hook`][prod], written by Uniswap Labs and
Zaha Studio, audited by ABDK Consulting and Certora, live on Base and Unichain.

[prod]: https://github.com/akshatmittal/v4-twamm-hook

That hook rewrites the premise this document started from. It has **no floating point at all**: it
never evaluates an exponential, matching the two order pools against each other at the pool price
with integer `mulDiv` and swapping only the imbalance. The 489,927 gas figure that made TWAMM look
like the workload worth porting came from the v4-periphery *example*, removed in December 2024.
Production had already made the algorithmic saving, and more of it than the 1.7× measured above.

Both hooks on the same node, same pool manager, same 5-second expiry grid, eight orders each, two
per expiry. The Rust program is in Arbitrum's cache, which is what any hook with users would be:

| | Rust | production Solidity |
| --- | ---: | ---: |
| swap, pool idle | 138,746 | 132,257 |
| swap, one span of virtual orders | **297,015** | 339,609 |
| swap, four expiries crossed | **402,484** | 494,948 |
| **per expiry crossed** | **26,367** | **38,834** |

Baseline swap with no hook at all: 115,065.

So the Rust hook is **6,489 gas worse on an idle pool** and **12,467 better per expiry — 32 %** —
and it is ahead from the first span of real work, because that first span alone saves 42,594.

Note how little building for speed moved this one: 26,924 gas per expiry before, 26,367 after. The
pm-AMM's solve halved over the same change. That is the cleanest confirmation of what the flag
actually does — **it buys arithmetic, and a TWAMM expiry is 92 % storage.**

**This is not a language result.** The two implementations differ in a way that flatters mine:
theirs settles against the pool at every interval, mine computes every span first and settles once
at the end. Mine is also plainly less finished — no MEV mitigations, no kill switch, no batched
claims, no splitting a span at an initialised tick. Some unknown part of that 31 % is the work I
have not done. What the measurement does establish is that a pure-Rust hook holds its own against a
production Solidity one on the workload where Stylus should be strongest.

### What the caching is, and why the warm number is the honest one

A Stylus contract is compressed WASM, and the node must fetch, decompress and instantiate it before
running a byte of it. Arbitrum charges that as init gas on **every call**: 30,282 for this contract
uncached, 5,024 cached — a fixed cost per call that Solidity does not pay, and one that grows with
the binary.

"Cached" is not a transient state that warms up during a transaction. It is a property of the
deployment: Arbitrum keeps a 512 MiB cache of compiled programs, and getting in is a one-off bid on
`CacheManager` (`0x51dEDBD2f190E0696AFbEE5E60bFdE96d86464ec` on One). A hook nobody has bid for pays
the full load forever; a hook with users would be bid for. So the table above reports the cached
figures, and the difference is measured rather than quoted: a dev node has no CacheManager, but the
chain owner can appoint one, and the dev account owns the chain, so `cache_stylus_program` appoints
itself and caches the program directly. The same idle swap costs 174,476 uncached and 140,173
cached.

One caveat on precision: the idle figure moved by about 9,000 gas between runs, because whether a
swap lands in the same second as the previous one decides if the hook has a span to execute at all.
The per-expiry numbers, which are differences over four expiries, are stable.

Two things about the build are worth recording, because both are the opposite of what the
documentation suggests:

- **`opt-level = "z"` is a bad trade here.** It makes this contract 12 % smaller and 46 % more
  expensive to run — 3,430 gas per interval against 2,356 — because the SDK runs `wasm-opt -Oz`
  afterwards regardless, so all `z` adds is a slower code generator. Left at `"s"`.
- **The hook does not fit in one code object.** 24 KB is one; ArbOS 61 lifts the ceiling by
  splitting a contract across up to four fragments, and Arbitrum One and Sepolia both report four
  today. But a hook's address encodes its callbacks, so it has to be CREATE2'd from a mined salt,
  and `cargo stylus get-initcode` refuses fragmented contracts — the init code contains the
  fragment addresses, and those do not exist until the fragments are deployed. `bench-lib.bash`
  works around it; `FEEDBACK.md` is where it belongs.

This is the first workload in this document where porting to Stylus is worth doing, and the reason
it is worth doing is not that the arithmetic is exotic. It is that there is enough of it.

The interval implemented here is the price update — the closed form and its exponential. Uniswap's
example also computes earnings factors for both order pools and writes back the order-pool state,
which is why theirs costs ~100,000 per interval where this one costs 23,700. The 6× applies to the
arithmetic, not to the storage they wrap around it.

## The search, exhausted

`Uniswap/hooklist` was searched three ways. By permission flags:
`beforeSwapReturnsDelta` marks a hook that prices swaps itself, which is 17 of the 33 on Arbitrum.
By description, across five categories whose arithmetic could plausibly clear the bar — zero-knowledge
proofs, encryption, signatures, options pricing, order books. And by deployed bytecode size, which
needs no description at all and is the most honest screen: a hook that fits comfortably under the
24,576-byte limit is not doing much.

The largest hooks on Arbitrum:

| bytes | hook | status |
| ---: | --- | --- |
| 24,039 | DopplerHookInitializer | not measured — no quote function to isolate |
| 23,988 | **TokiHook** | not measured — Pendle-style fixed-rate curve, the one untested profile |
| 23,510 | BunniHook | measured: `rpow` at Q96, 1.65×, and the protocol is dead |
| 23,465 | GlueHook | buyback-and-burn, little arithmetic |
| 22,906 | Alphix | not measured |
| 17,835 | WLimitOrderHook | limit orders, storage-bound |

The keyword sweep turned up nothing genuinely cryptographic. `UniswapV4KEMHook` was the closest —
"KEM" reads as post-quantum, which would clear the bar by an order of magnitude — but it is an
ordinary signed-quote RFQ hook using `ecrecover`, a 3,000-gas precompile that Stylus cannot beat.

**TokiHook is the strongest remaining candidate and it is unmeasured.** A Pendle-style fixed-rate
curve runs `exp` and `ln` in fixed point, the one arithmetic family not benchmarked here, and the
one the EVM has no support for whatsoever. Measuring it needs a swap executed against it: it exposes
no quote function to isolate, and anvil cannot fork Arbitrum because its block headers carry no blob
fields. From the shape of the primitive — a bit-scan, which Stylus wins at 4.5×, over a polynomial in
full-range 256-bit, which it wins at 2.8× — the ratio should land between the two, and a curve
evaluation is a handful of them. That is an estimate, not a measurement, and it does not reach the
bar.

## The workload that was already in the codebase: v4-core's swap math

The search above looked for a hook with unusual arithmetic. The best answer turned out to be the
arithmetic every hook already has underneath it.

`./bench-v4-math.bash` measures `SwapMath`, `TickMath`, `SqrtPriceMath` and `FullMath`. The control
side is not a reimplementation: [`uniswap/src/V4MathBench.sol`](uniswap/src/V4MathBench.sol) calls
those libraries straight out of v4-core. The Rust side is [`stylus/v4-math`](stylus/v4-math), a port
tested against v4-core's own vectors — every unit test in `test/libraries/` for those five, same
inputs, same expected values, 60 tests.

The workload is `walkSwap`: `Pool.swap`'s loop without the storage. Find the next initialised tick,
price the step up to it, cross, repeat. Replaying that loop is what OpenZeppelin's
`AntiSandwichHook` and Uniswap's own `alf/SwapSimulator` do on every swap.

| ticks crossed | Solidity | Rust | ratio | saving |
| ---: | ---: | ---: | ---: | ---: |
| 0 (the call itself) | 26,449 | 44,202 | — | −17,753 |
| 1 | 30,442 | 44,844 | 6.22× | −14,402 |
| 2 | 34,445 | 45,403 | 6.66× | −10,958 |
| 4 | 42,451 | 46,522 | 6.90× | −4,071 |
| 8 | 58,870 | 48,380 | 7.76× | **+10,490** |
| 16 | 90,957 | 52,853 | 7.46× | **+38,104** |
| 32 | 155,342 | 62,055 | 7.22× | **+93,287** |
| 64 | 286,027 | 80,698 | 7.11× | **+205,329** |

**4,056 gas per tick in Solidity against 570 in Rust.** The saving column is end to end with the
cached init gas already in it, so the crossover is where it changes sign: **a replay that crosses six
or more ticks is a net win**, and a deep one saves more than the whole baseline swap costs.

The primitives, at 100 iterations so the call overhead cancels:

| | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| `computeSwapStep` | 1,790 | 394 | **4.55×** |
| `getSqrtPriceAtTick` | 882 | 193 | **4.57×** |
| `getTickAtSqrtPrice` | 1,972 | 652 | **3.02×** |
| `mulDiv`, max inputs | 2,063 | 1,703 | 1.21× |

This is the widest margin in this document on arithmetic that is not contrived, and the reason is
worth stating: this code is *bit-twiddling*, not big-number arithmetic. `getSqrtPriceAtTick` is
nineteen shifts and a conditional multiply; the log in `getTickAtSqrtPrice` is fourteen squarings and
a shift. Solidity pays a 5-gas opcode for each one and can express none of them more cheaply. Rust
compiles them to what they are.

### The mistake in the first run, because it is the more useful result

The first measurement had Rust **losing** at `getSqrtPriceAtTick`, 0.28×. The cause was not the
language. The nineteen Q128.128 factors were `&str` constants parsed with `from_str_radix` inside the
loop, and `getTickAtSqrtPrice` parsed three decimal constants per call. Hoisting them to real `const`
values with `uint!` moved the ratio from **0.28× to 4.57×** — a sixteenfold change in the Rust, with
the algorithm untouched.

Nothing warns you. The code reads fine, the tests pass, `cargo stylus check` passes. In Solidity a
literal is a literal and there is no way to write this bug; in Rust a `U256` has to be built from
something, and building it from text is both the most readable option and roughly a hundred times the
cost of the arithmetic it feeds. This is the same class of trap as the `opt-level` one below: a
default that silently costs about an order of magnitude on the only thing Stylus is bought for.

## The one Solidity cannot do at all: a post-quantum signature

Every measurement above asks how much cheaper Stylus is. This one asks whether Solidity can do the
thing, and the answer is no -- by two orders of magnitude, not by a margin.

`./bench-crypto.bash` measures the two primitives an ML-DSA (Dilithium) verification spends its gas
on. Both sides are pinned to the standard before anything is timed: NIST's SHAKE256 vectors, XKCP's
vector for the permutation on a zero state, and the same transform output.

### Why the built-in hash does not help

Stylus has `native_keccak256` and the EVM has the `KECCAK256` opcode. Both are Keccak-256 with the
`0x01` pad compiled in. SHAKE pads with `0x1f` and squeezes an arbitrary length, so neither built-in
can produce it, and anything built on SHAKE -- every ML-DSA, ML-KEM and SLH-DSA operation -- has to
run Keccak-f[1600] itself.

That is the whole distance between these two columns:

| | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| one 32-byte hash, through the built-in | 184 | 16 | 11.5× |
| one Keccak-f[1600] permutation, by hand | **101,662** | **158** | **643×** |

The first row is worth its own note: the Stylus host's keccak is **11.5× cheaper than the EVM
opcode**, which is free money for any hook that hashes. The second row is the one that decides
things. A `keccak256` of 32 bytes costs 36 gas of opcode and performs exactly one permutation
internally; the same permutation written out costs 101,662. The EVM can do Keccak, but only through
the single door it provides, and SHAKE is not behind that door.

### SHAKE256 and the transform

| | Solidity | Rust | ratio |
| --- | ---: | ---: | ---: |
| SHAKE256, absorb 32 B, squeeze 32 B | 140,455 | 508 | 276× |
| SHAKE256, absorb 32 B, squeeze 168 B | 276,525 | 727 | 380× |
| SHAKE256, absorb 200 B, squeeze 1088 B | 1,250,263 | 2,559 | **489×** |
| one forward NTT, 256 coefficients | 244,873 | 2,128 | **115×** |
| the same NTT, in the Montgomery domain | — | 1,955 | 125× |

The transform is the word-size problem rather than the missing-primitive one. ML-DSA's modulus is
`8380417`, twenty-three bits wide. The EVM has one word and it is 256 bits, so every butterfly pays a
full `MULMOD`; in Rust it is a `u32` multiply into a `u64`. Montgomery has no Solidity counterpart at
all, because `MULMOD` already reduces for free -- it is measured only to say how much further Rust
goes once it stops imitating the EVM.

### What that projects to

ML-DSA-44 verification runs roughly 90 permutations -- nearly all of them in `ExpandA`, which
rejection-samples a 4×4 matrix of polynomials out of SHAKE128 -- and about 9 transforms.

| | Solidity | Rust |
| --- | ---: | ---: |
| permutations | 9,149,580 | 14,220 |
| transforms | 2,203,857 | 19,152 |
| **total** | **11,353,437** | **33,372** |

A swap on Arbitrum costs about 115,000 gas with no hook at all. So Solidity would spend **99 swaps'
worth of gas** to check one signature, and a third of an Ethereum block; Rust spends less than a
third of a swap. That is the difference between a hook that cannot exist and one that is unremarkable.

### Two things this does not say

**The contract size limit is not the barrier here, and I said it would be before measuring.** The
Solidity contract is 9,297 bytes and the Rust one 11,629 compressed; both fit under 24,576 with room
to spare. For ML-DSA the gas is the whole story, and it is about a hundred times more barrier than
needed.

**The Solidity side could still be faster.** Its permutation and butterfly are unrolled inline
assembly, because the readable versions spent most of their gas on bounds-checked array access --
667,204 against 101,662 for the permutation, 462,305 against 244,873 for the transform, and those
rows are in the script's output so the choice is visible rather than asserted. A maximally tuned
implementation might reach half the figure above again. It would change 643× to roughly 350×, and
change nothing about the conclusion.

## What this means for the project

A bookkeeping hook is the wrong thing to port. Read a counter, add to it,
write it back. That is the exact shape of hook where Stylus has nothing to offer.

Stylus pays off when a hook has to *think* on every swap — past the crossover measured above, which
is ~371 rounds of small-word arithmetic or 89 `mulDiv`s. In practice:

- **small-word cryptography**, which is the only place measured here where Solidity does not merely
  lose but cannot play: 643× on a Keccak permutation, 115× on a 23-bit NTT
- **replaying v4's own swap math**, which is the clearest case among things Solidity *can* do: 7.1×
  per tick, and any hook that simulates a swap before allowing it does exactly this
- on-chain math: TWAP and volatility oracles, curve solvers, Newton iterations
- dynamic fees computed from a model rather than looked up
- anything scanning or sorting a batch, where the loop dominates
- signature or proof verification inside the callback

Those are the hooks worth writing in Rust, and they are what
[`stylus/base-hook`](stylus/base-hook) exists to make writable end-to-end.

Two traps account for most of the distance between a Stylus hook that loses and one that wins, and
neither announces itself: `opt-level` capped at `"s"` by a WASM section ArbOS rejects, and constants
built from strings at runtime. Each was worth roughly an order of magnitude on arithmetic, both
compile and both pass `cargo stylus check`.

## A caveat on the numbers

Gas here is L2 execution gas only. The benchmarks set the L1 data-posting price to zero
(`ArbOwner.setL1PricePerUnit(0)`) because that component is identical across the variants being
compared — same calldata, same swap — and on a real chain it dwarfs the differences being measured.

Nothing here is measured inside `forge test`: forge cannot execute WASM, so every Stylus figure comes
from a transaction on a dev node.
