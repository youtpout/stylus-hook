# Feedback on Uniswap v4

This is feedback from an unusual angle: I spent this project building v4 hooks in **Rust**, compiled
to WASM and run on Arbitrum Stylus, with no Solidity in them at all. That meant reimplementing the
`IHooks` callbacks, the `IPoolManager` calldata encodings, and v4-core's swap math from scratch.

Porting something is the harshest way to read it. Every assumption the original could leave implicit
has to be made explicit, and every test has to be re-derived rather than trusted. All of what follows
came out of that.

Everything here is against **v4-core `v4.0.0-19-gd153b048`**.

---

## 1. A failing input in v4-core's own test suite

`test/libraries/TickMath.t.sol:205` — `test_fuzz_getTickAtSqrtPrice_getSqrtPriceAtTick_relation`:

```solidity
tick = int24(bound(tick, TickMath.MIN_TICK, TickMath.MAX_TICK - 1));
int24 nextTick = tick + 1;
...
assertEq(TickMath.getTickAtSqrtPrice(priceAtNextTick), nextTick, "lower price next tick");
```

At exactly `tick == MAX_TICK - 1`, the last assertion asks for the tick of
`getSqrtPriceAtTick(MAX_TICK)` — which is `MAX_SQRT_PRICE`, which `getTickAtSqrtPrice` rejects **by
design**, because a live price can never reach the max tick. The test cannot pass at that input.

I found it because my port checks the same property by sweeping all 1,774,543 ticks instead of
sampling, then reproduced it in Solidity to be sure it was not my port:

```
[FAIL: InvalidSqrtPrice(1461446703485210103287273052203988822378723970342)] test_relationAtMaxTickMinusOne()
```

Forge's fuzzer has simply never drawn that one value out of 1.77 million. **The fix is one
character**: bound at `MAX_TICK - 2`.

**The general suggestion behind it.** A randomised fuzz run over a bounded integer domain of ~2²¹
values is strictly worse than an exhaustive sweep, and the sweep here takes 8.7 seconds. Several
v4-core properties are over domains that small — ticks, fee pips, tick spacings. Where they are, it
is cheap to check all of them, and the fuzzer's job is better spent on the genuinely wide inputs
(`computeSwapStep`'s prices, liquidity and amounts, where I swept a deterministic grid instead and
found nothing).

---

## 2. The hook safety contract lives in a base class, not in a specification

`BaseHook` is an abstract contract, so `onlyPoolManager` is a guard a hook author cannot forget — the
compiler will not let them. That is excellent design *in Solidity*, and it does not survive leaving
Solidity. Rust has no abstract types, so there is nothing to inherit and nothing to enforce.

The problem is that the invariants exist only as that base class. To reimplement them I had to
reconstruct the list by reading `Hooks.sol`, `IHooks.sol` and a `BaseHook` implementation. As far as
I can tell, a hook must:

1. revert unless `msg.sender == poolManager`;
2. revert unless the `PoolKey` it was handed is one it is actually the hook for;
3. return its own selector, and nothing else, from every callback;
4. return a non-zero `BeforeSwapDelta` only when the corresponding permission bit is set.

I ended up enforcing (1) and (2) with a procedural macro
([`stylus/base-hook-macros/src/lib.rs`](stylus/base-hook-macros/src/lib.rs)) that inserts them into
every callback. It works, but I was guessing at a specification rather than implementing one.

**Suggestion:** a short "hook invariants" page in the docs, stated independently of any base
contract. It would make v4 portable to other VMs, and it would give Solidity hook auditors a
checklist that does not depend on which `BaseHook` the author inherited from.

---

## 3. Permission bits in the address put a size ceiling on any hook

A hook's address encodes its permissions in the low bits, so deploying one means mining a CREATE2
salt over its initcode. That is free in Solidity, where the runtime is capped at 24,576 bytes anyway.

