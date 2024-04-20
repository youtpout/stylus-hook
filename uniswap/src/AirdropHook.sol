// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {BaseHook} from "v4-periphery/BaseHook.sol";

import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "v4-core/src/types/BalanceDelta.sol";

interface IERC20 {
    function totalAirdrop() external returns (uint256);

    function restAirdrop() external returns (uint256);

    function claim(address receiver, uint256 amount) external;
}

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

    // Airdrop is caculated between total amount swaped by user and number of swap did
    struct SwapInfo {
        // amount swapped is profitable for liquidity provider
        uint256 amount0;
        uint256 amount1;
        // number swapped is profitable for network miner
        uint128 counter0;
        uint128 counter1;
    }

    mapping(PoolId => mapping(address => SwapInfo)) public totalSwapUser;
    mapping(PoolId => SwapInfo) public totalSwap;

    mapping(PoolId => address[]) public users;
    mapping(PoolId => mapping(address => bool)) public userExist;
    mapping(PoolId => address) public airdropToken;
    mapping(PoolId => mapping(address => bool)) public claimed;

    error AirdropNotEnd();
    error AlreadyClaimed();

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
        address sender,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata swapParams,
        BalanceDelta balanceDelta,
        bytes calldata
    ) external override returns (bytes4) {
        PoolId poolId = key.toId();

        // if the token for airdrop if define, aidrop calculation is finished
        if (airdropToken[poolId] != address(0)) {
            return AirdropHook.afterSwap.selector;
        }

        if (!userExist[poolId][sender]) {
            // if user not exist add it
            userExist[poolId][sender] = true;
            users[poolId].push(sender);
        }

        uint256 amount;
        if (swapParams.amountSpecified > 0) {
            amount = uint256(swapParams.amountSpecified);
        } else {
            amount = uint256(-swapParams.amountSpecified);
        }

        SwapInfo storage swapUser = totalSwapUser[poolId][sender];
        SwapInfo storage swapTotal = totalSwap[poolId];

        if (swapParams.zeroForOne) {
            swapUser.amount1 += amount;
            swapUser.counter1++;
            swapTotal.amount1 += amount;
            swapTotal.counter1++;
        } else {
            swapUser.amount0 += amount;
            swapUser.counter0++;
            swapTotal.amount0 += amount;
            swapTotal.counter0++;
        }

        return AirdropHook.afterSwap.selector;
    }

    // not secure but for test
    function closeAirdrop(PoolId poolId, address token) external {
        airdropToken[poolId] = token;
    }

    function claimAirdrop(PoolId poolId) external {
        if (airdropToken[poolId] == address(0)) {
            revert AirdropNotEnd();
        }

        if (claimed[poolId][msg.sender]) {
            revert AlreadyClaimed();
        }
    }
}
