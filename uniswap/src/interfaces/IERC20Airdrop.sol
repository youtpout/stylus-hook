// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

interface IERC20Airdrop {
    function totalAirdrop() external view returns (uint256);

    function restAirdrop() external returns (uint256);

    function claim(address receiver, uint256 amount) external;
}
