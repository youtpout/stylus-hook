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

contract DeployHookScript is Script {
    using PoolIdLibrary for PoolKey;
    using CurrencyLibrary for Currency;

    Counter counter;
    PoolId poolId;
    PoolManager manager;
    PoolKey key;
    PoolModifyLiquidityTest lpRouter;
    PoolSwapTest swapRouter;
    address createProxy;

    Token MUNI_ADDRESS;
    Token MUSDC_ADDRESS;

    uint160 public constant MIN_PRICE_LIMIT = TickMath.MIN_SQRT_RATIO + 1;
    uint160 public constant MAX_PRICE_LIMIT = TickMath.MAX_SQRT_RATIO - 1;

    bytes constant ZERO_BYTES = new bytes(0);

    function run() external {
        uint256 privateKey = 0x0123456789012345678901234567890123456789012345678901234567890123;
        address deployer = 0x14791697260E4c9A71f18484C9f997B308e59325;

         bytes
            memory bytecode = hex"602080600a5f395ff3fe36601f19018060205f375f35905f34f58015601c575f526014600cf35b5f80fd";

        createProxy = deployBytecode(bytecode);
        console.log("proxy %s", createProxy);

        vm.startBroadcast(privateKey);
        manager = new PoolManager(500000);
        lpRouter = new PoolModifyLiquidityTest(manager);
        swapRouter = new PoolSwapTest(manager);

        MUNI_ADDRESS = new Token("MUNI", "MUNI", deployer);
        MUSDC_ADDRESS = new Token("MUSDC", " MUSDC", deployer);

        // Deploy the hook to an address with the correct flags
        uint160 flags = uint160(
            Hooks.BEFORE_SWAP_FLAG |
                Hooks.AFTER_SWAP_FLAG |
                Hooks.BEFORE_ADD_LIQUIDITY_FLAG |
                Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
        );
        (address hookAddress, bytes32 salt) = HookMiner.find(
            address(createProxy),
            flags,
            type(Counter).creationCode,
            abi.encode(address(manager))
        );
        counter = new Counter{salt: salt}(IPoolManager(address(manager)));
        require(
            address(counter) == hookAddress,
            "CounterTest: hook address mismatch"
        );

        address token0 = uint160(address(MUSDC_ADDRESS)) <
            uint160(address(MUNI_ADDRESS))
            ? address(MUSDC_ADDRESS)
            : address(MUNI_ADDRESS);
        address token1 = uint160(address(MUSDC_ADDRESS)) <
            uint160(address(MUNI_ADDRESS))
            ? address(MUNI_ADDRESS)
            : address(MUSDC_ADDRESS);

        // Create the pool
        key = PoolKey(
            Currency.wrap(token0),
            Currency.wrap(token1),
            3000,
            60,
            IHooks(address(counter))
        );
        poolId = key.toId();

        bytes memory hookData = abi.encode(block.timestamp);
        // floor(sqrt(1) * 2^96)
        uint160 startingPrice = 79228162514264337593543950336;
        manager.initialize(key, startingPrice, hookData);

        bytes32 idBytes = PoolId.unwrap(poolId);

        console.log("Pool ID Below");
        console.logBytes32(bytes32(idBytes));

        // Provide liquidity to the pool
        IERC20(token0).approve(address(lpRouter), 1000e18);
        IERC20(token1).approve(address(lpRouter), 1000e18);
        IERC20(token0).approve(address(swapRouter), 1000e18);
        IERC20(token1).approve(address(swapRouter), 1000e18);

        lpRouter.modifyLiquidity(
            key,
            IPoolManager.ModifyLiquidityParams(-600, 600, 10_000e18),
            hookData
        );

        vm.stopBroadcast();
    }

      function deployBytecode(
        bytes memory bytecode
    ) internal returns (address deployedAddress) {
        deployedAddress;
        assembly {
            deployedAddress := create(0, add(bytecode, 0x20), mload(bytecode))
        }

        ///@notice check that the deployment was successful
        require(
            deployedAddress != address(0),
            "YulDeployer could not deploy contract"
        );
    }
}
