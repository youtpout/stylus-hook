// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import {Token} from "./Token.sol";
import {IERC20Airdrop} from "./interfaces/IERC20Airdrop.sol";

/// @title Airdrop Token
/// @author Youtpout
/// @notice An IERC20 with airdrop mechanism
/// @dev Explain to a developer any extra details
contract AirdropToken is Token, IERC20Airdrop {
    uint256 public immutable totalAirdrop;
    address public immutable hook;
    uint256 public restAirdrop;

    error NotHook();

    constructor(
        string memory name,
        string memory symbol,
        address initialOwner,
        address _hook,
        uint256 _amountAirDrop
    ) Token(name, symbol, initialOwner) {
        _mint(address(this), _amountAirDrop);
        totalAirdrop = _amountAirDrop;
        hook = _hook;
    }
    
    function claim(address receiver, uint256 amount) external {
        if (msg.sender != hook) {
            // only hook can call this method
            revert NotHook();
        }

        if (amount > restAirdrop) {
            amount = restAirdrop;
        }

        restAirdrop -= amount;
        transfer(receiver, amount);
    }
}
