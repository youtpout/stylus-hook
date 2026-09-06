// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IHooks} from "@uniswap/v4-core/src/interfaces/IHooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {PoolId, PoolIdLibrary} from "@uniswap/v4-core/src/types/PoolId.sol";
import {Currency} from "@uniswap/v4-core/src/types/Currency.sol";
import {TickMath} from "@uniswap/v4-core/src/libraries/TickMath.sol";
import {LiquidityAmounts} from "@uniswap/v4-core/test/utils/LiquidityAmounts.sol";
import {Constants} from "@uniswap/v4-core/test/utils/Constants.sol";
import {Actions} from "@uniswap/v4-periphery/src/libraries/Actions.sol";
import {IPositionManager} from "@uniswap/v4-periphery/src/interfaces/IPositionManager.sol";
import {IPermit2} from "permit2/src/interfaces/IPermit2.sol";
import {IUniswapV4Router04} from "hookmate/interfaces/router/IUniswapV4Router04.sol";

interface IApprove {
    function approve(address spender, uint256 amount) external returns (bool);
}

/// @notice The part of `stylus/native-twamm` this fixture needs. Not a reimplementation of the
///         hook — just the ABI of the Rust one, so its orders can be placed from on-chain.
interface ITwammOrders {
    function submitOrder(PoolKey memory key, bool zeroForOne, uint256 expiration, uint256 amountIn)
        external
        returns (bytes32);
    function claimProceeds(PoolKey memory key, bool zeroForOne, uint256 expiration)
        external
        returns (uint256);
    function executeVirtualOrders(PoolKey memory key) external;
}

/// @notice `ITWAMM.OrderKey` from the production TWAMM hook, redeclared because that contract is
///         UNLICENSED and referenced as a submodule rather than compiled into this project.
struct ProdOrderKey {
    address owner;
    uint160 expiration;
    bool zeroForOne;
}

/// @notice `ITWAMM.SubmitOrderParams` from the same. Note `duration`, where this repo's Rust hook
///         takes an absolute expiration.
struct ProdSubmitOrderParams {
    PoolKey key;
    bool zeroForOne;
    uint256 duration;
    uint256 amountIn;
}

/// @notice The part of `akshatmittal/v4-twamm-hook` this fixture calls — the TWAMM that is live on
///         Base and Unichain, used here as the Solidity control.
interface IProductionTwamm {
    function submitOrder(ProdSubmitOrderParams calldata params)
        external
        returns (bytes32 orderId, ProdOrderKey memory orderKey);
    function executeTWAMMOrders(PoolKey memory key) external;
}

