// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;


import "forge-std/Script.sol";
import "forge-std/console.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {PoolModifyLiquidityTest} from "v4-core/src/test/PoolModifyLiquidityTest.sol";
import {CurrencyLibrary, Currency} from "v4-core/src/types/Currency.sol";
import {IHooks} from "v4-core/src/interfaces/IHooks.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";

contract AddLiquidityScript is Script {
    using CurrencyLibrary for Currency;

    address constant GOERLI_POOLMANAGER =
        address(0x7B5cd09c909891b26305d37CA8731E442120ceD2); // pool manager deployed to GOERLI
    address constant MUNI_ADDRESS =
        address(0x0c56F1e4e1fA9F3f2Fe8bF8865fDbC001204859C); // mUNI deployed to GOERLI -- insert your own contract address here
    address constant MUSDC_ADDRESS =
        address(0xed723EF571F990d967EB0eD31a50351d6818446a); // mUSDC deployed to GOERLI -- insert your own contract address here
    address constant HOOK_ADDRESS =
        address(0x2b0e534B1BD6cd85C036E7b33856EA15c203403b); // address of the hook contract deployed to goerli -- you can use this hook address or deploy your own!

    PoolModifyLiquidityTest lpRouter =
        PoolModifyLiquidityTest(
            address(0xB8368640D1aB0684FA0c670C3c1F74831764F591)
        );

    function run() external {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;

        vm.startBroadcast(privateKey);
        // sort the tokens!
        address token0 = uint160(MUSDC_ADDRESS) < uint160(MUNI_ADDRESS)
            ? MUSDC_ADDRESS
            : MUNI_ADDRESS;
        address token1 = uint160(MUSDC_ADDRESS) < uint160(MUNI_ADDRESS)
            ? MUNI_ADDRESS
            : MUSDC_ADDRESS;
        uint24 swapFee = 4000; // 0.40% fee tier
        int24 tickSpacing = 10;

        PoolKey memory pool = PoolKey({
            currency0: Currency.wrap(token0),
            currency1: Currency.wrap(token1),
            fee: swapFee,
            tickSpacing: tickSpacing,
            hooks: IHooks(HOOK_ADDRESS)
        });

        // approve tokens to the LP Router
        IERC20(token0).approve(address(lpRouter), 1000e18);
        IERC20(token1).approve(address(lpRouter), 1000e18);

        // optionally specify hookData if the hook depends on arbitrary data for liquidity modification
        bytes memory hookData = new bytes(0);

        // logging the pool ID
        PoolId id = PoolIdLibrary.toId(pool);
        bytes32 idBytes = PoolId.unwrap(id);
        console.log("Pool ID Below");
        console.logBytes32(bytes32(idBytes));

        // Provide 10_000e18 worth of liquidity on the range of [-600, 600]

        lpRouter.modifyLiquidity(
            pool,
            IPoolManager.ModifyLiquidityParams(-600, 600, 10_000e18),
            hookData
        );

        vm.stopBroadcast();
    }
}
