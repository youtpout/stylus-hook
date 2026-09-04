// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IAirdropHook} from "../../src/interfaces/IAirdropHook.sol";
import {IERC20Airdrop} from "../../src/interfaces/IERC20Airdrop.sol";

/// @notice Solidity replica of the Stylus (Rust) airdrop contract in `stylus/airdrop`.
/// @dev Forge cannot run WASM, so this stands in for the real Stylus contract in unit tests. It is
///      the executable spec the Rust implementation must match: any change here must be mirrored in
///      `stylus/airdrop/src/lib.rs` and in `IAirdropHook`.
contract MockStylusAirdrop is IAirdropHook {
    struct SwapInfo {
        uint256 amount0;
        uint256 amount1;
        uint256 counter0;
        uint256 counter1;
    }

    mapping(bytes32 => mapping(address => SwapInfo)) internal _totalSwapUser;
    mapping(bytes32 => SwapInfo) internal _totalSwap;
    mapping(bytes32 => uint256) internal _usersCount;
    mapping(bytes32 => mapping(address => bool)) internal _userExist;
    mapping(bytes32 => address) internal _airdropToken;
    mapping(bytes32 => mapping(address => bool)) internal _claimed;

    address public hook;

    modifier onlyHook() {
        if (msg.sender != hook) revert NotHook();
        _;
    }

    function setHook(address value) external {
        if (hook != address(0)) revert HookAlreadyDefined();
        hook = value;
    }

    function getTotalSwap(bytes32 poolId) external view returns (uint256, uint256, uint256, uint256) {
        SwapInfo memory s = _totalSwap[poolId];
        return (s.amount0, s.amount1, s.counter0, s.counter1);
    }

    function getTotalSwapUser(bytes32 poolId, address user)
        external
        view
        returns (uint256, uint256, uint256, uint256)
    {
        SwapInfo memory s = _totalSwapUser[poolId][user];
        return (s.amount0, s.amount1, s.counter0, s.counter1);
    }

    function totalUsers(bytes32 poolId) external view returns (uint256) {
        return _usersCount[poolId];
    }

    function airdropToken(bytes32 poolId) external view returns (address) {
        return _airdropToken[poolId];
    }

    function hasClaimed(bytes32 poolId, address user) external view returns (bool) {
        return _claimed[poolId][user];
    }

    function addAfterSwap(bytes32 poolId, address user, bool zeroForOne, uint256 amountSpecified)
        external
        onlyHook
    {
        // once the airdrop token is defined the airdrop accounting is closed
        if (_airdropToken[poolId] != address(0)) return;

        if (!_userExist[poolId][user]) {
            _userExist[poolId][user] = true;
            _usersCount[poolId]++;
        }

        SwapInfo storage swapUser = _totalSwapUser[poolId][user];
        SwapInfo storage swapTotal = _totalSwap[poolId];

        if (zeroForOne) {
            swapUser.amount1 += amountSpecified;
            swapUser.counter1++;
            swapTotal.amount1 += amountSpecified;
            swapTotal.counter1++;
        } else {
            swapUser.amount0 += amountSpecified;
            swapUser.counter0++;
            swapTotal.amount0 += amountSpecified;
            swapTotal.counter0++;
        }
    }

    function closeAirdrop(bytes32 poolId, address token) external onlyHook {
        _airdropToken[poolId] = token;
    }

    function claimAirdrop(bytes32 poolId, address receiver) external onlyHook {
        address token = _airdropToken[poolId];
        if (token == address(0)) revert AirdropNotEnd();
        if (_claimed[poolId][receiver]) revert AlreadyClaimed();

        // set claimed first to prevent from reentrancy try
        _claimed[poolId][receiver] = true;

        IERC20Airdrop(token).claim(receiver, _amountToClaim(poolId, token, receiver));
    }

    function amountToClaim(bytes32 poolId, address receiver) external view returns (uint256) {
        address token = _airdropToken[poolId];
        if (token == address(0)) return 0;
        return _amountToClaim(poolId, token, receiver);
    }

    function _amountToClaim(bytes32 poolId, address token, address receiver)
        internal
        view
        returns (uint256)
    {
        uint256 amountToAirdrop = IERC20Airdrop(token).totalAirdrop();
        SwapInfo memory swapUser = _totalSwapUser[poolId][receiver];
        SwapInfo memory swapTotal = _totalSwap[poolId];
        return _calc(amountToAirdrop, swapUser.amount0, swapTotal.amount0, 40)
            + _calc(amountToAirdrop, swapUser.amount1, swapTotal.amount1, 40)
            + _calc(amountToAirdrop, swapUser.counter0, swapTotal.counter0, 10)
            + _calc(amountToAirdrop, swapUser.counter1, swapTotal.counter1, 10);
    }

    function _calc(uint256 amountToAirdrop, uint256 userVolume, uint256 totalVolume, uint256 percent)
        internal
        pure
        returns (uint256)
    {
        // 1 to prevent divide by zero
        return (amountToAirdrop * userVolume * percent) / (1 + totalVolume * 100);
    }
}
