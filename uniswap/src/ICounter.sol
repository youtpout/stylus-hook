// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/**
 * Solidity view of the Stylus (Rust) counter contract living in `stylus/counter`.
 * Generated with `cargo stylus export-abi`, kept in sync by hand.
 */
interface ICounter {
    /// @notice The v4 hook contract allowed to write into this contract.
    function hook() external view returns (address);

    /// @notice One-shot binding of the hook contract. Reverts once set.
    function setHook(address value) external;

    function beforeSwapCount(bytes32 poolId) external view returns (uint256);

    function afterSwapCount(bytes32 poolId) external view returns (uint256);

    function beforeAddLiquidityCount(bytes32 poolId) external view returns (uint256);

    function beforeRemoveLiquidityCount(bytes32 poolId) external view returns (uint256);

    function addBeforeSwap(bytes32 poolId) external;

    function addAfterSwap(bytes32 poolId) external;

    function addBeforeAddLiquidity(bytes32 poolId) external;

    function addBeforeRemoveLiquidity(bytes32 poolId) external;

    error NotHook();

    error HookAlreadyDefined();
}
