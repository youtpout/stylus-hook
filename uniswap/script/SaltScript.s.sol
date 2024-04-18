// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/Script.sol";
import {IHooks} from "v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {HookMiner} from "../test/utils/HookMiner.sol";
import {Redeploy} from "../src/Redeploy.sol";
import {Counter} from "../src/Counter.sol";

contract SaltScript is Script {
    Redeploy redeploy = Redeploy(0xF307542c099a7Ac9975B14E98C4b2ab0004345e8);
    address createProxy = 0x14791697260E4c9A71f18484C9f997B308e59325;
    Counter counter;

    function run() external {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;
        address deployer = 0x14791697260E4c9A71f18484C9f997B308e59325;

        uint160 flags = uint160(
            Hooks.BEFORE_SWAP_FLAG |
                Hooks.AFTER_SWAP_FLAG |
                Hooks.BEFORE_ADD_LIQUIDITY_FLAG |
                Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
        );

        (address hookAddress, bytes32 salt) = HookMiner.find(
            address(createProxy),
            flags,
            counter.code,
            abi.encode(address(manager))
        );
        console.logString(
            string.concat("hookAddress : ", vm.toString(address(hookAddress)))
        );

        console.logString(string.concat("salt : ", vm.toString(salt)));
    }
}
