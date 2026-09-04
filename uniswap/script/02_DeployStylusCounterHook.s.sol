// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {HookMiner} from "@uniswap/v4-periphery/src/utils/HookMiner.sol";

import {console} from "forge-std/console.sol";

import {BaseScript} from "./base/BaseScript.sol";
import {CounterProxy} from "../src/CounterProxy.sol";
import {ICounter} from "../src/ICounter.sol";

/// @notice Mines a CREATE2 salt, deploys `CounterProxy` in front of an already-deployed Stylus
///         contract, and binds the two together.
///
/// Deploy the Stylus side first:
///   cd stylus && cargo stylus deploy --contract stylus-counter-hook \
///     --endpoint https://sepolia-rollup.arbitrum.io/rpc --private-key $PRIVATE_KEY
///
/// then:
///   STYLUS_COUNTER=0x... forge script script/02_DeployStylusCounterHook.s.sol \
///     --rpc-url arbitrum_sepolia --broadcast
contract DeployStylusCounterHookScript is BaseScript {
    function run() public returns (CounterProxy hook) {
        ICounter stylusCounter = ICounter(vm.envAddress("STYLUS_COUNTER"));
        require(address(stylusCounter).code.length > 0, "STYLUS_COUNTER has no code");
        require(stylusCounter.hook() == address(0), "Stylus contract is already bound to a hook");

        uint160 flags = uint160(
            Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG | Hooks.BEFORE_ADD_LIQUIDITY_FLAG
                | Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
        );

        bytes memory constructorArgs = abi.encode(poolManager, stylusCounter);
        (address hookAddress, bytes32 salt) =
            HookMiner.find(CREATE2_FACTORY, flags, type(CounterProxy).creationCode, constructorArgs);

        vm.startBroadcast();
        hook = new CounterProxy{salt: salt}(poolManager, stylusCounter);
        // the Stylus contract only accepts writes from this hook
        stylusCounter.setHook(address(hook));
        vm.stopBroadcast();

        require(address(hook) == hookAddress, "DeployStylusCounterHookScript: hook address mismatch");
        console.log("CounterProxy (Stylus-backed):", address(hook));
        console.log("Stylus counter contract:", address(stylusCounter));
    }
}
