// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {console} from "forge-std/console.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {TwammHook} from "../src/TwammHook.sol";

contract TwammHookTest is Test {
    TwammHook internal hook;

    function setUp() public {
        hook = TwammHook(address(uint160(Hooks.BEFORE_SWAP_FLAG) ^ (0x4444 << 144)));
        deployCodeTo("TwammHook.sol:TwammHook", abi.encode(IPoolManager(address(0))), address(hook));
    }

    function test_reference_values() public view {
        for (uint256 n = 0; n <= 3; n++) {
            console.log(n, hook.work(n));
        }
    }

    /// @notice The curve has to move in the right direction: rate1 outsells rate0, so the pool
    ///         price walks up towards sqrt(rate1/rate0).
    function test_price_walks_towards_the_sell_ratio() public view {
        uint256 one = hook.work(1);
        uint256 three = hook.work(3);
        assertGt(one, 1e18, "price should rise");
        assertGt(three, one, "and keep rising over more intervals");
        assertLt(three, 2e18, "but stay bounded by sqrt(5/3) ~ 1.29 .. sqrt(5/3.2)");
    }
}
