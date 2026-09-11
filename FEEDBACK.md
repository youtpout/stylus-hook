# Feedback on the Uniswap v4 stack

This is feedback from an unusual angle: I spent this project building v4 hooks in **Rust**, compiled
to WASM and run on Arbitrum Stylus, with no Solidity in them at all. That meant reimplementing
`BaseHook`, the `IPoolManager` calldata encodings, and v4-core's swap math from scratch, and then
measuring the result against the Solidity the port came from.

Porting something is the harshest way to read it. Every assumption the original could leave implicit
has to be made explicit, and every test has to be re-derived rather than trusted. Most of what
follows comes out of that.

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

At exactly `tick == MAX_TICK - 1`, the last assertion asks for the tick of `getSqrtPriceAtTick(MAX_TICK)`
— which is `MAX_SQRT_PRICE`, which `getTickAtSqrtPrice` rejects **by design**, because a live price
can never reach the max tick. The test cannot pass at that input.

I found it because the Rust port checks the same property by sweeping all 1,774,543 ticks instead of
sampling, and then reproduced it in Solidity to be sure it was not my port:

```
[FAIL: InvalidSqrtPrice(1461446703485210103287273052203988822378723970342)] test_relationAtMaxTickMinusOne()
```

Forge's fuzzer has simply never drawn that one value out of 1.77 million. **The fix is one
character**: bound at `MAX_TICK - 2`.

The general note behind it: a randomised fuzz test over a bounded integer domain of ~2²¹ values is
strictly worse than an exhaustive sweep, and the sweep here takes 8.7 seconds. Where a v4-core
property is over ticks, fee pips, or any other small domain, it is cheap to check all of it.

---

## 2. The hook safety contract is carried by Solidity idiom, not by specification

`BaseHook.sol` is an abstract contract, so `onlyPoolManager` is a guard a hook author cannot forget —
the compiler will not let them. That is excellent design *in Solidity*, and it does not survive
leaving Solidity. Rust has no abstract types. Nothing in the protocol documentation states the
invariants as invariants; they are encoded in a base class.

For anyone implementing hooks in another language — or generating them, or verifying them — it would
help to have the contract written down plainly. As far as I can reconstruct it, a hook must:

1. revert unless `msg.sender == poolManager`;
2. revert unless the `PoolKey` it was handed hashes to a pool it is actually the hook for;
3. return its own selector, and nothing else, from every callback;
4. only return a non-zero `BeforeSwapDelta` when the corresponding permission bit is set.

I ended up enforcing (1) and (2) with a procedural macro
([`stylus/base-hook-macros/src/lib.rs`](stylus/base-hook-macros/src/lib.rs)) so they are inserted into
every callback rather than inherited. That works, but I had to derive the list by reading
`BaseHook.sol` and `Hooks.sol` rather than by reading a spec.

**Suggestion:** a short "hook invariants" page in the docs, stated independently of
`BaseHook.sol`, would make v4 portable to other VMs and easier to audit in Solidity too.

---

## 3. Permission bits in the address constrain what can host a hook

A hook's address encodes its permissions, so deploying one means mining a CREATE2 salt over the
initcode. That is fine for Solidity, where the runtime is capped at 24,576 bytes anyway.

