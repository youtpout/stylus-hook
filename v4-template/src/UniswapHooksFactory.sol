// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IPoolManager} from "v4-core/src/interfaces/IPoolManager.sol";
import {BaseHook} from "v4-periphery/BaseHook.sol";

contract UniswapHooksFactory {
    fallback() external payable {
        assembly {
            calldatacopy(0, 32, sub(calldatasize(), 32))
            let result := create2(
                callvalue(),
                0,
                sub(calldatasize(), 32),
                calldataload(0)
            )
            if iszero(result) {
                revert(0, 0)
            }
            mstore(0, result)
            return(12, 20)
        }
    }
}
