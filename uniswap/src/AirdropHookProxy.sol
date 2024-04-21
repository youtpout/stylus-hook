// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {BaseHook} from "v4-periphery/BaseHook.sol";

import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "v4-core/src/types/BalanceDelta.sol";
import {IERC20Airdrop} from "./interfaces/IERC20Airdrop.sol";

import {IAirdropHook} from "./interfaces/IAirdropHook.sol";

/// @title Airdrop Hook
/// @author Youtpout
/// @notice An hook to manage airdrop to user who use this pool
/// @dev Explain to a developer any extra details
contract AirdropHookProxy is BaseHook {
    using PoolIdLibrary for PoolKey;

    IAirdropHook constant hookProxy =
        IAirdropHook(0x8cDE56336E289c028C8f7CF5c20283fF02272182);

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

    function afterSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata swapParams,
        BalanceDelta,
        bytes calldata
    ) external override returns (bytes4) {
        // tx origin to manage initial caller, not the best thing to do but for hacakathon shortcut
        address sender = tx.origin;
        PoolId poolId = key.toId();

        uint256 amount;
        if (swapParams.amountSpecified > 0) {
            amount = uint256(swapParams.amountSpecified);
        } else {
            amount = uint256(-swapParams.amountSpecified);
        }

        hookProxy.addAfterSwap(
            PoolId.unwrap(poolId),
            sender,
            swapParams.zeroForOne,
            amount
        );

        return AirdropHookProxy.afterSwap.selector;
    }

    // not secure but for test
    function closeAirdrop(PoolId poolId, address token) external {
        hookProxy.closeAirdrop(PoolId.unwrap(poolId), token);
    }

    function claimAirdrop(PoolId poolId) external {
        hookProxy.claimAirdrop(PoolId.unwrap(poolId), msg.sender);
    }

    function amountToClaim(
        PoolId poolId,
        address receiver
    ) public view returns (uint256) {
        return hookProxy.amountToClaim(PoolId.unwrap(poolId), receiver);
    }
}
