// SPDX-License-Identifier: MIT
//! The two primitives an ML-DSA signature check spends its gas on, priced against Solidity.
//!
//! SHAKE256 comes from no built-in: `native_keccak256` and the EVM opcode are both Keccak-256 with
//! the `0x01` pad compiled in, and SHAKE pads with `0x1f`, so the permutation has to be written out.
//! The NTT's prime is 23 bits wide, so every butterfly on the EVM pays a full 256-bit `MULMOD`.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

pub mod keccak;
pub mod ntt;

use alloc::{vec, vec::Vec};

use alloy_primitives::{FixedBytes, U256};
use stylus_sdk::{abi::Bytes, crypto, prelude::*, storage::StorageU8};

#[storage]
#[entrypoint]
pub struct Crypto {
    _pad: StorageU8,
}

#[public]
impl Crypto {
    /// A no-op, to subtract the cost of being called at all.
    pub fn baseline(&self, _n: U256) -> U256 {
        U256::ZERO
    }

    // --- Keccak ---------------------------------------------------------------------------------

    /// `n` calls to the SDK's `keccak`, which is the host's `native_keccak256`.
    ///
    /// The control. Chained, so the hash of one round feeds the next and nothing can be hoisted.
    pub fn keccak_sdk_loop(&self, n: U256, seed: FixedBytes<32>) -> FixedBytes<32> {
        let mut acc = seed;
        let mut i = U256::ZERO;
        while i < n {
            acc = crypto::keccak(acc.as_slice());
            i += U256::from(1);
        }
        acc
    }

    /// `n` Keccak-f[1600] permutations, the primitive SHAKE is built from.
    pub fn keccak_f_loop(&self, n: U256) -> U256 {
        let mut state = [0u64; 25];
        let mut i = U256::ZERO;
        while i < n {
            keccak::keccak_f(&mut state);
            i += U256::from(1);
        }
        U256::from(state[0])
    }

    /// SHAKE256 over `input_len` zero bytes, squeezing `out_len`, `n` times.
    ///
    /// The real shape: ML-DSA-44 verification absorbs a few hundred bytes and squeezes a few
    /// thousand, so the interesting rows are the ones where the squeeze dominates.
    pub fn shake256_loop(&self, n: U256, input_len: U256, out_len: U256) -> FixedBytes<32> {
        let input = vec![0u8; input_len.to::<usize>()];
        let mut out = vec![0u8; out_len.to::<usize>()];
        let mut i = U256::ZERO;
        while i < n {
            keccak::shake256(&input, &mut out);
            // Chain it, so `n` rounds cannot collapse to one.
            if !input.is_empty() {
                out[0] ^= 1;
            }
            i += U256::from(1);
        }
        let mut digest = [0u8; 32];
        let take = core::cmp::min(32, out.len());
        digest[..take].copy_from_slice(&out[..take]);
        FixedBytes::from(digest)
    }

    /// SHAKE256 of `input`, squeezing 32 bytes. For checking against the Solidity twin.
    pub fn shake256(&self, input: Bytes) -> FixedBytes<32> {
        let mut out = [0u8; 32];
        keccak::shake256(&input, &mut out);
        FixedBytes::from(out)
    }

    // --- the transform --------------------------------------------------------------------------

    /// `n` forward NTTs over ML-DSA's ring, reducing with `%` as the Solidity twin's `mulmod` does.
    pub fn ntt_loop(&self, n: U256) -> U256 {
        let mut poly = [0u32; 256];
        let mut i = U256::ZERO;
        while i < n {
            for (j, slot) in poly.iter_mut().enumerate() {
                *slot = (*slot + j as u32) % ntt::Q;
            }
            ntt::ntt(&mut poly);
            i += U256::from(1);
        }
        U256::from(poly[0])
    }

    /// The same, in the Montgomery domain -- how ML-DSA actually runs it, with no division at all.
    ///
    /// Solidity has nothing to compare this against: `MULMOD` reduces for free, so there is no
    /// Montgomery version of the Solidity side to write. It is here to say how much further Rust
    /// goes once it stops imitating the EVM.
    pub fn ntt_montgomery_loop(&self, n: U256) -> U256 {
        let mut poly = [0i32; 256];
        let mut i = U256::ZERO;
        while i < n {
            for (j, slot) in poly.iter_mut().enumerate() {
                *slot = ntt::to_mont(j as u32).wrapping_add(*slot % ntt::Q as i32);
            }
            ntt::ntt_montgomery(&mut poly);
            i += U256::from(1);
        }
        U256::from(ntt::from_mont(poly[0]))
    }

    /// One forward NTT of `0..256`, first four coefficients. For checking against the twin.
    pub fn ntt_vector(&self) -> Vec<U256> {
        let mut poly = [0u32; 256];
        for (i, slot) in poly.iter_mut().enumerate() {
            *slot = i as u32;
        }
        ntt::ntt(&mut poly);
        poly[..4].iter().map(|v| U256::from(*v)).collect()
    }
}
