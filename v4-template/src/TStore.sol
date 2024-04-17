// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract TStore {
    uint256 public number;

    modifier nonreentrant() {
        // assembly {
        //     if sload(0) {
        //         revert(0, 0)
        //     }
        //     tstore(0, 1)
        // }
        // _;
        // // Unlocks the guard, making the pattern composable.
        // // After the function exits, it can be called again, even in the same transaction.
        // assembly {
        //     tstore(0, 0)
        // }
        _;
    }

    function increment() public nonreentrant {
        number++;
    }
}
