// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "@uniswap/v4-core/src/types/BeforeSwapDelta.sol";
import {FullMath} from "@uniswap/v4-core/src/libraries/FullMath.sol";

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

    /// @notice 0 runs xorshift64, 1 runs `FullMath.mulDiv`, 2 writes storage slots.
    uint8 public mode;

    /// @dev Written by mode 2. A mapping rather than an array because that is what hooks use, so
    ///      each access includes hashing the key as well as the `SSTORE` itself.
    mapping(uint256 slot => uint256 value) private slots;

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

    function setMode(uint8 newMode) external {
        mode = newMode;
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

    /// @notice `FullMath.mulDiv` on 256-bit words, `n` times. Must agree with the Rust twin.
    /// @dev This is the atom Uniswap's own swap math is built from: `computeSwapStep`,
    ///      `SqrtPriceMath` and every tick-walking simulation are mostly chains of mulDiv. A single
    ///      256-bit multiply and divide is one EVM opcode each; in WASM it is limb arithmetic. This
    ///      is the case that decides whether porting real AMM math to Stylus is worth anything.
    function workMulDiv(uint256 n) public pure returns (uint256) {
        uint256 a = 0x9E3779B97F4A7C15C2B2AE3D27D4EB4F165667B19E3779F9165667B19E3779F9;
        uint256 b = type(uint128).max; // one ulp below the denominator, so `a` stays 256-bit wide
        uint256 d = 1 << 128;
        for (uint256 i = 0; i < n; i++) {
            a = FullMath.mulDiv(a | (1 << 249), b, d) + i + 1;
        }
        return a;
    }

    /// @notice Writes `n` storage slots. Must agree with the Rust twin.
    /// @dev Storage is the one thing Stylus is not supposed to make cheaper: `SLOAD` and `SSTORE`
    ///      are host operations priced in EVM gas whichever VM runs the contract. This mode is here
    ///      to check that rather than assume it.
    function workStorage(uint256 n) public returns (uint256) {
        uint256 last;
        for (uint256 i = 0; i < n; i++) {
            last = i + 1;
            slots[i] = last;
        }
        return last;
    }

    function readStorage(uint256 key) external view returns (uint256) {
        return slots[key];
    }

    function _beforeSwap(address, PoolKey calldata, SwapParams calldata, bytes calldata)
        internal
        override
        returns (bytes4, BeforeSwapDelta, uint24)
    {
        uint8 m = mode;
        if (m == 0) {
            lastResult = work(rounds);
        } else if (m == 1) {
            lastResult = uint64(workMulDiv(rounds));
        } else {
            lastResult = uint64(workStorage(rounds));
        }
        return (BaseHook.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }
}
