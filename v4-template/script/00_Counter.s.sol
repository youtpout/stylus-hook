// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;


import "forge-std/Script.sol";
import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {PoolManager} from "v4-core/src/PoolManager.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolModifyLiquidityTest} from "v4-core/src/test/PoolModifyLiquidityTest.sol";
import {PoolSwapTest} from "v4-core/src/test/PoolSwapTest.sol";
import {PoolDonateTest} from "v4-core/src/test/PoolDonateTest.sol";
import {Counter} from "../src/Counter.sol";
import {HookMiner} from "../test/utils/HookMiner.sol";

contract CounterScript is Script {
    address constant CREATE2_DEPLOYER =
        address(0x4e59b44847b379578588920cA78FbF26c0B4956C);
    address constant GOERLI_POOLMANAGER =
        address(0x7B5cd09c909891b26305d37CA8731E442120ceD2);

    function setUp() public {}

    function run() public {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;

        vm.startBroadcast(privateKey);
        // hook contracts must have specific flags encoded in the address
        uint160 flags = uint160(
            Hooks.BEFORE_SWAP_FLAG |
                Hooks.AFTER_SWAP_FLAG |
                Hooks.BEFORE_ADD_LIQUIDITY_FLAG |
                Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
        );

        // Mine a salt that will produce a hook address with the correct flags
        (address hookAddress, bytes32 salt) = HookMiner.find(
            CREATE2_DEPLOYER,
            flags,
            type(Counter).creationCode,
            abi.encode(address(GOERLI_POOLMANAGER))
        );

        // Deploy the hook using CREATE2
        Counter counter = new Counter{salt: salt}(
            IPoolManager(address(GOERLI_POOLMANAGER))
        );
        require(
            address(counter) == hookAddress,
            "CounterScript: hook address mismatch"
        );

        vm.stopBroadcast();
    }
}
