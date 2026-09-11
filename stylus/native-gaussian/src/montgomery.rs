// SPDX-License-Identifier: MIT OR Apache-2.0
//! Montgomery multiplication in BN254's scalar field, to settle whether Stylus can beat `MULMOD`.
//!
//! The EVM has modular multiplication as a single opcode, `MULMOD`, at 8 gas. WASM has nothing of
//! the sort, and `ruint`'s `mul_mod` answers with a 512-bit product followed by a 512-by-256
//! division — measured at 94 gas against Solidity's 88, which says field arithmetic is not worth
//! porting.
//!
//! But no cryptography library multiplies that way. They keep values in Montgomery form, where the
//! reduction is a few multiply-accumulates and no division at all. That is what this is: CIOS
//! Montgomery, 4 limbs, for a fixed modulus. If the EVM's opcode still wins against *this*, then
//! every 256-bit prime field — Poseidon over BN254, P-256 signature verification, any SNARK
//! verifier — is genuinely out of reach, and the only fields worth porting are the small ones.

/// BN254's scalar field modulus, little-endian limbs.
pub const P: [u64; 4] = [
    0x43e1_f593_f000_0001,
    0x2833_e848_79b9_7091,
    0xb850_45b6_8181_585d,
    0x3064_4e72_e131_a029,
];

/// `-P^{-1} mod 2^64`.
const N0INV: u64 = 0xc2e1_f593_efff_ffff;

/// `a * b * R^{-1} mod P`, with `R = 2^256` — the CIOS form, which needs no division.
#[inline]
pub fn mont_mul(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
    let mut t = [0u64; 6];
    for &bi in b.iter() {
        // t += a * bi
        let mut carry = 0u128;
        for j in 0..4 {
            let cur = t[j] as u128 + (a[j] as u128) * (bi as u128) + carry;
            t[j] = cur as u64;
            carry = cur >> 64;
        }
        let cur = t[4] as u128 + carry;
        t[4] = cur as u64;
        t[5] = (cur >> 64) as u64;

        // m = t[0] * N0INV mod 2^64, then t += m * P, which zeroes t[0]
        let m = (t[0] as u128 * N0INV as u128) as u64;
        let mut carry = 0u128;
        for j in 0..4 {
            let cur = t[j] as u128 + (m as u128) * (P[j] as u128) + carry;
            t[j] = cur as u64;
            carry = cur >> 64;
        }
        let cur = t[4] as u128 + carry;
        t[4] = cur as u64;
        t[5] = t[5].wrapping_add((cur >> 64) as u64);

        // shift down one limb
        t[0] = t[1];
        t[1] = t[2];
        t[2] = t[3];
        t[3] = t[4];
        t[4] = t[5];
        t[5] = 0;
    }

    let mut r = [t[0], t[1], t[2], t[3]];
    if t[4] != 0 || !lt(&r, &P) {
        sub_p(&mut r);
    }
    r
}

#[inline]
fn lt(a: &[u64; 4], b: &[u64; 4]) -> bool {
    for i in (0..4).rev() {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
    }
    false
}

#[inline]
fn sub_p(a: &mut [u64; 4]) {
    let mut borrow = 0i128;
    for i in 0..4 {
        let cur = a[i] as i128 - P[i] as i128 - borrow;
        a[i] = cur as u64;
        borrow = if cur < 0 { 1 } else { 0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;

    fn p() -> U256 {
        U256::from_limbs(P)
    }

    fn to_mont(x: U256) -> [u64; 4] {
        // x * R mod P, with R = 2^256. Computed as (x << 256) mod P via repeated doubling, since
        // there is no wider type to hand.
        let mut acc = x % p();
        for _ in 0..256 {
            acc = acc.add_mod(acc, p());
        }
        acc.into_limbs()
    }

    /// The property that matters: Montgomery multiplication has to agree with `mulmod`.
    #[test]
    fn montgomery_agrees_with_plain_modular_multiplication() {
        let cases = [
            (U256::from(2u64), U256::from(3u64)),
            (U256::from(1u64), U256::from(1u64)),
            (p() - U256::from(1u64), p() - U256::from(2u64)),
            (
                U256::from_str_radix("123456789abcdef0fedcba9876543210", 16).unwrap(),
                U256::from_str_radix("fedcba9876543210123456789abcdef0", 16).unwrap(),
            ),
        ];
        for (a, b) in cases {
            let want = to_mont(a.mul_mod(b, p()));
            let got = mont_mul(&to_mont(a), &to_mont(b));
            assert_eq!(
                U256::from_limbs(got),
                U256::from_limbs(want),
                "mont_mul disagrees for a={a} b={b}"
            );
        }
    }

    #[test]
    fn the_result_is_always_reduced() {
        let a = to_mont(p() - U256::from(1u64));
        let r = U256::from_limbs(mont_mul(&a, &a));
        assert!(r < p(), "result {r} is not reduced");
    }
}
