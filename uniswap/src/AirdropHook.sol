// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {BaseHook} from "v4-periphery/BaseHook.sol";

import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "v4-core/src/types/BalanceDelta.sol";

/// @title Airdrop Hook
/// @author Youtpout
/// @notice An hook to manage airdrop to user who use this pool
/// @dev Explain to a developer any extra details
contract AirdropHook is BaseHook {
    using PoolIdLibrary for PoolKey;

    // NOTE: ---------------------------------------------------------
    // state variables should typically be unique to a pool
    // a single hook contract should be able to service multiple pools
    // ---------------------------------------------------------------

    mapping(PoolId => mapping(address => uint256)) public totalSwapAmount0;
    mapping(PoolId => mapping(address => uint256)) public totalSwapAmount1;

    mapping(PoolId => mapping(address => uint256)) public numberSwap0;
    mapping(PoolId => mapping(address => uint256)) public numberSwap1;

    mapping(PoolId => address[]) public users;
    mapping(PoolId => mapping(address => uint256)) public userExist;

    constructor(IPoolManager _poolManager) BaseHook(_poolManager) {}

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
                beforeAddLiquidity: false,
                afterAddLiquidity: false,
                beforeRemoveLiquidity: false,
                afterRemoveLiquidity: false,
                beforeSwap: false,
                afterSwap: true,
                beforeDonate: false,
                afterDonate: false
            });
    }

    function totalUsers(PoolId poolId) external returns (uint256) {
        return users[poolId].length;
    }

    function afterSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata,
        BalanceDelta,
        bytes calldata
    ) external override returns (bytes4) {
        //afterSwapCount[key.toId()]++;
        return AirdropHook.afterSwap.selector;
    }
}
