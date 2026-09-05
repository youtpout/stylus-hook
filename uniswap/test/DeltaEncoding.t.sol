// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";

import {BalanceDelta, toBalanceDelta, BalanceDeltaLibrary} from "@uniswap/v4-core/src/types/BalanceDelta.sol";
import {BeforeSwapDelta, toBeforeSwapDelta, BeforeSwapDeltaLibrary} from
    "@uniswap/v4-core/src/types/BeforeSwapDelta.sol";
import {LPFeeLibrary} from "@uniswap/v4-core/src/libraries/LPFeeLibrary.sol";

/// @notice Reference values for the delta packing and fee flags that `stylus/base-hook`
///         reimplements in Rust. The same literals appear in `base-hook/src/types.rs`; if v4-core
///         ever changes the encoding, this test fails here and the Rust one fails there.
contract DeltaEncodingTest is Test {
    using BalanceDeltaLibrary for BalanceDelta;
    using BeforeSwapDeltaLibrary for BeforeSwapDelta;

    function _raw(BalanceDelta delta) private pure returns (bytes32) {
        return bytes32(uint256(BalanceDelta.unwrap(delta)));
    }

    function test_balance_delta_packing() public pure {
        assertEq(_raw(toBalanceDelta(-5, 7)), hex"fffffffffffffffffffffffffffffffb00000000000000000000000000000007");
        assertEq(
            _raw(toBalanceDelta(1e18, -2e18)),
            hex"00000000000000000de0b6b3a7640000ffffffffffffffffe43e9298b1380000"
        );
    }

    function test_balance_delta_round_trips() public pure {
        assertEq(toBalanceDelta(-5, 7).amount0(), -5);
        assertEq(toBalanceDelta(-5, 7).amount1(), 7);
        assertEq(toBalanceDelta(7, -5).amount0(), 7);
        assertEq(toBalanceDelta(7, -5).amount1(), -5);
        assertEq(toBalanceDelta(type(int128).min, type(int128).max).amount0(), type(int128).min);
        assertEq(toBalanceDelta(type(int128).min, type(int128).max).amount1(), type(int128).max);
    }

    function test_before_swap_delta_uses_the_same_packing() public pure {
        BeforeSwapDelta delta = toBeforeSwapDelta(-11, 13);
        assertEq(delta.getSpecifiedDelta(), -11);
        assertEq(delta.getUnspecifiedDelta(), 13);
        assertEq(BeforeSwapDelta.unwrap(delta), BalanceDelta.unwrap(toBalanceDelta(-11, 13)));
    }

    function test_lp_fee_flags() public pure {
        assertEq(LPFeeLibrary.DYNAMIC_FEE_FLAG, 0x800000);
        assertEq(LPFeeLibrary.OVERRIDE_FEE_FLAG, 0x400000);
        assertEq(LPFeeLibrary.MAX_LP_FEE, 1000000);
        assertTrue(LPFeeLibrary.isOverride(uint24(3000 | 0x400000)));
        assertEq(LPFeeLibrary.removeOverrideFlag(uint24(3000 | 0x400000)), 3000);
    }
}
