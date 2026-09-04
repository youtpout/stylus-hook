// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {PoolKey} from "@uniswap/v4-core/src/types/PoolKey.sol";
import {Currency} from "@uniswap/v4-core/src/types/Currency.sol";
import {IUniswapV4Router04} from "hookmate/interfaces/router/IUniswapV4Router04.sol";

interface IApprove {
    function approve(address spender, uint256 amount) external returns (bool);
}

interface IPermit2Approve {
    function approve(address token, address spender, uint160 amount, uint48 expiration) external;
}

/// @notice Holds one pool per hook implementation and swaps through them one transaction at a time,
///         so the gas each hook costs can be read straight off the receipts.
contract SwapBench {
    IUniswapV4Router04 public immutable router;

    PoolKey[] private pools;
    bytes[] private hookDatas;

    constructor(IUniswapV4Router04 _router, address permit2, address poolManager, address[] memory tokens) {
        router = _router;
        for (uint256 i = 0; i < tokens.length; i++) {
            IApprove(tokens[i]).approve(address(_router), type(uint256).max);
            IApprove(tokens[i]).approve(permit2, type(uint256).max);
            IPermit2Approve(permit2).approve(tokens[i], poolManager, type(uint160).max, type(uint48).max);
            IPermit2Approve(permit2).approve(tokens[i], address(_router), type(uint160).max, type(uint48).max);
        }
    }

    function addPool(PoolKey calldata key, bytes calldata hookData) external {
        pools.push(key);
        hookDatas.push(hookData);
    }

    function poolCount() external view returns (uint256) {
        return pools.length;
    }

    /// @notice One swap, one transaction. Compare `gasUsed` across indices to price each hook.
    function swap(uint256 index, uint256 amountIn, bool zeroForOne) external {
        router.swapExactTokensForTokens({
            amountIn: amountIn,
            amountOutMin: 0,
            zeroForOne: zeroForOne,
            poolKey: pools[index],
            hookData: hookDatas[index],
            receiver: address(this),
            deadline: block.timestamp + 3600
        });
    }
}
