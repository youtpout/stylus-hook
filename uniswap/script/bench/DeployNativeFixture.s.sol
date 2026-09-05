// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";

import {MockERC20} from "solmate/src/test/utils/mocks/MockERC20.sol";

import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {Currency} from "@uniswap/v4-core/src/types/Currency.sol";
import {IPositionManager} from "@uniswap/v4-periphery/src/interfaces/IPositionManager.sol";
import {IPermit2} from "permit2/src/interfaces/IPermit2.sol";

import {IUniswapV4Router04} from "hookmate/interfaces/router/IUniswapV4Router04.sol";
import {Permit2Deployer} from "hookmate/artifacts/Permit2.sol";
import {V4PoolManagerDeployer} from "hookmate/artifacts/V4PoolManager.sol";
import {V4PositionManagerDeployer} from "hookmate/artifacts/V4PositionManager.sol";
import {V4RouterDeployer} from "hookmate/artifacts/V4Router.sol";

import {NativeHookFixture} from "./NativeHookFixture.sol";
import {StylusDeployer} from "./vendor/StylusDeployer.sol";

/// @notice Everything a Stylus hook needs around it, on a chain that has none of it: the v4
///         singleton and its periphery, two tokens, a `StylusDeployer` to CREATE2 the hook with,
///         and a fixture that can open a pool and swap without forge ever simulating the hook.
contract DeployNativeFixtureScript is Script {
    uint256 internal constant FIXTURE_FUNDS = 1_000_000 ether;

    function run() public {
        vm.startBroadcast();

        // cargo stylus deploy routes through this to CREATE2 a Stylus contract; the canonical one
        // only exists on real Arbitrum chains.
        StylusDeployer stylusDeployer = new StylusDeployer();

        IPermit2 permit2 = IPermit2(address(Permit2Deployer.deploy()));
        IPoolManager poolManager = IPoolManager(V4PoolManagerDeployer.deploy(msg.sender));
        IPositionManager positionManager = IPositionManager(
            V4PositionManagerDeployer.deploy(address(poolManager), address(permit2), 300_000, address(0), address(0))
        );
        IUniswapV4Router04 router =
            IUniswapV4Router04(payable(V4RouterDeployer.deploy(address(poolManager), address(permit2))));

        MockERC20 tokenA = new MockERC20("Native A", "NA", 18);
        MockERC20 tokenB = new MockERC20("Native B", "NB", 18);
        (MockERC20 token0, MockERC20 token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);

        address[] memory tokens = new address[](2);
        tokens[0] = address(token0);
        tokens[1] = address(token1);

        NativeHookFixture fixture =
            new NativeHookFixture(poolManager, positionManager, permit2, router, tokens);

        token0.mint(address(fixture), FIXTURE_FUNDS);
        token1.mint(address(fixture), FIXTURE_FUNDS);

        vm.stopBroadcast();

        console.log("stylusDeployer   :", address(stylusDeployer));
        console.log("poolManager      :", address(poolManager));
        console.log("positionManager  :", address(positionManager));
        console.log("swapRouter       :", address(router));
        console.log("currency0        :", address(token0));
        console.log("currency1        :", address(token1));
        console.log("fixture          :", address(fixture));
    }
}
