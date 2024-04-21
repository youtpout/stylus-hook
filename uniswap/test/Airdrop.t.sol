// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
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
import {AirdropHook} from "../src/AirdropHook.sol";
import {AirdropToken} from "../src/AirdropToken.sol";
import {HookMiner} from "./utils/HookMiner.sol";

contract AirdropTest is Test, Deployers {
    using PoolIdLibrary for PoolKey;
    using CurrencyLibrary for Currency;

    AirdropHook hook;
    AirdropToken token;
    PoolId poolId;

    address deployer = makeAddr("Deployer");
    address alice = makeAddr("Alice");
    address bob = makeAddr("Bob");
    address charlie = makeAddr("Charlie");
    address daniel = makeAddr("Daniel");

    function setUp() public {
        // creates the pool manager, utility routers, and test tokens
        Deployers.deployFreshManagerAndRouters();
        Deployers.deployMintAndApprove2Currencies();

        // Deploy the hook to an address with the correct flags
        uint160 flags = uint160(Hooks.AFTER_SWAP_FLAG);
        (address hookAddress, bytes32 salt) = HookMiner.find(
            address(this),
            flags,
            type(AirdropHook).creationCode,
            abi.encode(address(manager))
        );
        hook = new AirdropHook{salt: salt}(IPoolManager(address(manager)));
        require(
            address(hook) == hookAddress,
            "AirdropHook: hook address mismatch"
        );

        uint256 amountAirdrop = 50000000 * 10 ** 18;
        token = new AirdropToken(
            "Flydrop",
            "FLY",
            deployer,
            address(hook),
            amountAirdrop
        );

        // Create the pool
        key = PoolKey(currency0, currency1, 3000, 60, IHooks(address(hook)));
        poolId = key.toId();
        manager.initialize(key, SQRT_RATIO_1_1, ZERO_BYTES);

        // Provide liquidity to the pool
        modifyLiquidityRouter.modifyLiquidity(
            key,
            IPoolManager.ModifyLiquidityParams(-60, 60, 10 ether),
            ZERO_BYTES
        );
        modifyLiquidityRouter.modifyLiquidity(
            key,
            IPoolManager.ModifyLiquidityParams(-120, 120, 10 ether),
            ZERO_BYTES
        );
        modifyLiquidityRouter.modifyLiquidity(
            key,
            IPoolManager.ModifyLiquidityParams(
                TickMath.minUsableTick(60),
                TickMath.maxUsableTick(60),
                10 ether
            ),
            ZERO_BYTES
        );
    }

    function testAirdropHookTotal() public {
        // positions were created in setup()
        (
            uint256 amount0,
            uint256 amount1,
            uint256 counter0,
            uint256 counter1
        ) = hook.totalSwap(poolId);
        assertEq(amount0, 0);
        assertEq(amount1, 0);
        assertEq(counter0, 0);
        assertEq(counter1, 0);

        // Perform a test swap //
        bool zeroForOne = true;
        int256 amountSpecified = -1e18; // negative number indicates exact input swap!
        BalanceDelta swapDelta = swap(
            key,
            zeroForOne,
            amountSpecified,
            ZERO_BYTES
        );
        // ------------------- //

        assertEq(int256(swapDelta.amount0()), amountSpecified);

        (amount0, amount1, counter0, counter1) = hook.totalSwap(poolId);
        uint256 total1 = uint256(-amountSpecified);
        console.log(amount1);
        assertEq(amount0, 0);
        assertEq(amount1, total1);
        assertEq(counter0, 0);
        assertEq(counter1, 1);

        bytes memory res = abi.encode(hook.getHookPermissions());
        console.logBytes(res);

        // Perform a second swap //
        zeroForOne = false;
        amountSpecified = -1e18; // negative number indicates exact input swap!
        swapDelta = swap(key, zeroForOne, amountSpecified, ZERO_BYTES);
        // ------------------- //

        assertEq(int256(swapDelta.amount1()), amountSpecified);

        uint256 total0 = uint256(-amountSpecified);
        (amount0, amount1, counter0, counter1) = hook.totalSwap(poolId);
        console.log(amount1);
        assertEq(amount0, total0);
        assertEq(amount1, total1);
        assertEq(counter0, 1);
        assertEq(counter1, 1);
    }
}
