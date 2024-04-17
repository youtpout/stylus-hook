// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import {TStore} from "../src/TStore.sol";

contract TStoreScript is Script {
    function run() external {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;
        address deployer = 0x14791697260E4c9A71f18484C9f997B308e59325;

        vm.startBroadcast(privateKey);
        TStore con = new TStore();
        console.logString(
            string.concat("TStore deployed at: ", vm.toString(address(con)))
        );
        con.increment();
        vm.stopBroadcast();
    }
}
