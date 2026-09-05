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

## What this means for the project

The airdrop hook was the wrong thing to port. It is a bookkeeping hook: read a counter, add to it,
write it back. That is the exact shape of hook where Stylus has nothing to offer.

Stylus pays off when a hook has to *think* on every swap — the cases that are too expensive to write
in Solidity at all:

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
