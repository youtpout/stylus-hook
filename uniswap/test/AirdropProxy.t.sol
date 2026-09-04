// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {PoolId} from "@uniswap/v4-core/src/types/PoolId.sol";

import {AirdropHookProxy} from "../src/AirdropHookProxy.sol";
import {AirdropToken} from "../src/AirdropToken.sol";
import {IAirdropHook} from "../src/interfaces/IAirdropHook.sol";
import {MockStylusAirdrop} from "./mocks/MockStylusAirdrop.sol";
import {HookTest} from "./utils/HookTest.sol";

/// @notice Exercises `AirdropHookProxy` — the hook whose state lives in Stylus — against the
///         Solidity replica of the Rust contract. This pins down the ABI the Stylus contract in
///         `stylus/airdrop` must expose, since forge cannot execute WASM.
contract AirdropProxyTest is HookTest {
    AirdropHookProxy internal hook;
    MockStylusAirdrop internal stylus;
    AirdropToken internal airdrop;

    address internal alice = makeAddr("alice");
    address internal bob = makeAddr("bob");

    uint256 internal constant AIRDROP_SUPPLY = 50_000_000 ether;

    function setUp() public {
        deployStack();

        stylus = new MockStylusAirdrop();

        uint160 flags = uint160(Hooks.AFTER_SWAP_FLAG);
        hook = AirdropHookProxy(
            deployHookTo(flags, "AirdropHookProxy.sol:AirdropHookProxy", abi.encode(poolManager, stylus))
        );
        stylus.setHook(address(hook));

        createPoolAndAddLiquidity(IHooks(address(hook)), 100e18);

        airdrop = new AirdropToken("FlydropProxy", "FLYP", address(this), address(stylus), AIRDROP_SUPPLY);
    }

    function _swapFor(address user, uint256 amountIn, bool zeroForOne) internal {
        swap(amountIn, zeroForOne, abi.encode(user));
    }

    function test_proxy_hook_is_bound_once() public {
        vm.expectRevert(IAirdropHook.HookAlreadyDefined.selector);
        stylus.setHook(address(this));
    }

    function test_proxy_rejects_direct_writes() public {
        vm.expectRevert(IAirdropHook.NotHook.selector);
        stylus.addAfterSwap(bytes32(0), alice, true, 1e18);
    }

    function test_proxy_accounting_matches_solidity_hook() public {
        _swapFor(alice, 1e18, true);
        _swapFor(alice, 2e18, true);
        _swapFor(bob, 1e18, false);

        assertEq(hook.totalUsers(poolId), 2);

        (uint256 amount0, uint256 amount1, uint256 counter0, uint256 counter1) =
            stylus.getTotalSwap(bytes32(PoolId.unwrap(poolId)));
        assertEq(amount1, 3e18);
        assertEq(counter1, 2);
        assertEq(amount0, 1e18);
        assertEq(counter0, 1);
    }

    function test_proxy_claim() public {
        _swapFor(alice, 3e18, true);
        _swapFor(bob, 1e18, true);

        hook.closeAirdrop(poolId, address(airdrop));

        uint256 aliceShare = hook.amountToClaim(poolId, alice);
        assertGt(aliceShare, hook.amountToClaim(poolId, bob));

        vm.prank(alice);
        hook.claimAirdrop(poolId);
        assertEq(airdrop.balanceOf(alice), aliceShare);

        vm.prank(alice);
        vm.expectRevert(IAirdropHook.AlreadyClaimed.selector);
        hook.claimAirdrop(poolId);
    }
}
