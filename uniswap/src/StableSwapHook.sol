// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

import {Hooks} from "@uniswap/v4-core/src/libraries/Hooks.sol";
import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";
import {SwapParams} from "@uniswap/v4-core/src/types/PoolOperation.sol";
import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "@uniswap/v4-core/src/types/BeforeSwapDelta.sol";

/// @title StableSwapHook
/// @notice A hook that prices swaps on a StableSwap curve. The Solidity twin of
///         `stylus/native-stableswap`.
///
/// @dev Every shipping hook profiled in BENCHMARK.md turned out to be dominated by storage, and
///      porting one to Stylus was worth a few percent at best. This one is built the other way
///      round: pricing a swap means solving the StableSwap invariant `D` and then the output
///      reserve `y`, both by Newton's method — about twelve iterations of full-range 256-bit
///      arithmetic per swap, which is the regime where Rust runs several times cheaper.
///
///      The invariant is implemented from Curve's published StableSwap formula. No implementation
///      was copied.
contract StableSwapHook is BaseHook {
    /// @notice Amplification coefficient times n^n, for n = 2. Higher means a flatter curve.
    uint256 public constant DEFAULT_ANN = 100;

    uint256 public reserve0 = 1000e18;
    uint256 public reserve1 = 600e18;
    uint256 public ann = DEFAULT_ANN;
    uint256 public lastQuote;

    constructor(IPoolManager _poolManager) BaseHook(_poolManager) {}

    function getHookPermissions() public pure override returns (Hooks.Permissions memory) {
        return Hooks.Permissions({
            beforeInitialize: false,
            afterInitialize: false,
            beforeAddLiquidity: false,
            afterAddLiquidity: false,
            beforeRemoveLiquidity: false,
            afterRemoveLiquidity: false,
            beforeSwap: true,
            afterSwap: false,
            beforeDonate: false,
            afterDonate: false,
            beforeSwapReturnDelta: false,
            afterSwapReturnDelta: false,
            afterAddLiquidityReturnDelta: false,
            afterRemoveLiquidityReturnDelta: false
        });
    }

    function reserves() external view returns (uint256, uint256) {
        return (reserve0, reserve1);
    }

    /// @notice The StableSwap invariant for two assets, solved by Newton's method.
    /// @dev With imbalanced reserves this takes about four iterations.
    function getD(uint256 x0, uint256 x1, uint256 amp) public pure returns (uint256 d) {
        uint256 s = x0 + x1;
        if (s == 0) return 0;
        d = s;
        for (uint256 i = 0; i < 255; i++) {
            uint256 dp = ((d * d) / (x0 * 2)) * d / (x1 * 2);
            uint256 prev = d;
            d = (amp * s + 2 * dp) * d / ((amp - 1) * d + 3 * dp);
            if (d > prev ? d - prev <= 1 : prev - d <= 1) break;
        }
    }

    /// @notice The reserve of the output token once `amountIn` has been added to `x0`.
    /// @dev About eight more Newton iterations on top of {getD}.
    function getY(uint256 amountIn, uint256 x0, uint256 x1, uint256 amp) public pure returns (uint256 y) {
        uint256 d = getD(x0, x1, amp);
        uint256 x = x0 + amountIn;
        uint256 c = ((d * d) / (x * 2)) * d / (amp * 2);
        uint256 b = x + d / amp;
        y = d;
        for (uint256 i = 0; i < 255; i++) {
            uint256 prev = y;
            y = (y * y + c) / (2 * y + b - d);
            if (y > prev ? y - prev <= 1 : prev - y <= 1) break;
        }
    }

    /// @notice `n` chained `a * b / c` on values that never overflow 256 bits.
    /// @dev The isolation probe for the StableSwap result. Here a multiply is one `MUL` and a
    ///      divide is one `DIV`, five gas each, with no `FullMath` wrapper and no 512-bit
    ///      intermediate. Nothing else is measured: no storage, no hook plumbing.
    function plainMulDiv(uint256 n) public pure returns (uint256) {
        uint256 z = 1 << 100;
        uint256 y = (1 << 100) - 1;
        uint256 a = (1 << 100) + 1;
        for (uint256 i = 0; i < n; i++) {
            a = (a * y) / z + 1;
        }
        return a;
    }

    /// @notice Prices `amountIn` against the current reserves without touching them.
    function quote(uint256 amountIn) public view returns (uint256) {
        uint256 y = getY(amountIn, reserve0, reserve1, ann);
        return reserve1 > y ? reserve1 - y : 0;
    }

    /// @dev The delta returned is zero: this hook measures what the curve costs, it does not take
    ///      custody of the swap. A production version would return the priced delta here and
    ///      balance its books against the pool manager.
    function _beforeSwap(address, PoolKey calldata, SwapParams calldata params, bytes calldata)
        internal
        override
        returns (bytes4, BeforeSwapDelta, uint24)
    {
        uint256 amountIn = params.amountSpecified < 0
            ? uint256(-params.amountSpecified)
            : uint256(params.amountSpecified);

        uint256 x0 = reserve0;
        uint256 x1 = reserve1;
        uint256 y = getY(amountIn, x0, x1, ann);

        reserve0 = x0 + amountIn;
        reserve1 = y;
        lastQuote = x1 > y ? x1 - y : 0;

        return (BaseHook.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }
}
