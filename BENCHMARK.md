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
| no hook | 124,590 | — |
| hook in one Solidity contract | 160,276 | +35,686 |
| hook split over two Solidity contracts | 166,835 | +42,245 |
| **hook with its state in Stylus** | **201,121** | **+76,531** |
| hook with its state in Stylus, if cached | 183,894 (projected) | +59,304 |

Breaking down the 76,531:

| | gas |
| --- | ---: |
| the accounting itself, as Solidity would do it | 35,686 |
| the extra call the split design needs | 6,559 |
| Stylus, over the identical Solidity callee | 34,286 |
| — of which loading the WASM program, uncached | 20,439 |
| — the same, once the contract is cached | 3,212 |

**Stylus costs 2.14× what Solidity does for this hook, or 1.66× once the contract is cached.**

The dev node has no `CacheManager`, so the cached row is computed from
`ArbWasm.programInitGas(address)`, which reports both figures, rather than measured directly.

## Why Stylus loses here

Stylus makes *compute* roughly an order of magnitude cheaper. It does not make *storage* cheaper:
`SLOAD` and `SSTORE` are host calls priced in EVM gas either way.

`afterSwap` on this hook does about six `SLOAD`s and six `SSTORE`s and almost no arithmetic — six
warm `SSTORE`s alone are 17,400 gas of the 35,686 the Solidity version costs. There is no compute
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
| no hook | 120,224 | — |
| `Counter.sol`, one Solidity contract | 141,891 | +21,667 |
| `CounterProxy.sol` → Stylus | 207,885 | +87,661 |
| **`native-counter`, no Solidity in the hook** | **204,452** | **+84,228** |
| the same, if the contract is cached | 189,355 (projected) | +69,131 |

Going fully native saves **3,433 gas**, and only after Binaryen gets involved. Without `-Oz` the
native hook was *worse* than the shell, by 3,859:

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
call — it is one less contract, one less trust assumption, and it does now win on gas — but the
margin is thin, and it will only widen for a hook whose own work is worth more than its front door.

Against pure Solidity the native hook still costs 62,561 gas more per swap. Same conclusion as the
airdrop: this hook writes one storage slot per callback and computes nothing.

## Where Rust starts winning

The two hooks above both lose to Solidity, which invites the wrong conclusion. Stylus is not slower;
it is *differently priced*. It costs more to enter and far less to run. `./bench-compute.bash`
measures both halves of that: `ComputeHook.sol` and `stylus/native-compute` run the identical
xorshift64 loop in `beforeSwap`, and the only thing that changes between rows is how many rounds.

### xorshift64 on a `u64`

| rounds | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 130,080 | 167,626 | +37,546 |
| 50 | 137,202 | 168,142 | +30,940 |
| 200 | 158,652 | 169,774 | +11,122 |
| 500 | 201,580 | **173,067** | −28,513 |
| 1,000 | 273,116 | **178,543** | −94,573 |
| 5,000 | 845,048 | **222,001** | −623,047 |

143 gas per round in Solidity, 10.9 in Rust: **13× cheaper**. The crossover is at ~284 rounds.

### `mulDiv` on 256-bit words

This is the one that matters, because it is what Uniswap's own swap math is made of.
`SwapMath.computeSwapStep`, `SqrtPriceMath` and every tick-walking simulation are mostly chains of
`FullMath.mulDiv`.

| rounds | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 130,092 | 168,577 | +38,485 |
| 50 | 179,538 | 181,049 | +1,511 |
| 200 | 327,588 | **218,177** | −109,411 |
| 500 | 623,624 | **292,369** | −331,255 |
| 1,000 | 1,117,188 | **416,193** | −700,995 |
| 5,000 | 5,065,156 | **1,406,241** | −3,658,915 |

987 gas per `mulDiv` in Solidity, 247 in Rust: **4× cheaper**, and the crossover is only **52
operations** — about 51,000 gas of Solidity-side arithmetic.

That is the opposite of what the 256-bit word size suggests, and the reason is worth stating.
A bare `MUL` or `DIV` *is* a single 5-gas EVM opcode, and Stylus would lose that comparison. But
Uniswap does not use bare `mul` and `div` for pool math — it uses `FullMath.mulDiv`, which needs the
full 512-bit product, and the EVM has no 512-bit anything. Solidity gets there with Remco Bloemen's
long-division routine: `mulmod`, a modular inverse built by Newton iteration, and several dozen
opcodes. Rust gets there with a `U512` multiply and divide over limbs. The EVM's 256-bit word is an
advantage right up to the point where you need 257 bits, which is most of Uniswap's math.

### Writing storage

There is no such thing as "Stylus storage" as distinct from "Solidity storage". A Stylus contract
writes the same 32-byte slots in the same account trie, and ArbOS charges EVM prices for reaching
them. This mode checks that rather than assuming it — each round writes one mapping slot.

| slots written | Solidity | Rust | delta |
| ---: | ---: | ---: | ---: |
| 0 | 130,082 | 167,986 | +37,904 |
| 5 | 142,777 | 179,719 | +36,942 |
| 25 | 193,621 | 226,841 | +33,220 |
| 100 | 383,986 | 403,411 | +19,425 |

2,539 gas per slot in Solidity, 2,354 in Rust. Stylus is **7 % cheaper**, not 4× and not 13×, and
that margin is not the store itself: a cold slot costs 2,100 to touch and 100 to write the value it
already holds, identically in both. What Stylus shaves is the arithmetic wrapped around the access —
hashing the mapping key, and the loop. Paying off the entry fee on storage alone would take about
205 writes per call.

So the earlier claim in this document — that storage costs the same in both worlds — is very nearly
right, and right for every purpose a hook cares about. Moving state into a Stylus contract does not
make it cheaper to store; it makes the code around it cheaper.

### The fixed cost

At zero rounds the Rust hook still costs ~38,000 gas more, of which 21,759 is loading its WASM
program on every call — 5,417 once the contract is cached. That figure is higher here than for
`native-counter` because this hook carries both workloads and `U512` arithmetic; the program is
bigger, and `ArbWasm` charges by compiled size.

## What this means for the project## What this means for the project

The airdrop hook was the wrong thing to port. It is a bookkeeping hook: read a counter, add to it,
write it back. That is the exact shape of hook where Stylus has nothing to offer.

Stylus pays off when a hook has to *think* on every swap — past the crossover measured above, which
is ~284 rounds of small-word arithmetic or only ~52 `mulDiv`s. In practice:

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
(35,686 / 42,245 hook cost measured here vs 33,820 / 40,375 in forge) track the on-chain ones
closely for the Solidity variants, which is what makes it useful as a fast check.
