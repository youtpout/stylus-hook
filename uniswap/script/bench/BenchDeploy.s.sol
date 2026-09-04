// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";

import {MockERC20} from "solmate/src/test/utils/mocks/MockERC20.sol";

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {TickMath} from "@uniswap/v4-core/src/libraries/TickMath.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {Currency} from "@uniswap/v4-core/src/types/Currency.sol";
import {LiquidityAmounts} from "@uniswap/v4-core/test/utils/LiquidityAmounts.sol";
import {Constants} from "@uniswap/v4-core/test/utils/Constants.sol";
import {HookMiner} from "@uniswap/v4-periphery/src/utils/HookMiner.sol";
import {IPositionManager} from "@uniswap/v4-periphery/src/interfaces/IPositionManager.sol";
import {IPermit2} from "permit2/src/interfaces/IPermit2.sol";

import {IUniswapV4Router04} from "hookmate/interfaces/router/IUniswapV4Router04.sol";
import {Permit2Deployer} from "hookmate/artifacts/Permit2.sol";
import {V4PoolManagerDeployer} from "hookmate/artifacts/V4PoolManager.sol";
import {V4PositionManagerDeployer} from "hookmate/artifacts/V4PositionManager.sol";
import {V4RouterDeployer} from "hookmate/artifacts/V4Router.sol";

import {Actions} from "@uniswap/v4-periphery/src/libraries/Actions.sol";
import {AirdropHook} from "../../src/AirdropHook.sol";
import {AirdropHookProxy} from "../../src/AirdropHookProxy.sol";
import {IAirdropHook} from "../../src/interfaces/IAirdropHook.sol";
import {MockStylusAirdrop} from "../../test/mocks/MockStylusAirdrop.sol";
import {SwapBench} from "./SwapBench.sol";

