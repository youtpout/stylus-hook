// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "forge-std/Script.sol";
import {IHooks} from "v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "v4-core/src/libraries/Hooks.sol";
import {TickMath} from "v4-core/src/libraries/TickMath.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {BalanceDelta} from "v4-core/src/types/BalanceDelta.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";
import {CurrencyLibrary, Currency} from "v4-core/src/types/Currency.sol";
import {PoolSwapTest} from "v4-core/src/test/PoolSwapTest.sol";
import {Deployers} from "v4-core/test/utils/Deployers.sol";
import {Counter} from "../src/Counter.sol";
import {HookMiner} from "../test/utils/HookMiner.sol";
import {PoolManager} from "v4-core/src/PoolManager.sol";
import {PoolModifyLiquidityTest} from "v4-core/src/test/PoolModifyLiquidityTest.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {Token} from "../src/Token.sol";

contract ContractScript is Script {

    Counter counter;
    PoolId poolId;
    PoolManager manager;
    PoolKey key;
    PoolModifyLiquidityTest lpRouter;
    PoolSwapTest swapRouter;

    Token MUNI_ADDRESS;
    Token MUSDC_ADDRESS;

    function run() external {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;
        address deployer = 0x14791697260E4c9A71f18484C9f997B308e59325;

        vm.startBroadcast(privateKey);
        manager = new PoolManager(500000);
        lpRouter = new PoolModifyLiquidityTest(manager);
        swapRouter = new PoolSwapTest(manager);

        console.logString(
            string.concat(
                "manager deployed at: ",
                vm.toString(address(manager))
            )
        );
        console.logString(
            string.concat(
                "lpRouter deployed at: ",
                vm.toString(address(lpRouter))
            )
        );
        console.logString(
            string.concat(
                "swapRouter deployed at: ",
                vm.toString(address(swapRouter))
            )
        );

        MUNI_ADDRESS = new Token("MUNI", "MUNI", deployer);
        MUSDC_ADDRESS = new Token("MUSDC", " MUSDC", deployer);
        console.logString(
            string.concat(
                "MUNI deployed at: ",
                vm.toString(address(MUNI_ADDRESS))
            )
        );
        console.logString(
            string.concat(
                "MUSDC deployed at: ",
                vm.toString(address(MUSDC_ADDRESS))
            )
        );
     
        vm.stopBroadcast();
    }
}
