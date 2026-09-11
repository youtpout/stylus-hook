// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {CryptoBench} from "../src/CryptoBench.sol";

/// @notice Pins both primitives to the standard, so the benchmark cannot compare two wrong things.
///
/// @dev `stylus/native-crypto` asserts these same vectors. SHAKE256's come from NIST; the NTT's come
///      from the transform itself, which is checked against ML-DSA's published `zetas[1]` and the
///      order of its root of unity on the Rust side.
contract CryptoBenchTest is Test {
    CryptoBench internal bench;

    function setUp() public {
        bench = new CryptoBench();
    }

    function test_keccakF_matchesTheReferenceVector() public view {
        // XKCP's vector for the permutation on an all-zero state.
        assertEq(bench.keccakFLoop(1), 0xf1258f7940e1dde7);
    }

    /// The assembly permutation is what the benchmark measures, so it has to agree with the
    /// bounds-checked one it replaced, not only with the vector at round one.
    function test_keccakF_theTwoImplementationsAgree() public view {
        for (uint256 n = 1; n <= 4; n++) {
            assertEq(bench.keccakFLoop(n), bench.keccakFLoopChecked(n), "permutation count");
        }
    }

    function test_shake256_matchesNist() public view {
        assertEq(
            bench.shake256(""),
            hex"46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f"
        );
        assertEq(
            bench.shake256("abc"),
            hex"483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739"
        );
    }

    /// An input longer than the 136-byte rate, so the multi-block absorb is covered.
    function test_shake256_absorbsMoreThanOneBlock() public view {
        bytes memory input = new bytes(200);
        for (uint256 i = 0; i < 200; i++) {
            input[i] = 0xa3;
        }
        assertEq(
            bench.shake256(input),
            hex"cd8a920ed141aa0407a22d59288652e9d9f1a7ee0c1e7c1ca699424da84a904d"
        );
    }

    /// The assembly butterfly is what the benchmark measures, so it has to agree with the
    /// bounds-checked one across more than the first coefficient.
    function test_ntt_theTwoImplementationsAgree() public view {
        for (uint256 n = 1; n <= 3; n++) {
            assertEq(bench.nttLoop(n), bench.nttLoopChecked(n), "transform count");
        }
    }

    function test_ntt_matchesTheRustPort() public view {
        uint256[] memory got = bench.nttVector();
        assertEq(got[0], 8023823);
        assertEq(got[1], 4949942);
        assertEq(got[2], 5503697);
        assertEq(got[3], 7227518);
    }
}
