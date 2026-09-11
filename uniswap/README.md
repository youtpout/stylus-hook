# uniswap — the Solidity side

Foundry project holding the Uniswap v4 hooks. See the [root README](../README.md) for the overall
design.

| Contract | What it is |
| --- | --- |
| [`src/PmAmmMath.sol`](src/PmAmmMath.sol) | the pm-AMM's Gaussian solve — the twin of `stylus/native-gaussian`, pinned to the same values |
| [`src/TwammHook.sol`](src/TwammHook.sol) | a TWAMM interval, in quad floats and in fixed point |
| [`src/StableSwapHook.sol`](src/StableSwapHook.sol) | a StableSwap curve, for the arithmetic sweep |
| [`src/ComputeHook.sol`](src/ComputeHook.sol) | pure arithmetic, to find the crossover |
| [`src/CounterProxy.sol`](src/CounterProxy.sol) | v4 hook whose counters live in [`stylus/counter`](../stylus/counter) |
| [`src/Counter.sol`](src/Counter.sol) | the same hook in pure Solidity, as a baseline |
| [`src/ICounter.sol`](src/ICounter.sol) | the ABI the Stylus counter exposes |

Hooks inherit `BaseHook` from [`@openzeppelin/uniswap-hooks`](https://github.com/OpenZeppelin/uniswap-hooks),
the base contract the official [v4-template](https://github.com/uniswapfoundation/v4-template) uses.

```bash
forge test
forge build
```

Tests deploy a full local v4 stack through [`hookmate`](https://github.com/akshatmittal/hookmate)
and swap through the hooks for real. Since forge cannot run WASM,
Forge cannot execute WASM, so the Rust side is tested with `TestVM` on the Cargo side; where both
implement the same maths they are pinned to the same values.

## Scripts

| Script | Deploys |
| --- | --- |
| `02_DeployStylusCounterHook.s.sol` | `CounterProxy` in front of `$STYLUS_COUNTER`, then binds them |
