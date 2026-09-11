// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {PmAmmMath} from "../src/PmAmmMath.sol";

/// @notice Pins the Solidity side of the pm-AMM benchmark to the same values the Rust port asserts.
///
/// @dev The comparison in `bench-pmamm.bash` is only a language comparison if both sides compute the
///      same numbers by the same steps. `stylus/native-gaussian` has this exact table in its own
///      tests; if either drifts, one of the two suites fails and the benchmark stops meaning
///      anything before it is published.
contract PmAmmMathTest is Test {
    PmAmmMath internal math;

    function setUp() public {
        math = new PmAmmMath();
    }

    function test_gaussian_matches_the_rust_port() public view {
        int256[9] memory xs = [-3e18, -1e18, -4e17, -1, int256(0), 1, 4e17, 1e18, 3e18];
        int256[9] memory expWads = [
            int256(49787068367863942),
            367879441171442321,
            670320046035639300,
            999999999999999999,
            1000000000000000000,
            1000000000000000001,
            1491824697641270317,
            2718281828459045235,
            20085536923187667741
        ];
        int256[9] memory erfcs = [
            int256(1999977909501578499),
            1842700787760006725,
            1428392402898957764,
            999999969999999550,
            int256(1000000000000000000),
            1000000030000000450,
            571607597101042236,
            157299212239993275,
            22090498421501
        ];
        int256[9] memory cdfs = [
            int256(1349898073255602),
            158655261395674625,
            344578250551125821,
            500000000000000000,
            500000000000000000,
            500000000000000000,
            655421749448874179,
            841344738604325374,
            998650101926744398
        ];
        int256[9] memory pdfs = [
            int256(4431848411938006),
            241970724519143349,
            368270140303323307,
            398942280401432678,
            398942280401432678,
            398942280401432678,
            368270140303323307,
            241970724519143349,
            4431848411938006
        ];
        for (uint256 i = 0; i < xs.length; i++) {
            assertEq(math.expWad(xs[i]), expWads[i], "expWad");
            assertEq(math.erfc(xs[i]), erfcs[i], "erfc");
            assertEq(math.cdf(xs[i]), cdfs[i], "cdf");
            assertEq(math.pdf(xs[i]), pdfs[i], "pdf");
        }
    }

    /// The `2^k` range reduction inside `expWad` is where a port goes wrong.
    function test_range_reduction_at_the_extremes() public view {
        assertEq(math.expWad(20e18), 485165195409790277974777105);
        assertEq(math.expWad(-20e18), 2061153622);
        assertEq(math.expWad(100e18), 26881171418161354484134666106240937146178367581647816351662017);
        assertEq(math.expWad(-42139678854452767551), 0, "below the domain");
    }

    /// Gas figures for a solve that does not converge would mean nothing, so this checks it lands on
    /// an actual root — and on the same root the Rust port finds.
    function test_newton_converges_to_the_same_root() public view {
        int256 y = math.work(8);
        assertEq(y, 39788634305350540988, "same root as the Rust port");
        (int256 f,) = math.residual(100e18, 40e18, y);
        assertLt(f >= 0 ? f : -f, 1e6, "residual too large");
    }

    function test_zero_liquidity_is_refused() public {
        vm.expectRevert(PmAmmMath.ZeroLiquidity.selector);
        math.solveNewton(0, 1e18, 1e18, 1);
    }
}
