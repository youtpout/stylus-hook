// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/// @title The Solidity control for `stylus/native-crypto`
/// @notice The two primitives an ML-DSA signature needs, written the way Solidity would write them.
/// @dev Neither can use a built-in: `keccak256` has the `0x01` pad compiled in and SHAKE pads with
///      `0x1f`, so the permutation is run by hand on both sides. `mulmod` is one opcode, which is
///      why the Rust twin also carries a `%` version to be compared against it.
contract CryptoBench {
    uint32 internal constant Q = 8380417;
    uint256 internal constant SHAKE256_RATE = 136;

    function baseline(uint256) external pure returns (uint256) {
        return 0;
    }

    // --- Keccak ---------------------------------------------------------------------------------

    /// @notice `n` chained `keccak256` calls -- the opcode, as the control.
    function keccakOpcodeLoop(uint256 n, bytes32 seed) external pure returns (bytes32 acc) {
        acc = seed;
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                acc = keccak256(abi.encodePacked(acc));
            }
        }
    }

    /// @notice `n` Keccak-f[1600] permutations, through `mload`/`mstore`.
    function keccakFLoop(uint256 n) external pure returns (uint256) {
        uint64[25] memory a;
        Tables memory t = _tables();
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                _keccakFAsm(a, t);
            }
        }
        return a[0];
    }

    /// @notice The same, with Solidity's bounds-checked array access. Measured to show the cost of
    /// the checks, not as the control.
    function keccakFLoopChecked(uint256 n) external pure returns (uint256) {
        uint64[25] memory a;
        Tables memory t = _tables();
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                _keccakF(a, t);
            }
        }
        return a[0];
    }

    /// @notice SHAKE256 over `inputLen` zero bytes, squeezing `outLen`, `n` times.
    function shake256Loop(uint256 n, uint256 inputLen, uint256 outLen) external pure returns (bytes32) {
        bytes memory input = new bytes(inputLen);
        bytes memory out;
        Tables memory t = _tables();
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                out = _shake256(input, outLen, t);
                if (inputLen > 0) input[0] = bytes1(uint8(input[0]) ^ 1);
            }
        }
        return _first32(out);
    }

    /// @notice SHAKE256 of `input`, 32 bytes out. For checking against the Rust twin.
    function shake256(bytes calldata input) external pure returns (bytes32) {
        return _first32(_shake256(input, 32, _tables()));
    }

    function _first32(bytes memory b) private pure returns (bytes32 r) {
        if (b.length == 0) return bytes32(0);
        assembly ("memory-safe") {
            r := mload(add(b, 32))
        }
    }

    function _shake256(bytes memory input, uint256 outLen, Tables memory t)
        private
        pure
        returns (bytes memory out)
    {
        uint64[25] memory a;
        unchecked {
            uint256 full = input.length / SHAKE256_RATE;
            for (uint256 blk = 0; blk < full; blk++) {
                _absorb(a, input, blk * SHAKE256_RATE, SHAKE256_RATE);
                _keccakFAsm(a, t);
            }

            // The final block carries the `0x1f` pad and the `0x80` terminator.
            uint256 rest = input.length - full * SHAKE256_RATE;
            bytes memory tail = new bytes(SHAKE256_RATE);
            for (uint256 i = 0; i < rest; i++) {
                tail[i] = input[full * SHAKE256_RATE + i];
            }
            tail[rest] = 0x1f;
            tail[SHAKE256_RATE - 1] = bytes1(uint8(tail[SHAKE256_RATE - 1]) | 0x80);
            _absorb(a, tail, 0, SHAKE256_RATE);
            _keccakFAsm(a, t);

            out = new bytes(outLen);
            uint256 written = 0;
            while (written < outLen) {
                uint256 take = outLen - written < SHAKE256_RATE ? outLen - written : SHAKE256_RATE;
                for (uint256 i = 0; i < take; i++) {
                    out[written + i] = bytes1(uint8(a[i / 8] >> (8 * (i % 8))));
                }
                written += take;
                if (written < outLen) _keccakFAsm(a, t);
            }
        }
    }

    function _absorb(uint64[25] memory a, bytes memory block_, uint256 offset, uint256 len)
        private
        pure
    {
        unchecked {
            for (uint256 i = 0; i < len / 8; i++) {
                uint64 lane = 0;
                for (uint256 b = 0; b < 8; b++) {
                    lane |= uint64(uint8(block_[offset + 8 * i + b])) << (8 * b);
                }
                a[i] ^= lane;
            }
        }
    }

    function _rotl(uint64 x, uint256 n) private pure returns (uint64) {
        unchecked {
            return n == 0 ? x : (x << n) | (x >> (64 - n));
        }
    }

    /// @dev Every table the permutation needs, built once per external call.
    ///
    /// Building them per round costs more than the permutation itself: the first version of this
    /// contract spent 1.5M gas on one permutation for exactly that reason.
    struct Tables {
        uint64[24] rc;
        uint256[25] rho;
        uint256[25] pi;
        uint256[5] next1;
        uint256[5] next2;
        uint256[5] prev;
    }

    function _tables() private pure returns (Tables memory t) {
        t.rc = [
            0x0000000000000001, 0x0000000000008082, 0x800000000000808a, 0x8000000080008000,
            0x000000000000808b, 0x0000000080000001, 0x8000000080008081, 0x8000000000008009,
            0x000000000000008a, 0x0000000000000088, 0x0000000080008009, 0x000000008000000a,
            0x000000008000808b, 0x800000000000008b, 0x8000000000008089, 0x8000000000008003,
            0x8000000000008002, 0x8000000000000080, 0x000000000000800a, 0x800000008000000a,
            0x8000000080008081, 0x8000000000008080, 0x0000000080000001, 0x8000000080008008
        ];
        t.rho = [
            uint256(0), 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43,
            25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14
        ];
        unchecked {
            for (uint256 x = 0; x < 5; x++) {
                t.next1[x] = (x + 1) % 5;
                t.next2[x] = (x + 2) % 5;
                t.prev[x] = (x + 4) % 5;
                for (uint256 y = 0; y < 5; y++) {
                    t.pi[x + 5 * y] = y + 5 * ((2 * x + 3 * y) % 5);
                }
            }
        }
    }

    function _keccakF(uint64[25] memory a, Tables memory t) private pure {
        unchecked {
            uint64[5] memory c;
            uint64[25] memory b;
            for (uint256 round = 0; round < 24; round++) {
                for (uint256 x = 0; x < 5; x++) {
                    c[x] = a[x] ^ a[x + 5] ^ a[x + 10] ^ a[x + 15] ^ a[x + 20];
                }
                for (uint256 x = 0; x < 5; x++) {
                    uint64 d = c[t.prev[x]] ^ _rotl(c[t.next1[x]], 1);
                    for (uint256 y = 0; y < 5; y++) {
                        a[x + 5 * y] ^= d;
                    }
                }

                for (uint256 i = 0; i < 25; i++) {
                    b[t.pi[i]] = _rotl(a[i], t.rho[i]);
                }

                for (uint256 y = 0; y < 5; y++) {
                    uint256 row = 5 * y;
                    for (uint256 x = 0; x < 5; x++) {
                        a[row + x] = b[row + x] ^ (~b[row + t.next1[x]] & b[row + t.next2[x]]);
                    }
                }
                a[0] ^= t.rc[round];
            }
        }
    }

    /// @dev The same permutation through raw `mload`/`mstore`, which is how Keccak is written in
    ///      Solidity in practice. The readable version above spends most of its gas on bounds
    ///      checks — 662k against this one — so skipping them is the difference between a strawman
    ///      and a fair control. `test_keccakF_theTwoImplementationsAgree` pins the two together.
    function _keccakFAsm(uint64[25] memory a, Tables memory t) private pure {
        uint64[25] memory b;
        uint64[24] memory rc = t.rc;
        assembly ("memory-safe") {
            let M := 0xffffffffffffffff
            for { let round := 0 } lt(round, 24) { round := add(round, 1) } {
                // Theta: five column parities, then fold each back into its column.
                let c0 := xor(xor(xor(xor(mload(add(a, 0)), mload(add(a, 160))), mload(add(a, 320))), mload(add(a, 480))), mload(add(a, 640)))
                let c1 := xor(xor(xor(xor(mload(add(a, 32)), mload(add(a, 192))), mload(add(a, 352))), mload(add(a, 512))), mload(add(a, 672)))
                let c2 := xor(xor(xor(xor(mload(add(a, 64)), mload(add(a, 224))), mload(add(a, 384))), mload(add(a, 544))), mload(add(a, 704)))
                let c3 := xor(xor(xor(xor(mload(add(a, 96)), mload(add(a, 256))), mload(add(a, 416))), mload(add(a, 576))), mload(add(a, 736)))
                let c4 := xor(xor(xor(xor(mload(add(a, 128)), mload(add(a, 288))), mload(add(a, 448))), mload(add(a, 608))), mload(add(a, 768)))

                {
                    let d := xor(c4, and(or(shl(1, c1), shr(63, c1)), M))
                    mstore(add(a, 0), xor(mload(add(a, 0)), d))
                    mstore(add(a, 160), xor(mload(add(a, 160)), d))
                    mstore(add(a, 320), xor(mload(add(a, 320)), d))
                    mstore(add(a, 480), xor(mload(add(a, 480)), d))
                    mstore(add(a, 640), xor(mload(add(a, 640)), d))
                }
                {
                    let d := xor(c0, and(or(shl(1, c2), shr(63, c2)), M))
                    mstore(add(a, 32), xor(mload(add(a, 32)), d))
                    mstore(add(a, 192), xor(mload(add(a, 192)), d))
                    mstore(add(a, 352), xor(mload(add(a, 352)), d))
                    mstore(add(a, 512), xor(mload(add(a, 512)), d))
                    mstore(add(a, 672), xor(mload(add(a, 672)), d))
                }
                {
                    let d := xor(c1, and(or(shl(1, c3), shr(63, c3)), M))
                    mstore(add(a, 64), xor(mload(add(a, 64)), d))
                    mstore(add(a, 224), xor(mload(add(a, 224)), d))
                    mstore(add(a, 384), xor(mload(add(a, 384)), d))
                    mstore(add(a, 544), xor(mload(add(a, 544)), d))
                    mstore(add(a, 704), xor(mload(add(a, 704)), d))
                }
                {
                    let d := xor(c2, and(or(shl(1, c4), shr(63, c4)), M))
                    mstore(add(a, 96), xor(mload(add(a, 96)), d))
                    mstore(add(a, 256), xor(mload(add(a, 256)), d))
                    mstore(add(a, 416), xor(mload(add(a, 416)), d))
                    mstore(add(a, 576), xor(mload(add(a, 576)), d))
                    mstore(add(a, 736), xor(mload(add(a, 736)), d))
                }
                {
                    let d := xor(c3, and(or(shl(1, c0), shr(63, c0)), M))
                    mstore(add(a, 128), xor(mload(add(a, 128)), d))
                    mstore(add(a, 288), xor(mload(add(a, 288)), d))
                    mstore(add(a, 448), xor(mload(add(a, 448)), d))
                    mstore(add(a, 608), xor(mload(add(a, 608)), d))
                    mstore(add(a, 768), xor(mload(add(a, 768)), d))
                }

                // Rho and pi: rotate each lane by its own offset, into its own slot.
                mstore(add(b, 0), mload(add(a, 0)))
                mstore(add(b, 320), and(or(shl(1, mload(add(a, 32))), shr(63, mload(add(a, 32)))), M))
                mstore(add(b, 640), and(or(shl(62, mload(add(a, 64))), shr(2, mload(add(a, 64)))), M))
                mstore(add(b, 160), and(or(shl(28, mload(add(a, 96))), shr(36, mload(add(a, 96)))), M))
                mstore(add(b, 480), and(or(shl(27, mload(add(a, 128))), shr(37, mload(add(a, 128)))), M))
                mstore(add(b, 512), and(or(shl(36, mload(add(a, 160))), shr(28, mload(add(a, 160)))), M))
                mstore(add(b, 32), and(or(shl(44, mload(add(a, 192))), shr(20, mload(add(a, 192)))), M))
                mstore(add(b, 352), and(or(shl(6, mload(add(a, 224))), shr(58, mload(add(a, 224)))), M))
                mstore(add(b, 672), and(or(shl(55, mload(add(a, 256))), shr(9, mload(add(a, 256)))), M))
                mstore(add(b, 192), and(or(shl(20, mload(add(a, 288))), shr(44, mload(add(a, 288)))), M))
                mstore(add(b, 224), and(or(shl(3, mload(add(a, 320))), shr(61, mload(add(a, 320)))), M))
                mstore(add(b, 544), and(or(shl(10, mload(add(a, 352))), shr(54, mload(add(a, 352)))), M))
                mstore(add(b, 64), and(or(shl(43, mload(add(a, 384))), shr(21, mload(add(a, 384)))), M))
                mstore(add(b, 384), and(or(shl(25, mload(add(a, 416))), shr(39, mload(add(a, 416)))), M))
                mstore(add(b, 704), and(or(shl(39, mload(add(a, 448))), shr(25, mload(add(a, 448)))), M))
                mstore(add(b, 736), and(or(shl(41, mload(add(a, 480))), shr(23, mload(add(a, 480)))), M))
                mstore(add(b, 256), and(or(shl(45, mload(add(a, 512))), shr(19, mload(add(a, 512)))), M))
                mstore(add(b, 576), and(or(shl(15, mload(add(a, 544))), shr(49, mload(add(a, 544)))), M))
                mstore(add(b, 96), and(or(shl(21, mload(add(a, 576))), shr(43, mload(add(a, 576)))), M))
                mstore(add(b, 416), and(or(shl(8, mload(add(a, 608))), shr(56, mload(add(a, 608)))), M))
                mstore(add(b, 448), and(or(shl(18, mload(add(a, 640))), shr(46, mload(add(a, 640)))), M))
                mstore(add(b, 768), and(or(shl(2, mload(add(a, 672))), shr(62, mload(add(a, 672)))), M))
                mstore(add(b, 288), and(or(shl(61, mload(add(a, 704))), shr(3, mload(add(a, 704)))), M))
                mstore(add(b, 608), and(or(shl(56, mload(add(a, 736))), shr(8, mload(add(a, 736)))), M))
                mstore(add(b, 128), and(or(shl(14, mload(add(a, 768))), shr(50, mload(add(a, 768)))), M))

                // Chi back into the state, with iota folded into lane zero.
                mstore(add(a, 0), xor(xor(mload(add(b, 0)), and(not(mload(add(b, 32))), mload(add(b, 64)))), mload(add(rc, mul(32, round)))))
                mstore(add(a, 32), xor(mload(add(b, 32)), and(not(mload(add(b, 64))), mload(add(b, 96)))))
                mstore(add(a, 64), xor(mload(add(b, 64)), and(not(mload(add(b, 96))), mload(add(b, 128)))))
                mstore(add(a, 96), xor(mload(add(b, 96)), and(not(mload(add(b, 128))), mload(add(b, 0)))))
                mstore(add(a, 128), xor(mload(add(b, 128)), and(not(mload(add(b, 0))), mload(add(b, 32)))))
                mstore(add(a, 160), xor(mload(add(b, 160)), and(not(mload(add(b, 192))), mload(add(b, 224)))))
                mstore(add(a, 192), xor(mload(add(b, 192)), and(not(mload(add(b, 224))), mload(add(b, 256)))))
                mstore(add(a, 224), xor(mload(add(b, 224)), and(not(mload(add(b, 256))), mload(add(b, 288)))))
                mstore(add(a, 256), xor(mload(add(b, 256)), and(not(mload(add(b, 288))), mload(add(b, 160)))))
                mstore(add(a, 288), xor(mload(add(b, 288)), and(not(mload(add(b, 160))), mload(add(b, 192)))))
                mstore(add(a, 320), xor(mload(add(b, 320)), and(not(mload(add(b, 352))), mload(add(b, 384)))))
                mstore(add(a, 352), xor(mload(add(b, 352)), and(not(mload(add(b, 384))), mload(add(b, 416)))))
                mstore(add(a, 384), xor(mload(add(b, 384)), and(not(mload(add(b, 416))), mload(add(b, 448)))))
                mstore(add(a, 416), xor(mload(add(b, 416)), and(not(mload(add(b, 448))), mload(add(b, 320)))))
                mstore(add(a, 448), xor(mload(add(b, 448)), and(not(mload(add(b, 320))), mload(add(b, 352)))))
                mstore(add(a, 480), xor(mload(add(b, 480)), and(not(mload(add(b, 512))), mload(add(b, 544)))))
                mstore(add(a, 512), xor(mload(add(b, 512)), and(not(mload(add(b, 544))), mload(add(b, 576)))))
                mstore(add(a, 544), xor(mload(add(b, 544)), and(not(mload(add(b, 576))), mload(add(b, 608)))))
                mstore(add(a, 576), xor(mload(add(b, 576)), and(not(mload(add(b, 608))), mload(add(b, 480)))))
                mstore(add(a, 608), xor(mload(add(b, 608)), and(not(mload(add(b, 480))), mload(add(b, 512)))))
                mstore(add(a, 640), xor(mload(add(b, 640)), and(not(mload(add(b, 672))), mload(add(b, 704)))))
                mstore(add(a, 672), xor(mload(add(b, 672)), and(not(mload(add(b, 704))), mload(add(b, 736)))))
                mstore(add(a, 704), xor(mload(add(b, 704)), and(not(mload(add(b, 736))), mload(add(b, 768)))))
                mstore(add(a, 736), xor(mload(add(b, 736)), and(not(mload(add(b, 768))), mload(add(b, 640)))))
                mstore(add(a, 768), xor(mload(add(b, 768)), and(not(mload(add(b, 640))), mload(add(b, 672)))))
            }
        }
    }

    // --- the transform --------------------------------------------------------------------------

    /// @notice `n` forward NTTs over ML-DSA's ring, with `mulmod` doing the reduction.
    function nttLoop(uint256 n) external pure returns (uint256) {
        uint256[256] memory poly;
        uint32[256] memory zetas = _zetas();
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                for (uint256 j = 0; j < 256; j++) {
                    poly[j] = (poly[j] + j) % Q;
                }
                _ntt(poly, zetas);
            }
        }
        return poly[0];
    }

    /// @notice One forward NTT of `0..256`, first four coefficients. For checking against the twin.
    function nttVector() external pure returns (uint256[] memory first) {
        uint256[256] memory poly;
        unchecked {
            for (uint256 i = 0; i < 256; i++) {
                poly[i] = i;
            }
        }
        _ntt(poly, _zetas());
        first = new uint256[](4);
        for (uint256 i = 0; i < 4; i++) {
            first[i] = poly[i];
        }
    }

    function _zetas() private pure returns (uint32[256] memory) {
        return [
            uint32(1), 4808194, 3765607, 3761513, 5178923, 5496691, 5234739, 5178987,
            7778734, 3542485, 2682288, 2129892, 3764867, 7375178, 557458, 7159240,
            5010068, 4317364, 2663378, 6705802, 4855975, 7946292, 676590, 7044481,
            5152541, 1714295, 2453983, 1460718, 7737789, 4795319, 2815639, 2283733,
            3602218, 3182878, 2740543, 4793971, 5269599, 2101410, 3704823, 1159875,
            394148, 928749, 1095468, 4874037, 2071829, 4361428, 3241972, 2156050,
            3415069, 1759347, 7562881, 4805951, 3756790, 6444618, 6663429, 4430364,
            5483103, 3192354, 556856, 3870317, 2917338, 1853806, 3345963, 1858416,
            3073009, 1277625, 5744944, 3852015, 4183372, 5157610, 5258977, 8106357,
            2508980, 2028118, 1937570, 4564692, 2811291, 5396636, 7270901, 4158088,
            1528066, 482649, 1148858, 5418153, 7814814, 169688, 2462444, 5046034,
            4213992, 4892034, 1987814, 5183169, 1736313, 235407, 5130263, 3258457,
            5801164, 1787943, 5989328, 6125690, 3482206, 4197502, 7080401, 6018354,
            7062739, 2461387, 3035980, 621164, 3901472, 7153756, 2925816, 3374250,
            1356448, 5604662, 2683270, 5601629, 4912752, 2312838, 7727142, 7921254,
            348812, 8052569, 1011223, 6026202, 4561790, 6458164, 6143691, 1744507,
            1753, 6444997, 5720892, 6924527, 2660408, 6600190, 8321269, 2772600,
            1182243, 87208, 636927, 4415111, 4423672, 6084020, 5095502, 4663471,
            8352605, 822541, 1009365, 5926272, 6400920, 1596822, 4423473, 4620952,
            6695264, 4969849, 2678278, 4611469, 4829411, 635956, 8129971, 5925040,
            4234153, 6607829, 2192938, 6653329, 2387513, 4768667, 8111961, 5199961,
            3747250, 2296099, 1239911, 4541938, 3195676, 2642980, 1254190, 8368000,
            2998219, 141835, 8291116, 2513018, 7025525, 613238, 7070156, 6161950,
            7921677, 6458423, 4040196, 4908348, 2039144, 6500539, 7561656, 6201452,
            6757063, 2105286, 6006015, 6346610, 586241, 7200804, 527981, 5637006,
            6903432, 1994046, 2491325, 6987258, 507927, 7192532, 7655613, 6545891,
            5346675, 8041997, 2647994, 3009748, 5767564, 4148469, 749577, 4357667,
            3980599, 2569011, 6764887, 1723229, 1665318, 2028038, 1163598, 5011144,
            3994671, 8368538, 7009900, 3020393, 3363542, 214880, 545376, 7609976,
            3105558, 7277073, 508145, 7826699, 860144, 3430436, 140244, 6866265,
            6195333, 3123762, 2358373, 6187330, 5365997, 6663603, 2926054, 7987710,
            8077412, 3531229, 4405932, 4606686, 1900052, 7598542, 1054478, 7648983
        ];
    }

    /// @dev The butterfly through raw `mload`/`mstore`, for the same reason the permutation is:
    /// bounds-checked array access dominated the transform at 457k against this one.
    /// `test_ntt_theTwoImplementationsAgree` holds it to the checked version below.
    function _ntt(uint256[256] memory a, uint32[256] memory zetas) private pure {
        assembly ("memory-safe") {
            let q := Q
            let k := 0
            for { let len := 128 } gt(len, 0) { len := shr(1, len) } {
                let step := mul(2, len)
                for { let start := 0 } lt(start, 256) { start := add(start, step) } {
                    k := add(k, 1)
                    let zeta := mload(add(zetas, mul(32, k)))
                    let lo := add(a, mul(32, start))
                    let hi := add(lo, mul(32, len))
                    for { let j := 0 } lt(j, len) { j := add(j, 1) } {
                        let pl := add(lo, mul(32, j))
                        let ph := add(hi, mul(32, j))
                        let t := mulmod(zeta, mload(ph), q)
                        let v := mload(pl)
                        mstore(ph, mod(add(sub(v, t), q), q))
                        mstore(pl, mod(add(v, t), q))
                    }
                }
            }
        }
    }

    /// @notice The same transform with Solidity's bounds-checked array access, to show their cost.
    function nttLoopChecked(uint256 n) external pure returns (uint256) {
        uint256[256] memory poly;
        uint32[256] memory zetas = _zetas();
        unchecked {
            for (uint256 i = 0; i < n; i++) {
                for (uint256 j = 0; j < 256; j++) {
                    poly[j] = (poly[j] + j) % Q;
                }
                _nttChecked(poly, zetas);
            }
        }
        return poly[0];
    }

    function _nttChecked(uint256[256] memory a, uint32[256] memory zetas) private pure {
        unchecked {
            uint256 k = 0;
            uint256 len = 128;
            while (len >= 1) {
                for (uint256 start = 0; start < 256; start += 2 * len) {
                    uint256 zeta = zetas[++k];
                    for (uint256 j = start; j < start + len; j++) {
                        uint256 t = mulmod(zeta, a[j + len], Q);
                        a[j + len] = (a[j] + Q - t) % Q;
                        a[j] = (a[j] + t) % Q;
                    }
                }
                len >>= 1;
            }
        }
    }
}
