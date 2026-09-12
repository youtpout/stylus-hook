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
/// @notice A hook that does nothing but arithmetic, with a dial for how much of it, to find where
///         a hook is better off in Stylus. `stylus/native-compute` runs the identical loop.
/// @dev xorshift64: a few shifts and xors per round, no memory traffic and no storage. Each side
///      uses the word size it is good at, which is the comparison that matters.
contract ComputeHook is BaseHook {
    uint256 public rounds;
    uint64 public lastResult;

    /// @notice 0 xorshift64, 1 `mulDiv`, 2 storage writes, 3 `rpow`, 4 integer `sqrt`.
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

    /// @notice Fixed-point exponentiation by squaring, as `FixedPointMathLib.rpow` in solady.
    /// @dev This is what Bunni's Liquidity Density Functions run on every swap:
    ///      `LibGeometricDistribution` calls `alphaX96.rpow(length, Q96)` several times per LDF
    ///      query. One `rpow` is a chain of full-precision `mulDiv`s, so it is the natural unit for
    ///      pricing a hook that replaces the AMM curve with its own math.
    function rpow(uint256 x, uint256 n, uint256 precision) public pure returns (uint256 z) {
        z = n % 2 != 0 ? x : precision;
        for (n /= 2; n != 0; n /= 2) {
            x = FullMath.mulDiv(x, x, precision);
            if (n % 2 != 0) z = FullMath.mulDiv(z, x, precision);
        }
    }

    /// @notice `n` LDF-sized `rpow` calls. Must agree with the Rust twin.
    function workRpow(uint256 n) public pure returns (uint256) {
        uint256 q96 = 1 << 96;
        uint256 alpha = (q96 / 100) * 99; // a plausible LDF alpha, just under 1
        uint256 acc = q96;
        for (uint256 i = 0; i < n; i++) {
            acc = rpow(alpha + i, 100, q96);
        }
        return acc;
    }

    /// @notice Integer square root, Babylonian with a bit-length seed — solady's shape, and
    ///         EulerSwap's `Sqrt.sol`.
    /// @dev EulerSwap's curve inversion takes one per swap over a 255-bit discriminant. The seed
    ///      needs a bit length, which the EVM has no opcode for and WASM has as `i64.clz`.
    function isqrt(uint256 x) public pure returns (uint256 z) {
        if (x == 0) return 0;
        uint256 r = 1;
        uint256 v = x;
        if (v >= 1 << 128) { v >>= 128; r <<= 64; }
        if (v >= 1 << 64) { v >>= 64; r <<= 32; }
        if (v >= 1 << 32) { v >>= 32; r <<= 16; }
        if (v >= 1 << 16) { v >>= 16; r <<= 8; }
        if (v >= 1 << 8) { v >>= 8; r <<= 4; }
        if (v >= 1 << 4) { v >>= 4; r <<= 2; }
        if (v >= 1 << 2) { r <<= 1; }
        // seven Newton steps are enough for 256 bits
        z = r;
        for (uint256 i = 0; i < 7; i++) {
            z = (z + x / z) >> 1;
        }
        uint256 zz = x / z;
        return z <= zz ? z : zz;
    }

    /// @notice `n` square roots of full-range values. Must agree with the Rust twin.
    function workSqrt(uint256 n) public pure returns (uint256) {
        uint256 acc = 0;
        uint256 seed = 0x9E3779B97F4A7C15C2B2AE3D27D4EB4F165667B19E3779F9165667B19E3779F9;
        for (uint256 i = 0; i < n; i++) {
            acc = isqrt(seed - i);
        }
        return acc;
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
        } else if (m == 2) {
            lastResult = uint64(workStorage(rounds));
        } else if (m == 3) {
            lastResult = uint64(workRpow(rounds));
        } else {
            lastResult = uint64(workSqrt(rounds));
        }
        return (BaseHook.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }
}
