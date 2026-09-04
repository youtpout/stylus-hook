// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {console} from "forge-std/console.sol";

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {TickMath} from "@uniswap/v4-core/src/libraries/TickMath.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {PoolId} from "@uniswap/v4-core/src/types/PoolId.sol";
import {Currency} from "@uniswap/v4-core/src/types/Currency.sol";
import {LiquidityAmounts} from "@uniswap/v4-core/test/utils/LiquidityAmounts.sol";
import {Constants} from "@uniswap/v4-core/test/utils/Constants.sol";

import {EasyPosm} from "./utils/libraries/EasyPosm.sol";
import {BaseTest} from "./utils/BaseTest.sol";

import {AirdropHook} from "../src/AirdropHook.sol";
import {AirdropHookProxy} from "../src/AirdropHookProxy.sol";
import {MockStylusAirdrop} from "./mocks/MockStylusAirdrop.sol";

/// @notice Gas cost of the airdrop hook's `afterSwap`, measured against the same swap on a pool
///         with no hook at all.
///
/// IMPORTANT: forge executes EVM bytecode, not WASM. The `split` row here runs
/// `AirdropHookProxy` against `MockStylusAirdrop`, the *Solidity* replica of the Rust contract, so
/// it measures only the structural cost of the split design — one extra CALL and the calldata for
/// it — with the callee's execution priced as Solidity. The Stylus contract's own execution cost is
/// not in this number and cannot be measured here.
///
/// For that, `./bench.bash` runs the same comparison against a real Stylus chain. Its numbers for
/// the two Solidity variants track these closely, which is what makes this a useful fast check;
/// see BENCHMARK.md.
contract GasTest is BaseTest {
    Currency internal currency0;
    Currency internal currency1;

    AirdropHook internal solidityHook;
    AirdropHookProxy internal splitHook;
    MockStylusAirdrop internal stylusStandIn;

    PoolKey internal noHookPool;
    PoolKey internal solidityPool;
    PoolKey internal splitPool;

    address internal alice = makeAddr("alice");

    uint128 internal constant LIQUIDITY = 100e18;
    uint256 internal constant SWAP_AMOUNT = 1e18;

    function setUp() public {
        deployArtifactsAndLabel();
        (currency0, currency1) = deployCurrencyPair();

        solidityHook = AirdropHook(
            _deployHookTo(uint160(Hooks.AFTER_SWAP_FLAG), "AirdropHook.sol:AirdropHook", abi.encode(poolManager))
        );

        stylusStandIn = new MockStylusAirdrop();
        splitHook = AirdropHookProxy(
            _deployHookTo(
                uint160(Hooks.AFTER_SWAP_FLAG) ^ (0x1111 << 144),
                "AirdropHookProxy.sol:AirdropHookProxy",
                abi.encode(poolManager, stylusStandIn)
            )
        );
        stylusStandIn.setHook(address(splitHook));

        noHookPool = _openPool(IHooks(address(0)));
        solidityPool = _openPool(IHooks(address(solidityHook)));
        splitPool = _openPool(IHooks(address(splitHook)));
    }

    function _deployHookTo(uint160 flags, string memory artifact, bytes memory args)
        private
        returns (address hookAddress)
    {
        hookAddress = address(flags ^ (0x4444 << 144));
        deployCodeTo(artifact, args, hookAddress);
        vm.label(hookAddress, artifact);
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

        EasyPosm.mint(
            positionManager,
            key,
            tickLower,
            tickUpper,
            LIQUIDITY,
            amount0 + 1,
            amount1 + 1,
            address(this),
            block.timestamp,
            Constants.ZERO_BYTES
        );
    }

    function _swap(PoolKey memory key, bytes memory hookData) private returns (uint256 gasUsed) {
        uint256 before = gasleft();
        swapRouter.swapExactTokensForTokens({
            amountIn: SWAP_AMOUNT,
            amountOutMin: 0,
            zeroForOne: true,
            poolKey: key,
            hookData: hookData,
            receiver: address(this),
            deadline: block.timestamp + 1
        });
        gasUsed = before - gasleft();
    }

    function test_gas_afterSwap() public {
        bytes memory beneficiary = abi.encode(alice);

        // warm every slot first: the first swap on a pool pays for zero-to-non-zero writes, which
        // says more about SSTORE pricing than about the hook.
        _swap(noHookPool, Constants.ZERO_BYTES);
        _swap(solidityPool, beneficiary);
        _swap(splitPool, beneficiary);

        uint256 baseline = _swap(noHookPool, Constants.ZERO_BYTES);
        uint256 solidity = _swap(solidityPool, beneficiary);
        uint256 split = _swap(splitPool, beneficiary);

        console.log("swap, no hook              :", baseline);
        console.log("swap, AirdropHook (sol)    :", solidity);
        console.log("swap, AirdropHookProxy     :", split);
        console.log("");
        console.log("hook cost, Solidity        :", solidity - baseline);
        console.log("hook cost, split design    :", split - baseline);
        console.log("cost of the extra CALL     :", split - solidity);

        assertGt(solidity, baseline, "the hook must cost something");
        assertGt(split, solidity, "the split design pays for one more call");
    }
}
