// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";

import {AirdropHook} from "../src/AirdropHook.sol";
import {AirdropToken} from "../src/AirdropToken.sol";
import {HookTest} from "./utils/HookTest.sol";

contract AirdropTest is HookTest {
    AirdropHook internal hook;
    AirdropToken internal airdrop;

    address internal alice = makeAddr("alice");
    address internal bob = makeAddr("bob");

    uint256 internal constant AIRDROP_SUPPLY = 50_000_000 ether;

    function setUp() public {
        deployStack();

        uint160 flags = uint160(Hooks.AFTER_SWAP_FLAG);
        hook = AirdropHook(deployHookTo(flags, "AirdropHook.sol:AirdropHook", abi.encode(poolManager)));

        createPoolAndAddLiquidity(IHooks(address(hook)), 100e18);

        airdrop = new AirdropToken("Flydrop", "FLY", address(this), address(hook), AIRDROP_SUPPLY);
    }

    /// @dev The router is the `sender` seen by the hook, so the real beneficiary travels in hookData.
    function _swapFor(address user, uint256 amountIn, bool zeroForOne) internal {
        swap(amountIn, zeroForOne, abi.encode(user));
    }

    function test_airdrop_accounting() public {
        assertEq(hook.totalUsers(poolId), 0);

        _swapFor(alice, 1e18, true);
        _swapFor(alice, 2e18, true);
        _swapFor(bob, 1e18, false);

        assertEq(hook.totalUsers(poolId), 2);

        // zeroForOne swaps are booked on the "1" side, the other way round on the "0" side
        (uint256 amount0, uint256 amount1, uint256 counter0, uint256 counter1) = hook.totalSwap(poolId);
        assertEq(amount1, 3e18);
        assertEq(counter1, 2);
        assertEq(amount0, 1e18);
        assertEq(counter0, 1);

        (,, , uint256 aliceCounter1) = hook.totalSwapUser(poolId, alice);
        assertEq(aliceCounter1, 2);
    }

    function test_airdrop_stops_accounting_once_closed() public {
        _swapFor(alice, 1e18, true);
        hook.closeAirdrop(poolId, address(airdrop));

        _swapFor(bob, 1e18, true);

        // bob swapped after the airdrop was closed, so he is not registered
        assertEq(hook.totalUsers(poolId), 1);
        assertEq(hook.amountToClaim(poolId, bob), 0);
    }

    function test_airdrop_claim() public {
        _swapFor(alice, 3e18, true);
        _swapFor(bob, 1e18, true);

        hook.closeAirdrop(poolId, address(airdrop));

        uint256 aliceShare = hook.amountToClaim(poolId, alice);
        uint256 bobShare = hook.amountToClaim(poolId, bob);
        assertGt(aliceShare, bobShare, "alice swapped 3x more volume");
        assertGt(bobShare, 0);

        vm.prank(alice);
        hook.claimAirdrop(poolId);
        assertEq(airdrop.balanceOf(alice), aliceShare);

        vm.prank(alice);
        vm.expectRevert(AirdropHook.AlreadyClaimed.selector);
        hook.claimAirdrop(poolId);
    }

    function test_airdrop_claim_reverts_before_close() public {
        _swapFor(alice, 1e18, true);

        vm.prank(alice);
        vm.expectRevert(AirdropHook.AirdropNotEnd.selector);
        hook.claimAirdrop(poolId);
    }
}
