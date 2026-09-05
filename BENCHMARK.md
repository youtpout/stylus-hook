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

BunniHook is the strongest candidate found anywhere so far: it replaces the constant-product curve
outright, and it is deployed on ten chains including Arbitrum, so it can be profiled against a fork
of Arbitrum One with `profile-hooks.bash`'s tracer rather than reconstructed. That is the measurement
worth doing next.

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

BunniHook is the strongest candidate found anywhere so far: it replaces the constant-product curve
outright, and it is deployed on ten chains including Arbitrum, so it can be profiled against a fork
of Arbitrum One with `profile-hooks.bash`'s tracer rather than reconstructed. That is the measurement
worth doing next.

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
