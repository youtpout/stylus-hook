// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";

import {MockERC20} from "solmate/src/test/utils/mocks/MockERC20.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {IPositionManager} from "@uniswap/v4-periphery/src/interfaces/IPositionManager.sol";
import {HookMiner} from "@uniswap/v4-periphery/src/utils/HookMiner.sol";
import {IPermit2} from "permit2/src/interfaces/IPermit2.sol";

import {IUniswapV4Router04} from "hookmate/interfaces/router/IUniswapV4Router04.sol";
import {Permit2Deployer} from "hookmate/artifacts/Permit2.sol";
import {V4PoolManagerDeployer} from "hookmate/artifacts/V4PoolManager.sol";
import {V4PositionManagerDeployer} from "hookmate/artifacts/V4PositionManager.sol";
import {V4RouterDeployer} from "hookmate/artifacts/V4Router.sol";

import {ComputeHook} from "../../src/ComputeHook.sol";
import {NativeHookFixture} from "./NativeHookFixture.sol";
import {StylusDeployer} from "./vendor/StylusDeployer.sol";

/// @notice The Solidity half of the compute benchmark: a v4 deployment and `ComputeHook.sol`.
///
/// Its Rust twin cannot be deployed from here — it needs a CREATE2 salt mined against its WASM init
/// code, and forge cannot call it afterwards either. `bench-compute.bash` does that part.
contract DeployComputeBenchScript is Script {
    uint256 internal constant FIXTURE_FUNDS = 1_000_000 ether;

    IPermit2 internal permit2;
    IPoolManager internal poolManager;
    IPositionManager internal positionManager;
    IUniswapV4Router04 internal router;

    ComputeHook internal solidityHook;

    function _deployComputeHook() private {
        uint160 flags = uint160(Hooks.BEFORE_SWAP_FLAG);
        address create2Deployer = vm.envAddress("CREATE2_DEPLOYER");

        (address solidityAddress, bytes32 soliditySalt) =
            HookMiner.find(create2Deployer, flags, type(ComputeHook).creationCode, abi.encode(poolManager));
        solidityHook = new ComputeHook{salt: soliditySalt}(poolManager);
        require(address(solidityHook) == solidityAddress, "compute hook address mismatch");
    }

    function run() public {
        vm.startBroadcast();

        StylusDeployer stylusDeployer = new StylusDeployer();

        permit2 = IPermit2(address(Permit2Deployer.deploy()));
        poolManager = IPoolManager(V4PoolManagerDeployer.deploy(msg.sender));
        positionManager = IPositionManager(
            V4PositionManagerDeployer.deploy(address(poolManager), address(permit2), 300_000, address(0), address(0))
        );
        router = IUniswapV4Router04(payable(V4RouterDeployer.deploy(address(poolManager), address(permit2))));

        _deployComputeHook();

        MockERC20 tokenA = new MockERC20("Bench A", "BA", 18);
        MockERC20 tokenB = new MockERC20("Bench B", "BB", 18);
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
        console.log("currency0        :", address(token0));
        console.log("currency1        :", address(token1));
        console.log("fixture          :", address(fixture));
        console.log("ComputeHook (sol):", address(solidityHook));
    }
}
