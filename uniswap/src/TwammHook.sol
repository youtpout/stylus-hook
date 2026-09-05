// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "@uniswap/v4-core/src/types/BeforeSwapDelta.sol";

import {ABDKMathQuad} from "./vendor/ABDKMathQuad.sol";

/// @title TwammHook
/// @notice The arithmetic of a TWAMM interval, priced as a v4 hook. The Solidity twin of
///         `stylus/native-twamm`.
///
/// @dev Uniswap shipped a TWAMM hook as a v4-periphery example and recorded its cost: 489,927 gas
///      for one interval and about 100,000 for each one after. That is the only workload in
///      BENCHMARK.md above the crossover where a Stylus port pays for itself, and the reason is
///      what the arithmetic is made of — IEEE 754 quadruple-precision floating point, emulated in
///      software because the EVM has none.
///
///      The closed form below is the published TWAMM solution (Paradigm, 2021), implemented here
///      from the formula. Uniswap's own implementation is GPL-2.0 and its TwammMath is marked
///      UNLICENSED, so none of it is reproduced; only ABDK's float library is vendored, under its
///      own BSD-4-Clause terms.
///
///        sqrtSellRate  = sqrt(rate0 * rate1)
///        sqrtSellRatio = sqrt(rate1 / rate0)
///        pow           = 2 * sqrtSellRate * elapsed / liquidity
///        c             = (sqrtSellRatio - sqrtPrice) / (sqrtSellRatio + sqrtPrice)
///        newSqrtPrice  = sqrtSellRatio * (e^pow - c) / (e^pow + c)
contract TwammHook is BaseHook {
    using ABDKMathQuad for bytes16;
    using ABDKMathQuad for uint256;

    /// @notice How many intervals a swap advances through. The dial the benchmark sweeps.
    uint256 public intervals = 1;

    uint256 public lastSqrtPrice;

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

    function setIntervals(uint256 n) external {
        intervals = n;
    }

    /// @notice One TWAMM interval: the closed-form price the pool arrives at after `elapsed`
    ///         seconds of two order pools selling into each other against `liquidity`.
    /// @dev Scaled by 1e18 rather than Q96 so the Rust twin can hold the same numbers in an f64.
    function newSqrtPrice(
        uint256 sqrtPriceE18,
        uint256 liquidity,
        uint256 rate0,
        uint256 rate1,
        uint256 elapsed
    ) public pure returns (uint256) {
        bytes16 one = uint256(1e18).fromUInt();
        bytes16 r0 = rate0.fromUInt();
        bytes16 r1 = rate1.fromUInt();

        bytes16 sqrtSellRate = r0.mul(r1).sqrt();
        bytes16 sqrtSellRatio = r1.div(r0).sqrt();
        bytes16 sqrtPrice = sqrtPriceE18.fromUInt().div(one);

        bytes16 pow = uint256(2).fromUInt().mul(sqrtSellRate).mul(elapsed.fromUInt()).div(liquidity.fromUInt());
        bytes16 c = sqrtSellRatio.sub(sqrtPrice).div(sqrtSellRatio.add(sqrtPrice));
        bytes16 ePow = pow.exp();

        bytes16 next = sqrtSellRatio.mul(ePow.sub(c)).div(ePow.add(c));
        return next.mul(one).toUInt();
    }

    /// @notice `n` intervals, each feeding its price into the next. Must agree with the Rust twin.
    function work(uint256 n) public pure returns (uint256 price) {
        price = 1e18;
        for (uint256 i = 0; i < n; i++) {
            price = newSqrtPrice(price, 1_000_000e18, 3e18 + i * 1e17, 5e18, 600);
        }
    }

    function _beforeSwap(address, PoolKey calldata, SwapParams calldata, bytes calldata)
        internal
        override
        returns (bytes4, BeforeSwapDelta, uint24)
    {
        lastSqrtPrice = work(intervals);
        return (BaseHook.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }
}
