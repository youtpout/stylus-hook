// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {BaseHook} from "v4-periphery/BaseHook.sol";

import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "v4-core/src/types/BalanceDelta.sol";
import {ICounter} from "./ICounter.sol";

contract CounterProxy is BaseHook {
    using PoolIdLibrary for PoolKey;

    ICounter public HOOK_PROXY =
        ICounter(0x8cDE56336E289c028C8f7CF5c20283fF02272182);

    function beforeSwapCount(bytes32 pool_id) external view returns (uint256) {
        return HOOK_PROXY.beforeSwapCount(pool_id);
    }

    function afterSwapCount(bytes32 pool_id) external view returns (uint256) {
        return HOOK_PROXY.afterSwapCount(pool_id);
    }

    function beforeAddLiquidityCount(
        bytes32 pool_id
    ) external view returns (uint256) {
        return HOOK_PROXY.beforeAddLiquidityCount(pool_id);
    }

    function beforeRemoveLiquidityCount(
        bytes32 pool_id
    ) external view returns (uint256) {
        return HOOK_PROXY.beforeRemoveLiquidityCount(pool_id);
    }

    constructor(IPoolManager _poolManager) BaseHook(_poolManager) {
    }

    function getHookPermissions()
        public
        pure
        override
        returns (Hooks.Permissions memory)
    {
        return
            Hooks.Permissions({
                beforeInitialize: false,
                afterInitialize: false,
                beforeAddLiquidity: true,
                afterAddLiquidity: false,
                beforeRemoveLiquidity: true,
                afterRemoveLiquidity: false,
                beforeSwap: true,
                afterSwap: true,
                beforeDonate: false,
                afterDonate: false
            });
    }

    // -----------------------------------------------
    // NOTE: see IHooks.sol for function documentation
    // -----------------------------------------------

    function beforeSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata,
        bytes calldata
    ) external override returns (bytes4) {
        HOOK_PROXY.addBeforeSwap(PoolId.unwrap(key.toId()));
        return BaseHook.beforeSwap.selector;
    }

    function afterSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata,
        BalanceDelta,
        bytes calldata
    ) external override returns (bytes4) {
        HOOK_PROXY.addAfterSwap(PoolId.unwrap(key.toId()));
        return BaseHook.afterSwap.selector;
    }

    function beforeAddLiquidity(
        address,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata,
        bytes calldata
    ) external override returns (bytes4) {
        HOOK_PROXY.addBeforeAddLiquidity(PoolId.unwrap(key.toId()));
        return BaseHook.beforeAddLiquidity.selector;
    }

    function beforeRemoveLiquidity(
        address,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata,
        bytes calldata
    ) external override returns (bytes4) {
        HOOK_PROXY.addBeforeRemoveLiquidity(PoolId.unwrap(key.toId()));
        return BaseHook.beforeRemoveLiquidity.selector;
    }
}
