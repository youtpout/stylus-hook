// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {HookMiner} from "@uniswap/v4-periphery/src/utils/HookMiner.sol";

import {console} from "forge-std/console.sol";

import {BaseScript} from "./base/BaseScript.sol";
import {AirdropHookProxy} from "../src/AirdropHookProxy.sol";
import {IAirdropHook} from "../src/interfaces/IAirdropHook.sol";

/// @notice Mines a CREATE2 salt, deploys `AirdropHookProxy` in front of an already-deployed Stylus
///         contract, and binds the two together.
///
/// Deploy the Stylus side first:
///   cd stylus && cargo stylus deploy --contract stylus-airdrop-hook \
///     --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key $PRIVATE_KEY
///
/// then:
///   STYLUS_AIRDROP=0x... forge script script/01_DeployStylusAirdropHook.s.sol \
///     --rpc-url arbitrum_sepolia --broadcast
contract DeployStylusAirdropHookScript is BaseScript {
    function run() public returns (AirdropHookProxy hook) {
        IAirdropHook stylusAirdrop = IAirdropHook(vm.envAddress("STYLUS_AIRDROP"));
        require(address(stylusAirdrop).code.length > 0, "STYLUS_AIRDROP has no code");
        require(stylusAirdrop.hook() == address(0), "Stylus contract is already bound to a hook");

        uint160 flags = uint160(Hooks.AFTER_SWAP_FLAG);

        bytes memory constructorArgs = abi.encode(poolManager, stylusAirdrop);
        (address hookAddress, bytes32 salt) =
            HookMiner.find(CREATE2_FACTORY, flags, type(AirdropHookProxy).creationCode, constructorArgs);

        vm.startBroadcast();
        hook = new AirdropHookProxy{salt: salt}(poolManager, stylusAirdrop);
        // the Stylus contract only accepts writes from this hook
        stylusAirdrop.setHook(address(hook));
        vm.stopBroadcast();

        require(address(hook) == hookAddress, "DeployStylusAirdropHookScript: hook address mismatch");
        console.log("AirdropHookProxy (Stylus-backed):", address(hook));
        console.log("Stylus airdrop contract:", address(stylusAirdrop));
    }
}
