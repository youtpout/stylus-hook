// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;


import "forge-std/Script.sol";
import "forge-std/console.sol";
import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {PoolManager} from "v4-core/src/PoolManager.sol";
import {IHooks} from "v4-core/src/interfaces/IHooks.sol";
import {PoolKey} from "v4-core/src/types/PoolKey.sol";
import {CurrencyLibrary, Currency} from "v4-core/src/types/Currency.sol";
import {PoolId, PoolIdLibrary} from "v4-core/src/types/PoolId.sol";

contract CreatePoolScript is Script {
    using CurrencyLibrary for Currency;

    //addresses with contracts deployed
    address constant GOERLI_POOLMANAGER =
        address(0x7B5cd09c909891b26305d37CA8731E442120ceD2); //pool manager deployed to GOERLI
    address constant MUNI_ADDRESS =
        address(0x0c56F1e4e1fA9F3f2Fe8bF8865fDbC001204859C); //mUNI deployed to GOERLI -- insert your own contract address here
    address constant MUSDC_ADDRESS =
        address(0xed723EF571F990d967EB0eD31a50351d6818446a); //mUSDC deployed to GOERLI -- insert your own contract address here
    address constant HOOK_ADDRESS =
        address(0x2b0e534B1BD6cd85C036E7b33856EA15c203403b); //address of the hook contract deployed to goerli -- you can use this hook address or deploy your own!

    IPoolManager manager = IPoolManager(GOERLI_POOLMANAGER);

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
        uint24 swapFee = 4000;
        int24 tickSpacing = 10;

        // floor(sqrt(1) * 2^96)
        uint160 startingPrice = 79228162514264337593543950336;

        bytes memory hookData = abi.encode(block.timestamp);

        PoolKey memory pool = PoolKey({
            currency0: Currency.wrap(token0),
            currency1: Currency.wrap(token1),
            fee: swapFee,
            tickSpacing: tickSpacing,
            hooks: IHooks(HOOK_ADDRESS)
        });

        // Turn the Pool into an ID so you can use it for modifying positions, swapping, etc.
        PoolId id = PoolIdLibrary.toId(pool);
        bytes32 idBytes = PoolId.unwrap(id);

        console.log("Pool ID Below");
        console.logBytes32(bytes32(idBytes));

        manager.initialize(pool, startingPrice, hookData);
        vm.stopBroadcast();
    }
}
