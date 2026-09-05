// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {StableSwapHook} from "../src/StableSwapHook.sol";

/// @notice The curve has to agree with `stylus/native-stableswap` or the benchmark compares two
///         different AMMs. These are the same literals the Rust tests assert.
contract StableSwapHookTest is Test {
    StableSwapHook internal hook;

    function setUp() public {
        hook = StableSwapHook(address(uint160(Hooks.BEFORE_SWAP_FLAG) ^ (0x4444 << 144)));
        deployCodeTo("StableSwapHook.sol:StableSwapHook", abi.encode(IPoolManager(address(0))), address(hook));
    }

    function test_curve_matches_the_rust_twin() public view {
        assertEq(hook.getD(1000e18, 600e18, 100), 1598956273533045081014);
        assertEq(hook.getY(1e18, 1000e18, 600e18, 100), 599011076353954654953);
    }

    function test_a_stable_curve_barely_slips() public view {
        uint256 out = hook.quote(1e18);
        assertGt(out, 0.98e18, "a stable curve should barely slip");
        assertLt(out, 1e18);
    }
}
