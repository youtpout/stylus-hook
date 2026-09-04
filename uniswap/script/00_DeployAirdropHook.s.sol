// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {HookMiner} from "@uniswap/v4-periphery/src/utils/HookMiner.sol";

import {console} from "forge-std/console.sol";

import {BaseScript} from "./base/BaseScript.sol";
import {AirdropHook} from "../src/AirdropHook.sol";

/// @notice Mines a CREATE2 salt and deploys the pure-Solidity `AirdropHook`, the baseline the
///         Stylus hook is compared against.
///
/// forge script script/00_DeployAirdropHook.s.sol --rpc-url arbitrum_sepolia --broadcast
contract DeployAirdropHookScript is BaseScript {
    function run() public returns (AirdropHook hook) {
        uint160 flags = uint160(Hooks.AFTER_SWAP_FLAG);

        bytes memory constructorArgs = abi.encode(poolManager);
        (address hookAddress, bytes32 salt) =
            HookMiner.find(CREATE2_FACTORY, flags, type(AirdropHook).creationCode, constructorArgs);

        vm.startBroadcast();
        hook = new AirdropHook{salt: salt}(poolManager);
        vm.stopBroadcast();

        require(address(hook) == hookAddress, "DeployAirdropHookScript: hook address mismatch");
        console.log("AirdropHook (Solidity):", address(hook));
    }
}
