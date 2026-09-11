// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {V4MathBench} from "../src/V4MathBench.sol";

/// @notice Pins the swap walk, so the benchmark stays a language comparison.
///
/// @dev The primitives need no test here: `V4MathBench` calls v4-core's libraries directly, and
///      v4-core tests those. `walkSwap` is this repository's own composition of them, and the Rust
///      port has to agree with it step for step -- `stylus/native-v4-math` asserts this same table.
///      If either drifts, one of the two suites fails before any number is published.
contract V4MathBenchTest is Test {
    V4MathBench internal bench;

    uint160 internal constant P11 = 79228162514264337593543950336;
    uint160 internal constant MIN_P = 4295128739;

    function setUp() public {
        bench = new V4MathBench();
    }

    function _walk(int256 amount, uint256 maxSteps) internal view returns (uint160, uint256, uint256, uint256) {
        return bench.walkSwap(P11, MIN_P + 1, 60, 1e18, amount, 3000, maxSteps);
    }

    /// Eight ticks crossed, the step capped by the target price every time. Exact input and exact
    /// output land on the same numbers here, because the amount never binds -- which is the case the
    /// benchmark runs, so it is the one that has to be pinned.
    function test_walkSwap_eightTicks() public view {
        int256[2] memory amounts = [-1000e18, int256(1000e18)];
        for (uint256 i = 0; i < 2; i++) {
            (uint160 price, uint256 amountIn, uint256 amountOut, uint256 fee) = _walk(amounts[i], 8);
            assertEq(price, 77349415686109193705893883426, "price");
            assertEq(amountIn, 24289088824914538, "in");
            assertEq(amountOut, 23713118776633140, "out");
            assertEq(fee, 73086526052907, "fee");
        }
    }

    /// The step count has to be the only thing that changes, or the benchmark's dial is not a dial.
    function test_walkSwap_stepsAreMonotonic() public view {
        uint256 previousIn;
        for (uint256 n = 1; n <= 16; n++) {
            (, uint256 amountIn,,) = _walk(-1000e18, n);
            assertGt(amountIn, previousIn, "each step must consume more input");
            previousIn = amountIn;
        }
    }

    /// A walk that runs out of amount before it runs out of steps must stop, not spin. This is the
    /// branch where `remaining` binds and the two directions diverge.
    function test_walkSwap_stopsWhenTheAmountIsSpent() public view {
        (uint160 price, uint256 amountIn,, uint256 fee) = _walk(-1e15, 64);
        assertEq(amountIn + fee, 1e15, "the whole input is consumed");
        assertEq(price, 79149250711305166342700278159, "price");
        assertGt(price, MIN_P + 1, "and the limit was never reached");

        (uint160 outPrice,, uint256 amountOut,) = _walk(1e15, 64);
        assertEq(amountOut, 1e15, "the whole output is delivered");
        assertLt(outPrice, price, "exact output goes further for the same number");
    }

}
