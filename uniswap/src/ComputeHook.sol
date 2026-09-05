// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "@uniswap/v4-core/src/types/BeforeSwapDelta.sol";

/// @title ComputeHook
/// @notice A hook that does nothing but arithmetic, with a dial for how much of it.
///
/// It exists to find the point where a hook is better off in Stylus. Stylus buys a lower marginal
/// cost of computation at the price of a fixed cost per call; below some amount of work the fixed
/// cost wins and Solidity is cheaper. `stylus/native-compute` runs the identical loop in Rust, and
/// `bench-compute.bash` sweeps `rounds` across both to find where the lines cross.
///
/// @dev The loop is xorshift64, chosen because it is a handful of shifts and xors per round with no
///      memory traffic and no storage — the closest thing to measuring raw compute through a hook.
///      Each language uses the word size it is good at, which is the honest comparison: the EVM has
///      no cheaper option than its 256-bit word, and WASM has no 256-bit word at all.
contract ComputeHook is BaseHook {
    uint256 public rounds;
    uint64 public lastResult;

    constructor(IPoolManager _poolManager) BaseHook(_poolManager) {}

    function getHookPermissions() public pure override returns (Hooks.Permissions memory) {
        return Hooks.Permissions({
            beforeInitialize: false,
            afterInitialize: false,
            beforeAddLiquidity: false,
            afterAddLiquidity: false,
            beforeRemoveLiquidity: false,
            afterRemoveLiquidity: false,
            beforeSwap: true,
            afterSwap: false,
            beforeDonate: false,
            afterDonate: false,
            beforeSwapReturnDelta: false,
            afterSwapReturnDelta: false,
            afterAddLiquidityReturnDelta: false,
            afterRemoveLiquidityReturnDelta: false
        });
    }

    function setRounds(uint256 newRounds) external {
        rounds = newRounds;
    }

    /// @notice xorshift64, `n` times. Must agree with the Rust implementation.
    function work(uint256 n) public pure returns (uint64) {
        uint64 x = 0x9E3779B97F4A7C15;
        for (uint256 i = 0; i < n; i++) {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
        }
        return x;
    }

    function _beforeSwap(address, PoolKey calldata, SwapParams calldata, bytes calldata)
        internal
        override
        returns (bytes4, BeforeSwapDelta, uint24)
    {
        lastResult = work(rounds);
        return (BaseHook.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }
}
