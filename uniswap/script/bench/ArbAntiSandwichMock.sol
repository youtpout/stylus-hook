// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IPoolManager} from "@uniswap/v4-core/src/interfaces/IPoolManager.sol";

import {AntiSandwichMock} from "@openzeppelin/uniswap-hooks/src/mocks/general/AntiSandwichMock.sol";
import {BaseHook} from "@openzeppelin/uniswap-hooks/src/base/BaseHook.sol";

interface ArbSys {
    function arbBlockNumber() external view returns (uint256);
}

/// @notice `AntiSandwichMock`, made to work on Arbitrum.
///
/// `AntiSandwichHook` keys its top-of-block checkpoint on `block.number`. On Arbitrum that is not
/// the block number: the EVM's `NUMBER` opcode returns an *estimate of the L1 block number*, which
/// advances roughly every twelve seconds and is identical across the many L2 blocks inside it.
///
/// Two consequences. On a chain where it never advances — an Arbitrum dev node reports 0 — the
/// checkpoint is never taken, `Pool.swap` runs against an empty state and the hook reverts with
/// `InvalidPrice()` on the first `zeroForOne == false` swap. On Arbitrum One it does advance, but
/// the beginning-of-block price is then frozen for an entire L1 block rather than one L2 block,
/// which is a far wider window than the hook intends.
///
/// `_getBlockNumber` is `virtual` for exactly this reason, so the fix is a one-line override onto
/// `ArbSys.arbBlockNumber()`, the real L2 block number.
contract ArbAntiSandwichMock is AntiSandwichMock {
    ArbSys private constant ARB_SYS = ArbSys(0x0000000000000000000000000000000000000064);

    constructor(IPoolManager _poolManager) AntiSandwichMock(_poolManager) {}

    function _getBlockNumber() internal view override returns (uint48) {
        return uint48(ARB_SYS.arbBlockNumber());
    }
}
