// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Gaussian} from "solstat/Gaussian.sol";
import {FixedPointMathLib} from "solstat/../lib/solmate/src/utils/FixedPointMathLib.sol";

/// @title PmAmmMath
/// @notice The Solidity side of the pm-AMM arithmetic benchmark. Exposes `solstat`'s Gaussian and a
///         Newton solve of the pm-AMM invariant, with the same ABI as `stylus/native-gaussian`.
///
/// @dev Why this workload. Every hook benchmarked in this repository loses to its Solidity twin,
///      because Stylus trades a lower marginal cost of computation for a fixed cost per call. TWAMM
///      looked like the exception until the version actually deployed turned out to use plain
///      integer `mulDiv`. The pm-AMM is a better bet for one reason: its arithmetic cannot be
///      removed. With `z = (y - x) / L`, Paradigm's invariant is
///
///        f(y)  = (y - x)·Phi(z) + L·phi(z) - y
///        f'(y) = Phi(z) - 1
///
///      which is transcendental in `y`. No closed form exists, so a solve is mandatory, and every
///      iteration needs the Gaussian CDF and PDF. The derivative collapsing to `Phi(z) - 1` is a
///      gift — the two `z·phi(z)` terms cancel — but a full Gaussian evaluation is still needed per
///      step.
///
///      `Gnome101/Pm-AMM-Hook`, the one v4 hook that implements a pm-AMM, solves this by 100-step
///      bisection: 200 Gaussian evaluations per swap. That is not what is measured here. A benchmark
///      against a lazy implementation measures nothing, and this repository has already made that
///      mistake once with TWAMM. Newton is the honest floor.
///
///      `solstat` is referenced as a submodule rather than vendored: its files carry `SPDX: MIT` but
///      the repository ships an AGPL-3.0 `LICENSE`, and that contradiction is not one to inherit.
///      Its `erfc` is the Chebyshev fit from Numerical Recipes 3e p265, over Solmate's `expWad`.
contract PmAmmMath {
    using FixedPointMathLib for int256;

    int256 private constant ONE = 1e18;

    error ZeroDerivative();
    error ZeroLiquidity();

    /// @notice A no-op, to subtract the cost of being called at all.
    /// @dev `expWad(0)` and `pdf(0)` do not short-circuit, so they cannot serve as their own
    ///      baseline; only `erfc(0)` does.
    function baseline(int256) external pure returns (int256) {
        return 0;
    }

    /// @notice `n` iterations of `a * b / c`, wrapping, with the operand magnitude left to the
    ///         caller.
    /// @dev The EVM charges 5 gas for `MUL` and 5 for `DIV` whatever the operands are. A 256-bit
    ///      integer in WASM is four 64-bit limbs, and `ruint` short-circuits on the ones that are
    ///      zero — so the same expression should cost Rust much more when the operands are wide than
    ///      when they are narrow. This is the dial to test that with.
    function mulDivLoop(uint256 n, uint256 a, uint256 b, uint256 c) external pure returns (uint256 acc) {
        unchecked {
            for (uint256 i = 0; i < n; ++i) {
                acc += (a * b) / c;
            }
        }
    }

    // --- the primitives, priced one at a time ---------------------------------------------------

    function expWad(int256 x) external pure returns (int256) {
        return x.expWad();
    }

    function erfc(int256 x) external pure returns (int256) {
        return Gaussian.erfc(x);
    }

    function cdf(int256 x) external pure returns (int256) {
        return Gaussian.cdf(x);
    }

    function pdf(int256 x) external pure returns (int256) {
        return Gaussian.pdf(x);
    }

    /// @notice The invariant and its slope at `y` — what one Newton step costs.
    /// @dev Named `residual` rather than `invariant` because forge claims any function whose name
    ///      starts with `invariant` as an invariant test, and then rejects it for taking parameters.
    function residual(int256 L, int256 x, int256 y) public pure returns (int256 f, int256 df) {
        int256 z = ((y - x) * ONE) / L;
        int256 c = Gaussian.cdf(z);
        int256 p = Gaussian.pdf(z);
        f = ((y - x) * c) / ONE + (L * p) / ONE - y;
        df = c - ONE;
    }

    /// @notice Solves the invariant for `y` by Newton, with a fixed iteration count so the gas is
    ///         not input-dependent.
    function solveNewton(int256 L, int256 x, int256 y0, uint256 iters) public pure returns (int256 y) {
        if (L == 0) revert ZeroLiquidity();
        y = y0;
        for (uint256 i = 0; i < iters; ++i) {
            (int256 f, int256 df) = residual(L, x, y);
            if (df == 0) revert ZeroDerivative();
            y -= (f * ONE) / df;
        }
    }

    /// @notice `n` Newton iterations from a fixed start. The dial the benchmark sweeps, and the same
    ///         function `stylus/native-gaussian` exposes.
    function work(uint256 n) external pure returns (int256) {
        return solveNewton(100e18, 40e18, 60e18, n);
    }
}
