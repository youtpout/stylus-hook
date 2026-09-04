// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {TickMath} from "@uniswap/v4-core/src/libraries/TickMath.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {PoolId} from "@uniswap/v4-core/src/types/PoolId.sol";
import {Currency} from "@uniswap/v4-core/src/types/Currency.sol";
import {BalanceDelta} from "@uniswap/v4-core/src/types/BalanceDelta.sol";
import {StateLibrary} from "@uniswap/v4-core/src/libraries/StateLibrary.sol";
import {LiquidityAmounts} from "@uniswap/v4-core/test/utils/LiquidityAmounts.sol";
import {Constants} from "@uniswap/v4-core/test/utils/Constants.sol";
import {IPositionManager} from "@uniswap/v4-periphery/src/interfaces/IPositionManager.sol";

import {EasyPosm} from "./libraries/EasyPosm.sol";
import {BaseTest} from "./BaseTest.sol";

/// @notice Shared plumbing for the hook tests: deploys a local v4 stack, mines a hook address with
///         the requested permission flags, opens a pool on it and seeds full-range liquidity.
abstract contract HookTest is BaseTest {
    using EasyPosm for IPositionManager;
    using StateLibrary for IPoolManager;

    Currency internal currency0;
    Currency internal currency1;

    PoolKey internal poolKey;
    PoolId internal poolId;

    uint256 internal tokenId;
    int24 internal tickLower;
    int24 internal tickUpper;

    /// @dev Namespace the mined hook addresses so different hooks never collide in one test run.
    uint160 internal constant HOOK_NAMESPACE = 0x4444 << 144;

    function deployStack() internal {
        deployArtifactsAndLabel();
        (currency0, currency1) = deployCurrencyPair();
    }

    /// @dev `forge` cannot CREATE2-mine inside a test, so the hook bytecode is etched at an address
    ///      that already carries the right flags. This is the pattern used by the official
    ///      v4-template; `script/00_DeployHook.s.sol` does real CREATE2 mining on a live chain.
    function deployHookTo(uint160 flags, string memory artifact, bytes memory constructorArgs)
        internal
        returns (address hookAddress)
    {
        hookAddress = address(flags ^ HOOK_NAMESPACE);
        deployCodeTo(artifact, constructorArgs, hookAddress);
        vm.label(hookAddress, artifact);
    }

    function createPoolAndAddLiquidity(IHooks hooks, uint128 liquidityAmount) internal {
        poolKey = PoolKey(currency0, currency1, 3000, 60, hooks);
        poolId = poolKey.toId();
        poolManager.initialize(poolKey, Constants.SQRT_PRICE_1_1);

        tickLower = TickMath.minUsableTick(poolKey.tickSpacing);
        tickUpper = TickMath.maxUsableTick(poolKey.tickSpacing);

        (uint256 amount0Expected, uint256 amount1Expected) = LiquidityAmounts.getAmountsForLiquidity(
            Constants.SQRT_PRICE_1_1,
            TickMath.getSqrtPriceAtTick(tickLower),
            TickMath.getSqrtPriceAtTick(tickUpper),
            liquidityAmount
        );

        (tokenId,) = EasyPosm.mint(
            positionManager,
            poolKey,
            tickLower,
            tickUpper,
            liquidityAmount,
            amount0Expected + 1,
            amount1Expected + 1,
            address(this),
            block.timestamp,
            Constants.ZERO_BYTES
        );
    }

    function swap(uint256 amountIn, bool zeroForOne, bytes memory hookData)
        internal
        returns (BalanceDelta)
    {
        return swapRouter.swapExactTokensForTokens({
            amountIn: amountIn,
            amountOutMin: 0, // no slippage protection: tests want unlimited price impact
            zeroForOne: zeroForOne,
            poolKey: poolKey,
            hookData: hookData,
            receiver: address(this),
            deadline: block.timestamp + 1
        });
    }
}
