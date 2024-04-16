// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import {TStore} from "../src/TStore.sol";

contract TstoreTest is Test {

    TStore tStore;

    function setUp() public {
        tStore = new TStore();
    }

    function testIncrement() public {
        assertEq(tStore.number(), 0);
        tStore.increment();
        assertEq(tStore.number(), 1);
    }
}
