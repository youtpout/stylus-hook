// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {PoolId} from "@uniswap/v4-core/src/types/PoolId.sol";
import {BalanceDelta} from "@uniswap/v4-core/src/types/BalanceDelta.sol";

import {IERC20Airdrop} from "./interfaces/IERC20Airdrop.sol";

/// @title Airdrop Hook
/// @author Youtpout
/// @notice A hook that rewards the users of a pool with an airdrop proportional to how much
///         and how often they swapped in it.
/// @dev Pure-Solidity reference implementation. `AirdropHookProxy` is the equivalent whose
///      accounting lives in an Arbitrum Stylus (Rust/WASM) contract.
contract AirdropHook is BaseHook {
    // NOTE: ---------------------------------------------------------
    // state variables should typically be unique to a pool
    // a single hook contract should be able to service multiple pools
    // ---------------------------------------------------------------

    /// @dev Airdrop is computed from the amount swapped by a user and the number of swaps they did.
    struct SwapInfo {
        // amount swapped is profitable for liquidity providers
        uint256 amount0;
        uint256 amount1;
        // number of swaps is profitable for the network
        uint256 counter0;
        uint256 counter1;
    }

    mapping(PoolId poolId => mapping(address user => SwapInfo)) public totalSwapUser;
    mapping(PoolId poolId => SwapInfo) public totalSwap;

    mapping(PoolId poolId => uint256) public usersCount;
    mapping(PoolId poolId => mapping(address user => bool)) public userExist;
    mapping(PoolId poolId => address) public airdropToken;
    mapping(PoolId poolId => mapping(address user => bool)) public claimed;

    error AirdropNotEnd();
    error AlreadyClaimed();

    constructor(IPoolManager _poolManager) BaseHook(_poolManager) {}

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
        return usersCount[poolId];
    }

    /// @dev `sender` is the contract that called the PoolManager (usually a router), so the
    ///      beneficiary is read from `hookData` when the router forwards it. Routers that do not
    ///      forward hook data credit themselves, which is the documented behaviour of v4 hooks.
    function _beforeSwapBeneficiary(address sender, bytes calldata hookData) private pure returns (address) {
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
        PoolId poolId = key.toId();

        // once the airdrop token is defined the airdrop accounting is closed
        if (airdropToken[poolId] != address(0)) {
            return (BaseHook.afterSwap.selector, 0);
        }

        address user = _beforeSwapBeneficiary(sender, hookData);

        if (!userExist[poolId][user]) {
            userExist[poolId][user] = true;
            usersCount[poolId]++;
        }

        uint256 amount = swapParams.amountSpecified > 0
            ? uint256(swapParams.amountSpecified)
            : uint256(-swapParams.amountSpecified);

        SwapInfo storage swapUser = totalSwapUser[poolId][user];
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

        return (BaseHook.afterSwap.selector, 0);
    }

    // not secure but for test
    function closeAirdrop(PoolId poolId, address token) external {
        airdropToken[poolId] = token;
    }

    function claimAirdrop(PoolId poolId) external {
        IERC20Airdrop token = IERC20Airdrop(airdropToken[poolId]);
        if (address(token) == address(0)) {
            revert AirdropNotEnd();
        }

        if (claimed[poolId][msg.sender]) {
            revert AlreadyClaimed();
        }

        // set claimed first to prevent from reentrancy try
        claimed[poolId][msg.sender] = true;

        uint256 amount = _amountToClaim(poolId, token, msg.sender);

        token.claim(msg.sender, amount);
    }

    function amountToClaim(PoolId poolId, address receiver) public view returns (uint256) {
        address token = airdropToken[poolId];
        if (token == address(0)) return 0;
        return _amountToClaim(poolId, IERC20Airdrop(token), receiver);
    }

    function _amountToClaim(PoolId poolId, IERC20Airdrop token, address receiver)
        private
        view
        returns (uint256)
    {
        uint256 amountToAirdrop = token.totalAirdrop();
        SwapInfo memory swapUser = totalSwapUser[poolId][receiver];
        SwapInfo memory swapTotal = totalSwap[poolId];
        // 80 % based on volume and 20 % on the number of swaps, so 40 % / 10 % for each token
        uint256 amountVolume0 = _calculateTokenAirdrop(amountToAirdrop, swapUser.amount0, swapTotal.amount0, 40);
        uint256 amountVolume1 = _calculateTokenAirdrop(amountToAirdrop, swapUser.amount1, swapTotal.amount1, 40);
        uint256 amountCounter0 = _calculateTokenAirdrop(amountToAirdrop, swapUser.counter0, swapTotal.counter0, 10);
        uint256 amountCounter1 = _calculateTokenAirdrop(amountToAirdrop, swapUser.counter1, swapTotal.counter1, 10);
        return (amountVolume0 + amountVolume1 + amountCounter0 + amountCounter1);
    }

    function _calculateTokenAirdrop(
        uint256 amountToAirdrop,
        uint256 userVolume,
        uint256 totalVolume,
        uint256 percent
    ) private pure returns (uint256) {
        // 1 to prevent divide by zero
        return (amountToAirdrop * userVolume * percent) / (1 + totalVolume * 100);
    }
}
