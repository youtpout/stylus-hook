// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {BalanceDelta} from "@uniswap/v4-core/src/types/BalanceDelta.sol";
import {Constants} from "@uniswap/v4-core/test/utils/Constants.sol";
import {EasyPosm} from "./utils/libraries/EasyPosm.sol";

import {Counter} from "../src/Counter.sol";
import {HookTest} from "./utils/HookTest.sol";

contract CounterTest is HookTest {
    Counter internal hook;

    function setUp() public {
        deployStack();

        uint160 flags = uint160(
            Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG | Hooks.BEFORE_ADD_LIQUIDITY_FLAG
                | Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
        );
        hook = Counter(deployHookTo(flags, "Counter.sol:Counter", abi.encode(poolManager)));

        createPoolAndAddLiquidity(IHooks(address(hook)), 100e18);
    }

    function test_counter_swap() public {
        assertEq(hook.beforeAddLiquidityCount(poolId), 1);
        assertEq(hook.beforeRemoveLiquidityCount(poolId), 0);
        assertEq(hook.beforeSwapCount(poolId), 0);
        assertEq(hook.afterSwapCount(poolId), 0);

        uint256 amountIn = 1e18;
        BalanceDelta swapDelta = swap(amountIn, true, Constants.ZERO_BYTES);
        assertEq(int256(swapDelta.amount0()), -int256(amountIn));

        assertEq(hook.beforeSwapCount(poolId), 1);
        assertEq(hook.afterSwapCount(poolId), 1);
    }

    function test_counter_liquidity() public {
        assertEq(hook.beforeAddLiquidityCount(poolId), 1);
        assertEq(hook.beforeRemoveLiquidityCount(poolId), 0);

        EasyPosm.decreaseLiquidity(
            positionManager, tokenId, 1e18, 0, 0, address(this), block.timestamp, Constants.ZERO_BYTES
        );

        assertEq(hook.beforeAddLiquidityCount(poolId), 1);
        assertEq(hook.beforeRemoveLiquidityCount(poolId), 1);
    }
}
