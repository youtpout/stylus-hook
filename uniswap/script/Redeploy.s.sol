// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "forge-std/Script.sol";
import {Redeploy} from "../src/Redeploy.sol";

contract RedeployScript is Script {
    Redeploy redeploy;
    function run() external {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;
        address deployer = 0x14791697260E4c9A71f18484C9f997B308e59325;

        vm.startBroadcast(privateKey);
        redeploy = new Redeploy();
        vm.stopBroadcast();
    }
}