/// @notice Opens a pool on a given hook and swaps through it, entirely from on-chain calls.
///
/// Every step here is reachable with a plain `cast send`, which is the point: a forge script cannot
/// touch a pool whose hook is a Stylus contract, because it simulates locally and activated Stylus
/// code starts with 0xEF — an invalid opcode to the EVM. Driving the pool through a deployed
/// contract sidesteps the simulator entirely.
contract NativeHookFixture {
    using PoolIdLibrary for PoolKey;

    IPoolManager public immutable poolManager;
    IPositionManager public immutable positionManager;
    IUniswapV4Router04 public immutable router;

    PoolKey[] public keys;
    uint256[] public tokenIds;

    constructor(
        IPoolManager _poolManager,
        IPositionManager _positionManager,
        IPermit2 _permit2,
        IUniswapV4Router04 _router,
        address[] memory tokens
    ) {
        poolManager = _poolManager;
        positionManager = _positionManager;
        router = _router;

        for (uint256 i = 0; i < tokens.length; i++) {
            IApprove(tokens[i]).approve(address(_permit2), type(uint256).max);
            IApprove(tokens[i]).approve(address(_router), type(uint256).max);
            _permit2.approve(tokens[i], address(_positionManager), type(uint160).max, type(uint48).max);
            _permit2.approve(tokens[i], address(_poolManager), type(uint160).max, type(uint48).max);
            _permit2.approve(tokens[i], address(_router), type(uint160).max, type(uint48).max);
        }
    }

    function poolCount() external view returns (uint256) {
        return keys.length;
    }

    function poolId(uint256 index) external view returns (bytes32) {
        return PoolId.unwrap(keys[index].toId());
    }

    /// @notice Initialises a pool on `hooks` and seeds it with full-range liquidity.
    /// @dev Calls the hook's `beforeAddLiquidity` if it declares one.
    function open(Currency currency0, Currency currency1, IHooks hooks, uint128 liquidity)
        external
        returns (uint256 index)
    {
        PoolKey memory key = PoolKey(currency0, currency1, 3000, 60, hooks);
        index = keys.length;
        keys.push(key);
        poolManager.initialize(key, Constants.SQRT_PRICE_1_1);

        int24 tickLower = TickMath.minUsableTick(key.tickSpacing);
        int24 tickUpper = TickMath.maxUsableTick(key.tickSpacing);
        (uint256 amount0, uint256 amount1) = LiquidityAmounts.getAmountsForLiquidity(
            Constants.SQRT_PRICE_1_1,
            TickMath.getSqrtPriceAtTick(tickLower),
            TickMath.getSqrtPriceAtTick(tickUpper),
            liquidity
        );

        bytes memory actions = abi.encodePacked(
            uint8(Actions.MINT_POSITION), uint8(Actions.SETTLE_PAIR), uint8(Actions.SWEEP), uint8(Actions.SWEEP)
        );
        bytes[] memory params = new bytes[](4);
        params[0] =
            abi.encode(key, tickLower, tickUpper, liquidity, amount0 + 1, amount1 + 1, address(this), bytes(""));
        params[1] = abi.encode(key.currency0, key.currency1);
        params[2] = abi.encode(key.currency0, address(this));
        params[3] = abi.encode(key.currency1, address(this));

        tokenIds.push(positionManager.nextTokenId());
        positionManager.modifyLiquidities(abi.encode(actions, params), block.timestamp + 3600);
    }

    /// @notice Swaps through one pool, calling its hook's `beforeSwap` and `afterSwap`.
    /// @dev One swap per transaction, so the gas each hook costs comes off the receipt.
    function swap(uint256 index, uint256 amountIn, bool zeroForOne) external {
        router.swapExactTokensForTokens({
            amountIn: amountIn,
            amountOutMin: 0,
            zeroForOne: zeroForOne,
            poolKey: keys[index],
            hookData: "",
            receiver: address(this),
            deadline: block.timestamp + 3600
        });
    }

    /// @notice Places a long-term order into a TWAMM hook.
    /// @dev The benchmark's tokens live in this fixture, so the orders have to come from here too.
    function submitTwammOrder(uint256 index, address hook, bool zeroForOne, uint256 expiration, uint256 amountIn)
        external
    {
        PoolKey memory key = keys[index];
        Currency sold = zeroForOne ? key.currency0 : key.currency1;
        IApprove(Currency.unwrap(sold)).approve(hook, type(uint256).max);
        ITwammOrders(hook).submitOrder(key, zeroForOne, expiration, amountIn);
    }

    /// @notice Places a long-term order into the production TWAMM hook.
    /// @dev Same job as {submitTwammOrder}, against a different ABI: that hook takes a duration and
    ///      rounds it onto its own interval grid, rather than an absolute expiration.
    function submitProductionTwammOrder(
        uint256 index,
        address hook,
        bool zeroForOne,
        uint256 duration,
        uint256 amountIn
    ) external {
        PoolKey memory key = keys[index];
        Currency sold = zeroForOne ? key.currency0 : key.currency1;
        IApprove(Currency.unwrap(sold)).approve(hook, type(uint256).max);
        IProductionTwamm(hook).submitOrder(ProdSubmitOrderParams(key, zeroForOne, duration, amountIn));
    }

    /// @notice Withdraws what a long-term order has earned so far.
    function claimTwammProceeds(uint256 index, address hook, bool zeroForOne, uint256 expiration) external {
        ITwammOrders(hook).claimProceeds(keys[index], zeroForOne, expiration);
    }

    /// @notice Brings a TWAMM pool up to date without swapping through it.
    function executeTwammOrders(uint256 index, address hook) external {
        ITwammOrders(hook).executeVirtualOrders(keys[index]);
    }

    /// @notice Removes some liquidity, calling the hook's `beforeRemoveLiquidity`.
    function removeLiquidity(uint256 index, uint128 liquidity) external {
        bytes memory actions =
            abi.encodePacked(uint8(Actions.DECREASE_LIQUIDITY), uint8(Actions.TAKE_PAIR));
        bytes[] memory params = new bytes[](2);
        params[0] = abi.encode(tokenIds[index], liquidity, 0, 0, bytes(""));
        params[1] = abi.encode(keys[index].currency0, keys[index].currency1, address(this));
        positionManager.modifyLiquidities(abi.encode(actions, params), block.timestamp + 3600);
    }
}
