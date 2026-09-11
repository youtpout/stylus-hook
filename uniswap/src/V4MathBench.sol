// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {FullMath} from "@uniswap/v4-core/src/libraries/FullMath.sol";
import {SqrtPriceMath} from "@uniswap/v4-core/src/libraries/SqrtPriceMath.sol";
import {SwapMath} from "@uniswap/v4-core/src/libraries/SwapMath.sol";
import {TickMath} from "@uniswap/v4-core/src/libraries/TickMath.sol";
import {TickBitmap} from "@uniswap/v4-core/src/libraries/TickBitmap.sol";

/// @title The Solidity control for `stylus/native-v4-math`
/// @notice Every call here goes straight into v4-core's own libraries. Nothing is reimplemented, so
/// this side of the benchmark is Uniswap's production code and the Rust side is a port of it tested
/// against Uniswap's own vectors. What is measured is the language.
contract V4MathBench {
    /// A no-op, to subtract the cost of being called at all.
    function baseline(uint256) external pure returns (uint256) {
        return 0;
    }

    // --- the primitives, priced one at a time ---------------------------------------------------

    function mulDiv(uint256 a, uint256 b, uint256 denominator) external pure returns (uint256) {
        return FullMath.mulDiv(a, b, denominator);
    }

    function mulDivRoundingUp(uint256 a, uint256 b, uint256 denominator) external pure returns (uint256) {
        return FullMath.mulDivRoundingUp(a, b, denominator);
    }

    function getSqrtPriceAtTick(int24 tick) external pure returns (uint160) {
        return TickMath.getSqrtPriceAtTick(tick);
    }

    function getTickAtSqrtPrice(uint160 sqrtPriceX96) external pure returns (int24) {
        return TickMath.getTickAtSqrtPrice(sqrtPriceX96);
    }

    function getAmount0Delta(uint160 sqrtPriceAX96, uint160 sqrtPriceBX96, uint128 liquidity, bool roundUp)
        external
        pure
        returns (uint256)
    {
        return SqrtPriceMath.getAmount0Delta(sqrtPriceAX96, sqrtPriceBX96, liquidity, roundUp);
    }

    function getAmount1Delta(uint160 sqrtPriceAX96, uint160 sqrtPriceBX96, uint128 liquidity, bool roundUp)
        external
        pure
        returns (uint256)
    {
        return SqrtPriceMath.getAmount1Delta(sqrtPriceAX96, sqrtPriceBX96, liquidity, roundUp);
    }

    function computeSwapStep(
        uint160 sqrtPriceCurrentX96,
        uint160 sqrtPriceTargetX96,
        uint128 liquidity,
        int256 amountRemaining,
        uint24 feePips
    ) external pure returns (uint160, uint256, uint256, uint256) {
        return SwapMath.computeSwapStep(sqrtPriceCurrentX96, sqrtPriceTargetX96, liquidity, amountRemaining, feePips);
    }

    // --- the workload --------------------------------------------------------------------------

    /// @dev The walk's state and its fixed parameters, both in memory. Solidity runs out of stack
    /// slots with them as locals, and the repository's other measurements are all taken with
    /// `via_ir = false` -- moving this one contract to the IR pipeline would make it incomparable.
    struct Walk {
        uint160 sqrtPrice;
        int24 tick;
        bool zeroForOne;
        int256 remaining;
        uint256 totalIn;
        uint256 totalOut;
        uint256 totalFee;
    }

    struct Cfg {
        uint160 limit;
        int24 tickSpacing;
        uint128 liquidity;
        uint24 feePips;
        uint256 maxSteps;
    }

    /// @notice `Pool.swap`'s loop, without the storage.
    /// @dev Every multiple of `tickSpacing` is taken as initialised, which is the dense case and so
    /// the honest upper bound on how many steps a swap takes. The tick arithmetic is v4-core's own
    /// `TickBitmap.compress`, and the cross-or-locate branch at the end is `Pool.swap`'s.
    function walkSwap(
        uint160 startSqrtPriceX96,
        uint160 sqrtPriceLimitX96,
        int24 tickSpacing,
        uint128 liquidity,
        int256 amountSpecified,
        uint24 feePips,
        uint256 maxSteps
    ) external pure returns (uint160, uint256, uint256, uint256) {
        Walk memory w = Walk({
            sqrtPrice: startSqrtPriceX96,
            tick: TickMath.getTickAtSqrtPrice(startSqrtPriceX96),
            zeroForOne: sqrtPriceLimitX96 < startSqrtPriceX96,
            remaining: amountSpecified,
            totalIn: 0,
            totalOut: 0,
            totalFee: 0
        });
        Cfg memory c = Cfg({
            limit: sqrtPriceLimitX96,
            tickSpacing: tickSpacing,
            liquidity: liquidity,
            feePips: feePips,
            maxSteps: maxSteps
        });
        _walk(w, c);
        return (w.sqrtPrice, w.totalIn, w.totalOut, w.totalFee);
    }

    function _walk(Walk memory w, Cfg memory c) private pure {
        for (uint256 steps = 0; steps < c.maxSteps; steps++) {
            if (w.remaining == 0 || w.sqrtPrice == c.limit) break;

            int24 compressed = TickBitmap.compress(w.tick, c.tickSpacing);
            int24 nextTick = w.zeroForOne ? (compressed - 1) * c.tickSpacing : (compressed + 1) * c.tickSpacing;
            if (nextTick < TickMath.MIN_TICK || nextTick > TickMath.MAX_TICK) break;

            uint160 nextPrice = TickMath.getSqrtPriceAtTick(nextTick);

            (uint160 next, uint256 amountIn, uint256 amountOut, uint256 feeAmount) = SwapMath.computeSwapStep(
                w.sqrtPrice,
                SwapMath.getSqrtPriceTarget(w.zeroForOne, nextPrice, c.limit),
                c.liquidity,
                w.remaining,
                c.feePips
            );

            if (w.remaining < 0) w.remaining += int256(amountIn + feeAmount);
            else w.remaining -= int256(amountOut);
            w.totalIn += amountIn;
            w.totalOut += amountOut;
            w.totalFee += feeAmount;

            if (next == nextPrice) w.tick = nextTick;
            else if (next != w.sqrtPrice) w.tick = TickMath.getTickAtSqrtPrice(next);
            w.sqrtPrice = next;
        }
    }

    /// @dev The router's state, in memory, for the same stack reason as `Walk`.
    struct Route {
        uint256 chunk;
        uint24 feePips;
        int24 tickSpacing;
        uint256 rounds;
        uint256 maxSteps;
    }

    /// @notice Split an exact-input swap across `startPrices.length` pools, by greedy marginal
    /// allocation.
    /// @dev This is what a router does off-chain today, and the reason it is done off-chain is that
    /// evaluating the alternatives costs more than the swap. Each round walks every pool once to
    /// price one more chunk and gives it to the best, so the cost is `rounds * pools` replays.
    function routeExactIn(
        uint160[] calldata startPrices,
        uint128[] calldata liquidities,
        int24 tickSpacing,
        uint256 amountIn,
        uint24 feePips,
        uint256 rounds,
        uint256 maxSteps
    ) external pure returns (uint256 totalOut, uint256[] memory allocated) {
        allocated = new uint256[](startPrices.length);
        if (startPrices.length == 0 || liquidities.length != startPrices.length || rounds == 0) {
            return (0, allocated);
        }
        return _route(
            startPrices,
            liquidities,
            allocated,
            Route({
                chunk: amountIn / rounds,
                feePips: feePips,
                tickSpacing: tickSpacing,
                rounds: rounds,
                maxSteps: maxSteps
            })
        );
    }

    /// @dev The winner of one round, boxed so `_route` does not have to hold three more slots.
    struct Best {
        uint256 pool;
        uint256 gain;
        uint256 out;
    }

    function _route(
        uint160[] calldata startPrices,
        uint128[] calldata liquidities,
        uint256[] memory allocated,
        Route memory cfg
    ) private pure returns (uint256 totalOut, uint256[] memory) {
        uint256[] memory currentOut = new uint256[](startPrices.length);
        unchecked {
            for (uint256 r = 0; r < cfg.rounds; r++) {
                Best memory b = _bestPool(startPrices, liquidities, allocated, currentOut, cfg);
                allocated[b.pool] += cfg.chunk;
                currentOut[b.pool] = b.out;
            }
            for (uint256 p = 0; p < currentOut.length; p++) {
                totalOut += currentOut[p];
            }
        }
        return (totalOut, allocated);
    }

    /// @dev Price one more chunk into every pool and return whichever gains the most by it.
    function _bestPool(
        uint160[] calldata startPrices,
        uint128[] calldata liquidities,
        uint256[] memory allocated,
        uint256[] memory currentOut,
        Route memory cfg
    ) private pure returns (Best memory b) {
        unchecked {
            for (uint256 p = 0; p < startPrices.length; p++) {
                uint256 out = _routeProbe(startPrices[p], liquidities[p], allocated[p] + cfg.chunk, cfg);
                uint256 gain = out > currentOut[p] ? out - currentOut[p] : 0;
                if (gain > b.gain) {
                    b.pool = p;
                    b.gain = gain;
                    b.out = out;
                }
            }
        }
    }

    function _routeProbe(uint160 price, uint128 liquidity, uint256 amount, Route memory cfg)
        private
        pure
        returns (uint256)
    {
        Walk memory w = Walk({
            sqrtPrice: price,
            tick: TickMath.getTickAtSqrtPrice(price),
            zeroForOne: true,
            remaining: -int256(amount),
            totalIn: 0,
            totalOut: 0,
            totalFee: 0
        });
        Cfg memory c = Cfg({
            limit: TickMath.MIN_SQRT_PRICE + 1,
            tickSpacing: cfg.tickSpacing,
            liquidity: liquidity,
            feePips: cfg.feePips,
            maxSteps: cfg.maxSteps
        });
        _walk(w, c);
        return w.totalOut;
    }

    // --- dials, to attribute the result to its parts --------------------------------------------

    function swapStepLoop(
        uint256 n,
        uint160 sqrtPriceCurrentX96,
        uint160 sqrtPriceTargetX96,
        uint128 liquidity,
        int256 amountRemaining,
        uint24 feePips
    ) external pure returns (uint256 acc) {
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                (, uint256 amountIn, uint256 amountOut,) = SwapMath.computeSwapStep(
                    sqrtPriceCurrentX96, sqrtPriceTargetX96, liquidity, amountRemaining, feePips
                );
                acc += amountIn + amountOut;
            }
        }
    }

    function sqrtPriceAtTickLoop(uint256 n, int24 startTick) external pure returns (uint256 acc) {
        unchecked {
            int24 tick = startTick;
            for (uint256 i = 0; i < n; i++) {
                acc += TickMath.getSqrtPriceAtTick(tick);
                tick++;
            }
        }
    }

    function tickAtSqrtPriceLoop(uint256 n, uint160 startPriceX96) external pure returns (int256 acc) {
        unchecked {
            uint160 price = startPriceX96;
            for (uint256 i = 0; i < n; i++) {
                acc += TickMath.getTickAtSqrtPrice(price);
                price += 1e15;
            }
        }
    }
}
