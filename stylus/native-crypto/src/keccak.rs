//! Keccak-f[1600] and SHAKE256, by hand.
//!
//! Not because a hash is missing — Stylus has `native_keccak256` and the EVM has the `KECCAK256`
//! opcode, and both are Keccak-256 with the `0x01` pad wired in. SHAKE256 pads with `0x1f` and
//! squeezes an arbitrary length, so neither built-in can produce it. Anything built on SHAKE — every
//! ML-DSA, ML-KEM and SLH-DSA operation — has to run the permutation itself.
//!
//! That is the whole point of the measurement: the permutation is 24 rounds of 64-bit rotations on
//! 25 lanes, and a 64-bit rotation is one WASM instruction and four EVM opcodes plus a mask.

/// Round constants, from the FIPS 202 LFSR.
static RC: [u64; 24] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808a,
    0x8000000080008000,
    0x000000000000808b,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008a,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000a,
    0x000000008000808b,
    0x800000000000008b,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800a,
    0x800000008000000a,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

/// Rho rotation offsets, indexed `x + 5y`.
static RHO: [u32; 25] = [
    0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14,
];

/// SHAKE256's rate in bytes: 1088 bits.
pub const SHAKE256_RATE: usize = 136;

/// The Keccak-f[1600] permutation, in place.
pub fn keccak_f(a: &mut [u64; 25]) {
    for &rc in RC.iter() {
        // Theta.
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = a[x] ^ a[x + 5] ^ a[x + 10] ^ a[x + 15] ^ a[x + 20];
        }
        for x in 0..5 {
            let d = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            for y in 0..5 {
                a[x + 5 * y] ^= d;
            }
        }

        // Rho and pi, which have to land in a scratch state because pi is a permutation.
        let mut b = [0u64; 25];
        for x in 0..5 {
            for y in 0..5 {
                b[y + 5 * ((2 * x + 3 * y) % 5)] = a[x + 5 * y].rotate_left(RHO[x + 5 * y]);
            }
        }

        // Chi and iota.
        for y in 0..5 {
            for x in 0..5 {
                a[x + 5 * y] = b[x + 5 * y] ^ (!b[(x + 1) % 5 + 5 * y] & b[(x + 2) % 5 + 5 * y]);
            }
        }
        a[0] ^= rc;
    }
}

/// SHAKE256, absorbing `input` and squeezing `out.len()` bytes.
pub fn shake256(input: &[u8], out: &mut [u8]) {
    let mut a = [0u64; 25];
    let mut block = [0u8; SHAKE256_RATE];

    // Absorb.
    let mut chunks = input.chunks_exact(SHAKE256_RATE);
    for chunk in chunks.by_ref() {
        absorb(&mut a, chunk);
        keccak_f(&mut a);
    }
    let rest = chunks.remainder();
    block[..rest.len()].copy_from_slice(rest);
    block[rest.len()] = 0x1f; // the pad that no built-in keccak will give you
    block[SHAKE256_RATE - 1] |= 0x80;
    absorb(&mut a, &block);
    keccak_f(&mut a);

    // Squeeze.
    let mut written = 0;
    while written < out.len() {
        let take = core::cmp::min(SHAKE256_RATE, out.len() - written);
        for (i, byte) in out[written..written + take].iter_mut().enumerate() {
            *byte = (a[i / 8] >> (8 * (i % 8))) as u8;
        }
        written += take;
        if written < out.len() {
            keccak_f(&mut a);
        }
    }
}

fn absorb(a: &mut [u64; 25], block: &[u8]) {
    for (i, lane) in block.chunks_exact(8).enumerate() {
        a[i] ^= u64::from_le_bytes(lane.try_into().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The XKCP vector for the permutation on an all-zero state.
    #[test]
    fn the_permutation_matches_the_reference_vector() {
        let mut a = [0u64; 25];
        keccak_f(&mut a);
        assert_eq!(a[0], 0xf1258f7940e1dde7);
        assert_eq!(a[1], 0x84d5ccf933c0478a);
    }

    /// NIST's SHAKE256 vectors, which also pin the rho offsets and round constants above.
    #[test]
    fn shake256_matches_nist() {
        let mut out = [0u8; 32];
        shake256(b"", &mut out);
        assert_eq!(
            hex(&out),
            "46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762f"
        );

        shake256(b"abc", &mut out);
        assert_eq!(
            hex(&out),
            "483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739"
        );
    }

    /// An input longer than the rate, so the multi-block absorb path is covered too.
    #[test]
    fn shake256_absorbs_more_than_one_block() {
        let input = [0xa3u8; 200];
        let mut out = [0u8; 32];
        shake256(&input, &mut out);
        assert_eq!(
            hex(&out),
            "cd8a920ed141aa0407a22d59288652e9d9f1a7ee0c1e7c1ca699424da84a904d"
        );

        // Squeezing past the rate has to permute again rather than repeat the first block.
        let mut long = [0u8; 200];
        shake256(&input, &mut long);
        assert_eq!(&long[..32], &out[..]);
        assert_eq!(
            hex(&long[136..168]),
            "512bb85a226c4243556e696f6bd072c5aa2d9b69730244b56853d16970ad817e"
        );
    }

    fn hex(bytes: &[u8]) -> alloc::string::String {
        use alloc::string::String;
        use core::fmt::Write;
        let mut s = String::new();
        for b in bytes {
            write!(s, "{b:02x}").unwrap();
        }
        s
    }
    extern crate alloc;
}
