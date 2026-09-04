// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {PoolId} from "@uniswap/v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "@uniswap/v4-core/src/types/BalanceDelta.sol";

import {IAirdropHook} from "./interfaces/IAirdropHook.sol";

/// @title Airdrop Hook Proxy
/// @author Youtpout
/// @notice Same behaviour as `AirdropHook`, but the whole airdrop accounting lives in a Stylus
///         (Rust/WASM) contract on Arbitrum. This contract only exists so the deployed address
///         carries the v4 hook permission flags; it forwards each callback to Stylus.
contract AirdropHookProxy is BaseHook {
    /// @notice The Stylus contract holding the airdrop accounting.
    IAirdropHook public immutable stylusAirdrop;

    constructor(IPoolManager _poolManager, IAirdropHook _stylusAirdrop) BaseHook(_poolManager) {
        stylusAirdrop = _stylusAirdrop;
    }

    function getHookPermissions() public pure override returns (Hooks.Permissions memory) {
        return Hooks.Permissions({
            beforeInitialize: false,
            afterInitialize: false,
            beforeAddLiquidity: false,
            afterAddLiquidity: false,
            beforeRemoveLiquidity: false,
            afterRemoveLiquidity: false,
            beforeSwap: false,
            afterSwap: true,
            beforeDonate: false,
            afterDonate: false,
            beforeSwapReturnDelta: false,
            afterSwapReturnDelta: false,
            afterAddLiquidityReturnDelta: false,
            afterRemoveLiquidityReturnDelta: false
        });
    }

    function totalUsers(PoolId poolId) external view returns (uint256) {
        return stylusAirdrop.totalUsers(PoolId.unwrap(poolId));
    }

    /// @dev See `AirdropHook._beforeSwapBeneficiary`.
    function _beneficiary(address sender, bytes calldata hookData) private pure returns (address) {
        if (hookData.length == 32) return abi.decode(hookData, (address));
        return sender;
    }

    function _afterSwap(
        address sender,
        PoolKey calldata key,
        SwapParams calldata swapParams,
        BalanceDelta,
        bytes calldata hookData
    ) internal override returns (bytes4, int128) {
        uint256 amount = swapParams.amountSpecified > 0
            ? uint256(swapParams.amountSpecified)
            : uint256(-swapParams.amountSpecified);

        stylusAirdrop.addAfterSwap(
            PoolId.unwrap(key.toId()), _beneficiary(sender, hookData), swapParams.zeroForOne, amount
        );

        return (BaseHook.afterSwap.selector, 0);
    }

    // not secure but for test
    function closeAirdrop(PoolId poolId, address token) external {
        stylusAirdrop.closeAirdrop(PoolId.unwrap(poolId), token);
    }

    function claimAirdrop(PoolId poolId) external {
        stylusAirdrop.claimAirdrop(PoolId.unwrap(poolId), msg.sender);
    }

    function amountToClaim(PoolId poolId, address receiver) public view returns (uint256) {
        return stylusAirdrop.amountToClaim(PoolId.unwrap(poolId), receiver);
    }
}
