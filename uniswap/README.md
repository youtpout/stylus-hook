# uniswap — the Solidity side

Foundry project holding the Uniswap v4 hooks. See the [root README](../README.md) for the overall
design.

| Contract | What it is |
| --- | --- |
| [`src/AirdropHookProxy.sol`](src/AirdropHookProxy.sol) | v4 hook whose accounting lives in [`stylus/airdrop`](../stylus/airdrop) |
| [`src/AirdropHook.sol`](src/AirdropHook.sol) | the same hook in pure Solidity, as a baseline |
| [`src/CounterProxy.sol`](src/CounterProxy.sol) | v4 hook whose counters live in [`stylus/counter`](../stylus/counter) |
| [`src/Counter.sol`](src/Counter.sol) | the same hook in pure Solidity, as a baseline |
| [`src/interfaces/IAirdropHook.sol`](src/interfaces/IAirdropHook.sol) | the ABI the Stylus contract exposes (`cargo stylus export-abi`) |
| [`src/ICounter.sol`](src/ICounter.sol) | idem for the counter |

Hooks inherit `BaseHook` from [`@openzeppelin/uniswap-hooks`](https://github.com/OpenZeppelin/uniswap-hooks),
the base contract the official [v4-template](https://github.com/uniswapfoundation/v4-template) uses.

```bash
forge test
forge build
```

Tests deploy a full local v4 stack through [`hookmate`](https://github.com/akshatmittal/hookmate)
and swap through the hooks for real. Since forge cannot run WASM,
[`test/mocks/MockStylusAirdrop.sol`](test/mocks/MockStylusAirdrop.sol) stands in for the Rust
contract — it is the executable spec that `stylus/airdrop/src/lib.rs` must match.

## Scripts

| Script | Deploys |
| --- | --- |
| `00_DeployAirdropHook.s.sol` | the pure-Solidity `AirdropHook` |
| `01_DeployStylusAirdropHook.s.sol` | `AirdropHookProxy` in front of `$STYLUS_AIRDROP`, then binds them |
| `02_DeployStylusCounterHook.s.sol` | `CounterProxy` in front of `$STYLUS_COUNTER`, then binds them |