/// @notice Stands up a complete v4 deployment plus the three airdrop-hook variants on a chain that
///         does not have one, then wires a `SwapBench` that can swap through each in its own
///         transaction.
///
/// Meant for a local Arbitrum Nitro dev node, which is the cheapest chain that can execute Stylus.
/// Deploy the Stylus contract first and pass it as `STYLUS_AIRDROP`; `CREATE2_DEPLOYER` must hold
/// the deterministic-deployment-proxy (its canonical presigned transaction cannot be replayed on
/// Arbitrum, so deploy a copy and pass the address here and to `--create2-deployer`).
contract BenchDeployScript is Script {
    uint128 internal constant LIQUIDITY = 1000e18;
    uint256 internal constant BENCH_FUNDS = 100_000 ether;

    IPermit2 internal permit2;
    IPoolManager internal poolManager;
    IPositionManager internal positionManager;
    IUniswapV4Router04 internal swapRouter;

    AirdropHook internal solidityHook;
    AirdropHookProxy internal stylusHook;
    AirdropHookProxy internal replicaHook;
    Currency internal currency0;
    Currency internal currency1;

    function run() public {
        address create2Deployer = vm.envAddress("CREATE2_DEPLOYER");
        IAirdropHook stylusAirdrop = IAirdropHook(vm.envAddress("STYLUS_AIRDROP"));
        require(address(stylusAirdrop).code.length > 0, "STYLUS_AIRDROP has no code");

        vm.startBroadcast();

        // 1. the v4 stack
        permit2 = IPermit2(address(Permit2Deployer.deploy()));
        poolManager = IPoolManager(V4PoolManagerDeployer.deploy(msg.sender));
        positionManager = IPositionManager(
            V4PositionManagerDeployer.deploy(address(poolManager), address(permit2), 300_000, address(0), address(0))
        );
        swapRouter = IUniswapV4Router04(payable(V4RouterDeployer.deploy(address(poolManager), address(permit2))));

        console.log("permit2          :", address(permit2));
        console.log("poolManager      :", address(poolManager));
        console.log("positionManager  :", address(positionManager));
        console.log("swapRouter       :", address(swapRouter));

        // 2. tokens
        _deployCurrencyPair();

        // 3. the hooks, each mined to an address carrying AFTER_SWAP_FLAG
        _deployHooks(create2Deployer, stylusAirdrop);

        // 4. one pool per variant, plus a hookless control
        PoolKey memory noHookPool = _openPool(IHooks(address(0)));
        PoolKey memory solidityPool = _openPool(IHooks(address(solidityHook)));
        PoolKey memory stylusPool = _openPool(IHooks(address(stylusHook)));
        PoolKey memory replicaPool = _openPool(IHooks(address(replicaHook)));

        // 5. the bench, funded and approved
        address[] memory tokens = new address[](2);
        tokens[0] = Currency.unwrap(currency0);
        tokens[1] = Currency.unwrap(currency1);
        SwapBench bench = new SwapBench(swapRouter, address(permit2), address(poolManager), tokens);

        MockERC20(tokens[0]).transfer(address(bench), BENCH_FUNDS);
        MockERC20(tokens[1]).transfer(address(bench), BENCH_FUNDS);

        bytes memory beneficiary = abi.encode(address(bench));
        bench.addPool(noHookPool, "");
        bench.addPool(solidityPool, beneficiary);
        bench.addPool(stylusPool, beneficiary);
        bench.addPool(replicaPool, beneficiary);

        vm.stopBroadcast();

        console.log("SwapBench        :", address(bench));
        console.log("  pool 0 = no hook");
        console.log("  pool 1 = AirdropHook, one Solidity contract");
        console.log("  pool 2 = AirdropHookProxy -> Stylus");
        console.log("  pool 3 = AirdropHookProxy -> Solidity replica of the Stylus contract");
    }

    function _deployHooks(address create2Deployer, IAirdropHook stylusAirdrop) private {
        uint160 flags = uint160(Hooks.AFTER_SWAP_FLAG);

        (address solidityAddress, bytes32 soliditySalt) =
            HookMiner.find(create2Deployer, flags, type(AirdropHook).creationCode, abi.encode(poolManager));
        solidityHook = new AirdropHook{salt: soliditySalt}(poolManager);
        require(address(solidityHook) == solidityAddress, "solidity hook address mismatch");

        (address stylusAddress, bytes32 stylusSalt) = HookMiner.find(
            create2Deployer, flags, type(AirdropHookProxy).creationCode, abi.encode(poolManager, stylusAirdrop)
        );
        stylusHook = new AirdropHookProxy{salt: stylusSalt}(poolManager, stylusAirdrop);
        require(address(stylusHook) == stylusAddress, "stylus hook address mismatch");
        // `stylusAirdrop.setHook` is deliberately NOT called here: forge executes EVM bytecode, and
        // an activated Stylus contract's code starts with the 0xEF prefix, which the local EVM
        // rejects as an invalid opcode. Bind the two with a plain transaction instead:
        //   cast send $STYLUS_AIRDROP "setHook(address)" <AirdropHookProxy>

        // A third variant: the same two-contract split, but with the Solidity replica of the Rust
        // contract as the callee. The gap between this and the Stylus one is what Stylus actually
        // costs, with the extra CALL and the identical storage writes cancelling out.
        MockStylusAirdrop replica = new MockStylusAirdrop();
        (address replicaAddress, bytes32 replicaSalt) = HookMiner.find(
            create2Deployer, flags, type(AirdropHookProxy).creationCode, abi.encode(poolManager, replica)
        );
        replicaHook = new AirdropHookProxy{salt: replicaSalt}(poolManager, IAirdropHook(address(replica)));
        require(address(replicaHook) == replicaAddress, "replica hook address mismatch");
        replica.setHook(address(replicaHook));

        console.log("AirdropHook (sol):", address(solidityHook));
        console.log("AirdropHookProxy :", address(stylusHook));
        console.log("Stylus contract  :", address(stylusAirdrop));
        console.log("split w/ replica :", address(replicaHook));
    }

    function _deployCurrencyPair() private {
        MockERC20 tokenA = new MockERC20("Bench A", "BA", 18);
        MockERC20 tokenB = new MockERC20("Bench B", "BB", 18);
        (MockERC20 token0, MockERC20 token1) = tokenA < tokenB ? (tokenA, tokenB) : (tokenB, tokenA);

        for (uint256 i = 0; i < 2; i++) {
            MockERC20 token = i == 0 ? token0 : token1;
            token.mint(msg.sender, 10_000_000 ether);
            token.approve(address(permit2), type(uint256).max);
            token.approve(address(swapRouter), type(uint256).max);
            permit2.approve(address(token), address(positionManager), type(uint160).max, type(uint48).max);
            permit2.approve(address(token), address(poolManager), type(uint160).max, type(uint48).max);
        }

        currency0 = Currency.wrap(address(token0));
        currency1 = Currency.wrap(address(token1));
        console.log("currency0        :", address(token0));
        console.log("currency1        :", address(token1));
    }

    function _openPool(IHooks hooks) private returns (PoolKey memory key) {
        key = PoolKey(currency0, currency1, 3000, 60, hooks);
        poolManager.initialize(key, Constants.SQRT_PRICE_1_1);

        int24 tickLower = TickMath.minUsableTick(key.tickSpacing);
        int24 tickUpper = TickMath.maxUsableTick(key.tickSpacing);
        (uint256 amount0, uint256 amount1) = LiquidityAmounts.getAmountsForLiquidity(
            Constants.SQRT_PRICE_1_1,
            TickMath.getSqrtPriceAtTick(tickLower),
            TickMath.getSqrtPriceAtTick(tickUpper),
            LIQUIDITY
        );

        // EasyPosm cannot be used here: it reads `address(this)` to compute balance deltas, and a
        // script contract is ephemeral, so forge rejects it. Drive the position manager directly.
        bytes memory actions = abi.encodePacked(
            uint8(Actions.MINT_POSITION), uint8(Actions.SETTLE_PAIR), uint8(Actions.SWEEP), uint8(Actions.SWEEP)
        );
        bytes[] memory params = new bytes[](4);
        params[0] =
            abi.encode(key, tickLower, tickUpper, LIQUIDITY, amount0 + 1, amount1 + 1, msg.sender, bytes(""));
        params[1] = abi.encode(key.currency0, key.currency1);
        params[2] = abi.encode(key.currency0, msg.sender);
        params[3] = abi.encode(key.currency1, msg.sender);

        positionManager.modifyLiquidities(abi.encode(actions, params), block.timestamp + 3600);
    }
}
