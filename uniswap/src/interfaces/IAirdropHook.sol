// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IAirdropHook {
    function getTotalSwap(
        bytes32 pool_id
    ) external view returns (uint256, uint256, uint256, uint256);

    function setHook(address value) external;

    function addAfterSwap(
        bytes32 pool_id,
        address sender,
        bool zero_for_one,
        uint256 amount_specified
    ) external;

    function closeAirdrop(bytes32 pool_id, address token) external;

    function claimAirdrop(bytes32 pool_id, address receiver) external;

    function claim(bytes32 pool_id) external;

    function amountToClaim(
        bytes32 pool_id,
        address receiver
    ) external view returns (uint256);

    function amountToClaim(
        bytes32 pool_id,
        address token,
        address receiver
    ) external view returns (uint256);

    function calculateTokenAirdrop(
        uint256 amount_to_airdrop,
        uint256 user_volume,
        uint256 total_volume,
        uint256 percent
    ) external view returns (uint256);

    error NotHook();

    error HookAlreadyDefined();

    error AirdropNotEnd();

    error AlreadyClaimed();
}
