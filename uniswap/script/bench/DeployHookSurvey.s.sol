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

import {LimitOrderHookMock} from "@openzeppelin/uniswap-hooks/src/mocks/general/LimitOrderHookMock.sol";
import {BaseOracleHookMock} from "@openzeppelin/uniswap-hooks/src/mocks/oracles/panoptic/BaseOracleHookMock.sol";

import {ArbAntiSandwichMock} from "./ArbAntiSandwichMock.sol";
import {NativeHookFixture} from "./NativeHookFixture.sol";

/// @notice Deploys the shipping hooks worth pricing, each at a mined address, on one v4 stack.
///
/// `profile-hooks.bash` then swaps through each and traces the transaction opcode by opcode, so the
/// gas each one adds can be split into arithmetic — the only part Stylus makes cheaper — and
/// storage and calls, which it does not.
contract DeployHookSurveyScript is Script {
    uint256 internal constant FIXTURE_FUNDS = 1_000_000 ether;

    IPermit2 internal permit2;
    IPoolManager internal poolManager;
    IPositionManager internal positionManager;
    IUniswapV4Router04 internal router;

    function run() public {
        address create2Deployer = vm.envAddress("CREATE2_DEPLOYER");

        vm.startBroadcast();

        permit2 = IPermit2(address(Permit2Deployer.deploy()));
        poolManager = IPoolManager(V4PoolManagerDeployer.deploy(msg.sender));
        positionManager = IPositionManager(
            V4PositionManagerDeployer.deploy(address(poolManager), address(permit2), 300_000, address(0), address(0))
        );
        router = IUniswapV4Router04(payable(V4RouterDeployer.deploy(address(poolManager), address(permit2))));

        address antiSandwich = _mineAndDeployAntiSandwich(create2Deployer);
        address limitOrder = _mineAndDeployLimitOrder(create2Deployer);
        address oracle = _mineAndDeployOracle(create2Deployer);

        MockERC20 tokenA = new MockERC20("Survey A", "SA", 18);
        MockERC20 tokenB = new MockERC20("Survey B", "SB", 18);
        (MockERC20 token0, MockERC20 token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);
        address[] memory tokens = new address[](2);
        tokens[0] = address(token0);
        tokens[1] = address(token1);

        NativeHookFixture fixture =
            new NativeHookFixture(poolManager, positionManager, permit2, router, tokens);
        token0.mint(address(fixture), FIXTURE_FUNDS);
        token1.mint(address(fixture), FIXTURE_FUNDS);

        vm.stopBroadcast();

        console.log("poolManager      :", address(poolManager));
        console.log("currency0        :", address(token0));
        console.log("currency1        :", address(token1));
        console.log("fixture          :", address(fixture));
        console.log("AntiSandwich     :", antiSandwich);
        console.log("LimitOrder       :", limitOrder);
        console.log("PanopticOracle   :", oracle);
    }

    function _mineAndDeployAntiSandwich(address create2Deployer) private returns (address) {
        uint160 flags = uint160(
            Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG | Hooks.AFTER_SWAP_RETURNS_DELTA_FLAG
        );
        (address addr, bytes32 salt) = HookMiner.find(
            create2Deployer, flags, type(ArbAntiSandwichMock).creationCode, abi.encode(poolManager)
        );
        ArbAntiSandwichMock hook = new ArbAntiSandwichMock{salt: salt}(poolManager);
        require(address(hook) == addr, "anti-sandwich mismatch");
        return addr;
    }

    function _mineAndDeployLimitOrder(address create2Deployer) private returns (address) {
        uint160 flags = uint160(Hooks.AFTER_INITIALIZE_FLAG | Hooks.AFTER_SWAP_FLAG);
        (address addr, bytes32 salt) = HookMiner.find(
            create2Deployer, flags, type(LimitOrderHookMock).creationCode, abi.encode(poolManager)
        );
        LimitOrderHookMock hook = new LimitOrderHookMock{salt: salt}(poolManager);
        require(address(hook) == addr, "limit order mismatch");
        return addr;
    }

    function _mineAndDeployOracle(address create2Deployer) private returns (address) {
        uint160 flags = uint160(Hooks.AFTER_INITIALIZE_FLAG | Hooks.BEFORE_SWAP_FLAG);
        (address addr, bytes32 salt) = HookMiner.find(
            create2Deployer, flags, type(BaseOracleHookMock).creationCode, abi.encode(poolManager, int24(9116))
        );
        BaseOracleHookMock hook = new BaseOracleHookMock{salt: salt}(poolManager, int24(9116));
        require(address(hook) == addr, "oracle mismatch");
        return addr;
    }
}