It stops working for a larger runtime. Past a certain size a Stylus contract is deployed as several
code fragments behind a root contract, and there is then no single initcode to mine a salt over. I
got around it by reconstructing the deployment prelude by hand
([`bench-lib.bash:51`](bench-lib.bash#L51)), but the general point holds: **the permission-in-address
design silently bounds how large a hook can be, in any language.**

I do not think this should change — encoding permissions in the address is a genuinely good trick and
it is load-bearing for the gas savings. But it is the thing that will stop a hook from being written
in a runtime bigger than Solidity's, and that is worth a line in the docs rather than a discovery.

---

## 4. A hook that settles is a re-entrant caller of the singleton

Any hook that settles its own deltas calls back into the `PoolManager` from inside the callback the
`PoolManager` just made. Under flash accounting that is completely normal and completely invisible in
Solidity, which has no opinion about re-entrancy unless you add one.

The docs describe `unlock` / `settle` / `take` clearly, and describe `beforeSwapReturnsDelta`
clearly, but never say in one sentence that **a hook returning a delta will re-enter the singleton**.
For anyone whose platform treats re-entrancy as opt-in — and that is not only Stylus; it is any hook
using a standard `nonReentrant` modifier on its own external surface — that sentence is the whole
difference between a hook that works and one that reverts.

---

## 5. The removed v4-periphery TWAMM example is still what people benchmark

The figure that made TWAMM look like the obvious compute-heavy hook worth porting — roughly 490,000
gas for a swap crossing an expiry — comes from the v4-periphery example removed in December 2024.

The TWAMM actually in production
([`akshatmittal/v4-twamm-hook`](https://github.com/akshatmittal/v4-twamm-hook), by Uniswap Labs and
Zaha Studio, audited by ABDK and Certora) has **no floating point at all**. It matches the two order
pools against each other at the pool price with integer `mulDiv` and swaps only the imbalance.
Production had already made the algorithmic saving, and more of it than a language port could.

I built an entire hook on the old premise before measuring the real one. A line in the v4-periphery
repo pointing removed examples at their production successors would have saved me that, and the
example is still the first result people find.

---

## 6. What worked well, specifically

**Flash accounting and the singleton are what made any of this possible.** A hook that computes
several spans of virtual orders and settles once at the end, or one that calls `swap` on other pools
inside its own callback, are both natural under `unlock` / `settle` / `take` and would be painful
under v3's model. This is the part of v4 that most changes what a hook can be, and it held up under a
reimplementation that gave it no benefit of the doubt.

**`beforeSwapReturnsDelta` is the right primitive.** One permission bit turns a hook from an observer
into a market maker, and because it is a bit rather than a separate interface, a custom curve is a
small amount of code.

**The library boundaries are clean enough to port one at a time.** `FullMath`, `UnsafeMath`,
`SqrtPriceMath`, `SwapMath` and `TickMath` have no hidden coupling. I ported them function for
function and each one passed against Uniswap's own vectors before I touched the next. That is not
true of most protocol code.

**The assembly is commented with what it computes, not with how.** `SwapMath.getSqrtPriceTarget`,
`TickBitmap.compress` and `TickMath.getSqrtPriceAtTick` each state the plain-Solidity equivalent
above the `assembly` block. Those comments are the only reason the port was possible at all — they
are a specification, and they should be treated as a requirement rather than a courtesy.

**The test vectors caught every mistake I made**, and I made several. The one worth reporting back:
alloy's `I256 >> n` is a *logical* shift where Solidity's is arithmetic, which silently broke
`getTickAtSqrtPrice` for every price below tick 0 — half the range. `TickMath.t.sol`'s vectors found
it immediately. The only place a test did not catch me is §1, and that is because the test was wrong
rather than weak.

---

## 7. Which hook shapes this suggests are worth the trouble

Offered because it is the one thing this project can tell Uniswap that a Solidity implementer
cannot: which parts of a hook are compute-bound and which are not. Method and full numbers in
[BENCHMARK.md](BENCHMARK.md).

Measured against the same operation in Solidity, per swap:

| | |
| --- | ---: |
| replaying `Pool.swap`'s loop, per tick crossed | **7.1×** |
| `SwapMath.computeSwapStep` | **4.55×** |
| `TickMath.getSqrtPriceAtTick` | **4.57×** |
| a Gaussian solve, for a curve with no closed form | 2.26× |
| a TWAMM expiry, end to end | 1.47× |
| an `SSTORE` | 1.06× |

The shape of the answer: **a v4 hook's arithmetic is worth moving off the EVM and its storage is
not.** Hooks that count swaps, keep balances or read a vault gain nothing. Hooks that *simulate* —
anti-sandwich replays, price-impact limits, route evaluation, solvers for curves with no closed form
— gain a lot, and those are exactly the hooks `beforeSwapReturnsDelta` was added to allow.

The one that surprised me is `getSqrtPriceAtTick`. It is nineteen shifts and a conditional multiply,
and the EVM pays a 5-gas opcode for each with no cheaper way to write it. Tick math is the most
EVM-hostile code in v4-core, and it runs on the hot path of every swap.
