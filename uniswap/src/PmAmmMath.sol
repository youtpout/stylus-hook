// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Gaussian} from "solstat/Gaussian.sol";
import {FixedPointMathLib} from "solstat/../lib/solmate/src/utils/FixedPointMathLib.sol";

/// @title PmAmmMath
/// @notice The Solidity side of the pm-AMM arithmetic benchmark: solstat's Gaussian and a Newton
///         solve of the invariant, with the same ABI as `stylus/native-gaussian`.
/// @dev The invariant is transcendental in `y`, so a solve is mandatory and every iteration needs
///      the Gaussian CDF and PDF. Newton rather than bisection, which is the honest floor.
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
    /// @dev The EVM charges 5 gas for `MUL` and `DIV` whatever the operands are, while `ruint`
    ///      short-circuits on zero limbs — so this is the dial for testing operand width.
    function mulDivLoop(uint256 n, uint256 a, uint256 b, uint256 c) external pure returns (uint256 acc) {
        unchecked {
            for (uint256 i = 0; i < n; ++i) {
                acc += (a * b) / c;
            }
        }
    }

    /// @notice `n` modular multiplications in a ~254-bit prime field — BN254's scalar field, the one
    ///         Poseidon and every BN254 SNARK verifier work in.
    /// @dev The EVM has `MULMOD` as a single opcode at 8 gas. There is no equivalent in WASM: a
    ///      256-bit modular multiply there means a 512-bit product and a reduction, by hand. This is
    ///      the dial that says whether porting field arithmetic to Stylus is worth anything.
    function mulmodLoop(uint256 n, uint256 a, uint256 b, uint256 m) external pure returns (uint256 acc) {
        unchecked {
            for (uint256 i = 0; i < n; ++i) {
                acc = mulmod(acc + a, b, m);
            }
        }
    }

    /// @notice The same, in the 64-bit Goldilocks field `2^64 - 2^32 + 1` that Plonky2, Plonky3 and
    ///         Risc0 verify over.
    /// @dev Here the EVM still pays for a 256-bit word it cannot use, and Solidity cannot reach a
    ///      64-bit multiply at all — it has to widen. A `u64` is one WASM register.
    function goldilocksLoop(uint256 n, uint256 a, uint256 b) external pure returns (uint256 acc) {
        uint256 P = 0xFFFFFFFF00000001;
        unchecked {
            acc = a;
            for (uint256 i = 0; i < n; ++i) {
                acc = mulmod(acc + b, b, P);
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
