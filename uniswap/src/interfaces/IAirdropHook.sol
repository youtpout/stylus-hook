// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/**
 * Solidity view of the Stylus (Rust) airdrop contract living in `stylus/airdrop`.
 * Generated with `cargo stylus export-abi`, kept in sync by hand.
 */
interface IAirdropHook {
    /// @notice The v4 hook contract allowed to write into this contract.
    function hook() external view returns (address);

    /// @notice One-shot binding of the hook contract. Reverts once set.
    function setHook(address value) external;

    function getTotalSwap(bytes32 poolId) external view returns (uint256, uint256, uint256, uint256);

    function getTotalSwapUser(bytes32 poolId, address user)
        external
        view
        returns (uint256, uint256, uint256, uint256);

    function totalUsers(bytes32 poolId) external view returns (uint256);

    function airdropToken(bytes32 poolId) external view returns (address);

    function hasClaimed(bytes32 poolId, address user) external view returns (bool);

    function addAfterSwap(bytes32 poolId, address user, bool zeroForOne, uint256 amountSpecified) external;

    function closeAirdrop(bytes32 poolId, address token) external;

    function claimAirdrop(bytes32 poolId, address receiver) external;

    function amountToClaim(bytes32 poolId, address receiver) external view returns (uint256);

    error NotHook();

    error HookAlreadyDefined();

    error AirdropNotEnd();

    error AlreadyClaimed();

    error TokenCallFailed();
}
