// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {ComputeHook} from "../src/ComputeHook.sol";

/// @notice The Solidity half of the compute benchmark has to run the *same* loop as
///         `stylus/native-compute`, or the two sides of the crossover measurement are not
///         comparable. These are the same literals the Rust test asserts.
contract ComputeHookTest is Test {
    ComputeHook internal hook;

    function setUp() public {
        // BaseHook validates its own address, so etch it at one carrying BEFORE_SWAP_FLAG
        hook = ComputeHook(address(uint160(Hooks.BEFORE_SWAP_FLAG) ^ (0x4444 << 144)));
        deployCodeTo("ComputeHook.sol:ComputeHook", abi.encode(IPoolManager(address(0))), address(hook));
    }

    function test_mulDiv_matches_the_rust_twin() public view {
        assertEq(hook.workMulDiv(0), 0x9e3779b97f4a7c15c2b2ae3d27d4eb4f165667b19e3779f9165667b19e3779f9);
        assertEq(hook.workMulDiv(1), 0x9e3779b97f4a7c15c2b2ae3d27d4eb4e781eedf81eecfde353a3b97476628eaa);
        assertEq(hook.workMulDiv(100), 0x9e3779b97f4a7c15c2b2ae3d27d4eb1148aadb3be51f0179088a57ce0f0bae90);
    }

    function test_sqrt_matches_the_rust_twin() public view {
        assertEq(hook.workSqrt(0), 0);
        assertEq(hook.workSqrt(1), 267513451581452811726538898354923035252);
        assertEq(hook.workSqrt(10), 267513451581452811726538898354923035252);
    }

    function test_xorshift_matches_the_rust_twin() public view {
        assertEq(hook.work(0), 0x9e3779b97f4a7c15);
        assertEq(hook.work(1), 0xdc1b77ae0bf34dad);
        assertEq(hook.work(10), 0x8f8ea9d349428d8e);
        assertEq(hook.work(100), 0xab5917a81f0fb2ae);
    }
}
