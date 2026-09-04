// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams, ModifyLiquidityParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {PoolId} from "@uniswap/v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "@uniswap/v4-core/src/types/BalanceDelta.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "@uniswap/v4-core/src/types/BeforeSwapDelta.sol";

import {ICounter} from "./ICounter.sol";

/// @title CounterProxy
/// @notice Same behaviour as `Counter`, but every piece of state and logic lives in a Stylus
///         (Rust/WASM) contract on Arbitrum. This contract only exists so the deployed address
///         carries the v4 hook permission flags; it forwards each callback to Stylus.
contract CounterProxy is BaseHook {
    /// @notice The Stylus contract holding the counters.
    ICounter public immutable stylusCounter;

    constructor(IPoolManager _poolManager, ICounter _stylusCounter) BaseHook(_poolManager) {
        stylusCounter = _stylusCounter;
    }

    function beforeSwapCount(PoolId poolId) external view returns (uint256) {
        return stylusCounter.beforeSwapCount(PoolId.unwrap(poolId));
    }

    function afterSwapCount(PoolId poolId) external view returns (uint256) {
        return stylusCounter.afterSwapCount(PoolId.unwrap(poolId));
    }

    function beforeAddLiquidityCount(PoolId poolId) external view returns (uint256) {
        return stylusCounter.beforeAddLiquidityCount(PoolId.unwrap(poolId));
    }

    function beforeRemoveLiquidityCount(PoolId poolId) external view returns (uint256) {
        return stylusCounter.beforeRemoveLiquidityCount(PoolId.unwrap(poolId));
    }

    function getHookPermissions() public pure override returns (Hooks.Permissions memory) {
        return Hooks.Permissions({
            beforeInitialize: false,
            afterInitialize: false,
            beforeAddLiquidity: true,
            afterAddLiquidity: false,
            beforeRemoveLiquidity: true,
            afterRemoveLiquidity: false,
            beforeSwap: true,
            afterSwap: true,
            beforeDonate: false,
            afterDonate: false,
            beforeSwapReturnDelta: false,
            afterSwapReturnDelta: false,
            afterAddLiquidityReturnDelta: false,
            afterRemoveLiquidityReturnDelta: false
        });
    }

    // -----------------------------------------------
    // NOTE: see IHooks.sol for function documentation
    // -----------------------------------------------

    function _beforeSwap(address, PoolKey calldata key, SwapParams calldata, bytes calldata)
        internal
        override
        returns (bytes4, BeforeSwapDelta, uint24)
    {
        stylusCounter.addBeforeSwap(PoolId.unwrap(key.toId()));
        return (BaseHook.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }

    function _afterSwap(address, PoolKey calldata key, SwapParams calldata, BalanceDelta, bytes calldata)
        internal
        override
        returns (bytes4, int128)
    {
        stylusCounter.addAfterSwap(PoolId.unwrap(key.toId()));
        return (BaseHook.afterSwap.selector, 0);
    }

    function _beforeAddLiquidity(address, PoolKey calldata key, ModifyLiquidityParams calldata, bytes calldata)
        internal
        override
        returns (bytes4)
    {
        stylusCounter.addBeforeAddLiquidity(PoolId.unwrap(key.toId()));
        return BaseHook.beforeAddLiquidity.selector;
    }

    function _beforeRemoveLiquidity(address, PoolKey calldata key, ModifyLiquidityParams calldata, bytes calldata)
        internal
        override
        returns (bytes4)
    {
        stylusCounter.addBeforeRemoveLiquidity(PoolId.unwrap(key.toId()));
        return BaseHook.beforeRemoveLiquidity.selector;
    }
}