It breaks for larger runtimes. A Stylus contract above 24,576 *compressed* bytes is deployed as
multiple fragments behind a root contract, and there is then no single initcode to mine over. I
worked around it by reconstructing the deployment prelude by hand
([`bench-lib.bash:51`](bench-lib.bash#L51)), but the general point stands: **the permission-in-address
design silently sets a size ceiling on any hook, in any language.**

This is not a bug, and I do not think it should change — the address encoding is a genuinely good
trick and it is load-bearing. But it is worth knowing that it is what will stop a hook from being
written in a language whose runtime is bigger than Solidity's, and it may be worth a line in the
docs.

---

## 4. A settling hook has to re-enter the PoolManager, and nothing says so

Any hook that settles its own deltas calls back into the PoolManager inside the callback it was
called from. In Solidity this is invisible. On Stylus, re-entrancy is off by default and the
entrypoint returns a bare revert with **no data at all** — which produced a `TokenCallFailed()` with
an empty reason and cost me an afternoon.

That is Stylus's fault, not Uniswap's. The Uniswap-side note is smaller: the docs describe flash
accounting and `settle`/`take` clearly, but never say in one sentence that **a hook that settles is a
re-entrant caller of the singleton**. For anyone whose platform treats re-entrancy as opt-in, that
sentence is the whole difference.

---

## 5. Two ecosystem things worth knowing

**The v4-periphery TWAMM example is gone, and it is still what people benchmark.** The figure that
made TWAMM look like the obvious compute-heavy hook — around 490,000 gas for a swap crossing an
expiry — comes from the example removed in December 2024. The TWAMM actually in production
([`akshatmittal/v4-twamm-hook`](https://github.com/akshatmittal/v4-twamm-hook), by Uniswap Labs and
Zaha Studio) has **no floating point at all**: it matches the two order pools against each other at
the pool price with integer `mulDiv` and swaps only the imbalance. I built a whole hook on the old
premise before measuring the real one. A note in the repo pointing removed examples at their
production successors would save other people the same detour.

**`AntiSandwichHook` keys its checkpoint on `block.number`.** This is OpenZeppelin's hook, not
Uniswap's, but it ships in the library the official v4-template depends on. On Arbitrum the `NUMBER`
opcode returns the **L1** block number, not the L2 one — measured on Arbitrum One itself:

| | |
| --- | ---: |
| Solidity `block.number` | 25,912,325 |
| Ethereum L1 height, same moment | 25,912,326 |
| Arbitrum L2 block | 502,057,928 |

So the hook's per-block checkpoint spans roughly 240 L2 blocks on Arbitrum, and on a dev node with no
L1 it never advances at all. The fix is one line — `ArbSys.arbBlockNumber()` — but nothing in the
hook tells you to make it, and Arbitrum is where a lot of v4 volume is.

---

## 6. What worked well, specifically

**Flash accounting and the singleton are what made any of this possible.** A hook that computes
several spans of virtual orders and settles once at the end, or one that calls `swap` on other pools
inside its own callback, are both natural under `unlock`/`settle`/`take` and would be painful under
v3's model. This is the part of v4 that most changes what a hook can be, and it held up under a
reimplementation that gave it no benefit of the doubt.

**`beforeSwapReturnsDelta` is the right primitive.** It is the single permission that turns a hook
from an observer into a market maker, and the fact that it is one bit rather than a separate
interface is the reason a custom curve is a small amount of code.

**The library boundaries are clean enough to port one at a time.** `FullMath`, `UnsafeMath`,
`SqrtPriceMath`, `SwapMath` and `TickMath` have no hidden coupling: I ported them function for
function and each one's tests passed against Uniswap's own vectors before I touched the next. That is
not true of most protocol code.

**The test vectors are good.** Porting against `test/libraries/` caught every mistake I made — and I
made several, including one place where alloy's `I256 >> n` is a *logical* shift where Solidity's is
arithmetic, which would have silently broken every tick below zero. Uniswap's tests found it. The one
place a test did not catch me is §1, and that is because the test was wrong rather than weak.

---

## 7. Appendix: what it cost on the other side

Not Uniswap's problem, recorded because it explains what the port was fighting. Full detail and
method in [BENCHMARK.md](BENCHMARK.md).

- **`opt-level = 2` and `3` emit a WASM `DataCount` section that ArbOS refuses to activate.** A Stylus
  contract is therefore silently capped at size optimisation on a platform that charges for
  execution. The build succeeds, `cargo stylus check` succeeds, only activation fails. Passing
  `--llvm-memory-copy-fill-lowering` fixes it and is worth roughly **2× the gas** on arithmetic.
- **Constants built from strings are parsed at runtime.** `TickMath`'s nineteen Q128.128 factors as
  `&str` parsed inside the loop measured **0.28×** against Solidity; as real `const` values, **4.57×**.
  In Solidity a literal is a literal and this bug cannot be written.
- **No `immutable`, no transient storage, and a `static` does not persist across calls** (measured:
  wrote 42, read back 777). The pool manager address ended up as a build-time constant.
- **`cargo stylus get-initcode` refuses fragmented contracts**, which is what breaks hook address
  mining in §3.
- **SHAKE256 is unreachable from any built-in hash, on either platform.** `keccak256` and Stylus's
  `native_keccak256` are both Keccak-256 with the `0x01` pad compiled in; SHAKE pads with `0x1f`. Both
  languages must run Keccak-f[1600] themselves — 101,662 gas in Solidity, **158 in Rust**. Separately,
  the Stylus host's keccak is **11.5× cheaper than the EVM opcode**, which is free money for any hook
  that hashes.

---

## What the numbers said, in one line

Stylus is not faster; it is *differently priced*. It costs more to enter a hook and far less to
compute inside one. A hook that reads a counter and writes it back loses. A hook that replays
`Pool.swap` wins at **7.1× per tick crossed**, and one that needs a hash that is not `keccak256` wins
by **two orders of magnitude** — which is the difference between a hook that is expensive and a hook
that cannot exist.
