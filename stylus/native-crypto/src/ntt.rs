//! The ML-DSA (Dilithium) number-theoretic transform, over the 23-bit prime `q = 8380417`.
//!
//! [`ntt`] uses `%`, matching the Solidity twin's `mulmod`, so the two agree byte for byte and the
//! comparison is a language comparison. [`ntt_montgomery`] is how a real implementation multiplies
//! and is measured separately, since `MULMOD` already reduces for free.

/// ML-DSA's prime, `2^23 - 2^13 + 1`.
pub const Q: u32 = 8380417;

/// `q^-1 mod 2^32`, for the Montgomery reduction. The value ML-DSA publishes.
const QINV: i32 = 58728449;

/// `zeta^bitrev8(k) mod q`, with `zeta = 1753` -- a primitive 512th root of unity mod `q`.
pub static ZETAS: [u32; 256] = [
    1, 4808194, 3765607, 3761513, 5178923, 5496691, 5234739, 5178987, 7778734, 3542485, 2682288,
    2129892, 3764867, 7375178, 557458, 7159240, 5010068, 4317364, 2663378, 6705802, 4855975,
    7946292, 676590, 7044481, 5152541, 1714295, 2453983, 1460718, 7737789, 4795319, 2815639,
    2283733, 3602218, 3182878, 2740543, 4793971, 5269599, 2101410, 3704823, 1159875, 394148,
    928749, 1095468, 4874037, 2071829, 4361428, 3241972, 2156050, 3415069, 1759347, 7562881,
    4805951, 3756790, 6444618, 6663429, 4430364, 5483103, 3192354, 556856, 3870317, 2917338,
    1853806, 3345963, 1858416, 3073009, 1277625, 5744944, 3852015, 4183372, 5157610, 5258977,
    8106357, 2508980, 2028118, 1937570, 4564692, 2811291, 5396636, 7270901, 4158088, 1528066,
    482649, 1148858, 5418153, 7814814, 169688, 2462444, 5046034, 4213992, 4892034, 1987814,
    5183169, 1736313, 235407, 5130263, 3258457, 5801164, 1787943, 5989328, 6125690, 3482206,
    4197502, 7080401, 6018354, 7062739, 2461387, 3035980, 621164, 3901472, 7153756, 2925816,
    3374250, 1356448, 5604662, 2683270, 5601629, 4912752, 2312838, 7727142, 7921254, 348812,
    8052569, 1011223, 6026202, 4561790, 6458164, 6143691, 1744507, 1753, 6444997, 5720892, 6924527,
    2660408, 6600190, 8321269, 2772600, 1182243, 87208, 636927, 4415111, 4423672, 6084020, 5095502,
    4663471, 8352605, 822541, 1009365, 5926272, 6400920, 1596822, 4423473, 4620952, 6695264,
    4969849, 2678278, 4611469, 4829411, 635956, 8129971, 5925040, 4234153, 6607829, 2192938,
    6653329, 2387513, 4768667, 8111961, 5199961, 3747250, 2296099, 1239911, 4541938, 3195676,
    2642980, 1254190, 8368000, 2998219, 141835, 8291116, 2513018, 7025525, 613238, 7070156,
    6161950, 7921677, 6458423, 4040196, 4908348, 2039144, 6500539, 7561656, 6201452, 6757063,
    2105286, 6006015, 6346610, 586241, 7200804, 527981, 5637006, 6903432, 1994046, 2491325,
    6987258, 507927, 7192532, 7655613, 6545891, 5346675, 8041997, 2647994, 3009748, 5767564,
    4148469, 749577, 4357667, 3980599, 2569011, 6764887, 1723229, 1665318, 2028038, 1163598,
    5011144, 3994671, 8368538, 7009900, 3020393, 3363542, 214880, 545376, 7609976, 3105558,
    7277073, 508145, 7826699, 860144, 3430436, 140244, 6866265, 6195333, 3123762, 2358373, 6187330,
    5365997, 6663603, 2926054, 7987710, 8077412, 3531229, 4405932, 4606686, 1900052, 7598542,
    1054478, 7648983,
];

/// The forward NTT, reducing with `%`.
pub fn ntt(a: &mut [u32; 256]) {
    let mut k = 0usize;
    let mut len = 128usize;
    while len >= 1 {
        let mut start = 0usize;
        while start < 256 {
            k += 1;
            let zeta = ZETAS[k] as u64;
            for j in start..start + len {
                let t = (zeta * a[j + len] as u64 % Q as u64) as u32;
                a[j + len] = (a[j] + Q - t) % Q;
                a[j] = (a[j] + t) % Q;
            }
            start += 2 * len;
        }
        len >>= 1;
    }
}

/// `montgomery_reduce`: `a * 2^-32 mod q`, for `|a| < q * 2^31`.
#[inline]
fn mont_reduce(a: i64) -> i32 {
    let t = (a as i32).wrapping_mul(QINV);
    ((a - t as i64 * Q as i64) >> 32) as i32
}

/// The same transform in the Montgomery domain, which is how ML-DSA actually runs it.
///
/// Inputs and outputs are `x * 2^32 mod q`. Values stay in `(-q, q)` as signed 32-bit, exactly as the
/// reference implementation does, so no reduction is needed inside the butterfly.
pub fn ntt_montgomery(a: &mut [i32; 256]) {
    let mut k = 0usize;
    let mut len = 128usize;
    while len >= 1 {
        let mut start = 0usize;
        while start < 256 {
            k += 1;
            let zeta = to_mont(ZETAS[k]) as i64;
            for j in start..start + len {
                let t = mont_reduce(zeta * a[j + len] as i64);
                a[j + len] = a[j] - t;
                a[j] += t;
            }
            start += 2 * len;
        }
        len >>= 1;
    }
}

/// `x * 2^32 mod q`.
pub fn to_mont(x: u32) -> i32 {
    ((x as u64) << 32).checked_rem(Q as u64).unwrap_or(0) as i32
}

/// `x * 2^-32 mod q`, brought back into `[0, q)`.
pub fn from_mont(x: i32) -> u32 {
    let r = mont_reduce(x as i64) % Q as i32;
    if r < 0 {
        (r + Q as i32) as u32
    } else {
        r as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeta_is_a_primitive_512th_root() {
        let pow = |mut base: u64, mut e: u32| {
            let mut acc = 1u64;
            while e > 0 {
                if e & 1 == 1 {
                    acc = acc * base % Q as u64;
                }
                base = base * base % Q as u64;
                e >>= 1;
            }
            acc
        };
        assert_eq!(pow(1753, 512), 1);
        assert_ne!(pow(1753, 256), 1);
        assert_eq!(ZETAS[0], 1);
        assert_eq!(ZETAS[1], 4808194);
    }

    /// The same vector `uniswap/src/CryptoBench.sol` asserts.
    #[test]
    fn the_transform_matches_the_solidity_twin() {
        let mut a = [0u32; 256];
        for (i, slot) in a.iter_mut().enumerate() {
            *slot = i as u32;
        }
        ntt(&mut a);
        assert_eq!(&a[..4], &[8023823, 4949942, 5503697, 7227518]);
    }

    /// Montgomery has to agree with `%` once it is brought back out of the domain, or the faster
    /// version is measuring the wrong thing.
    #[test]
    fn montgomery_agrees_with_the_plain_reduction() {
        let mut plain = [0u32; 256];
        let mut mont = [0i32; 256];
        for i in 0..256 {
            plain[i] = i as u32;
            mont[i] = to_mont(i as u32);
        }
        ntt(&mut plain);
        ntt_montgomery(&mut mont);
        for i in 0..256 {
            assert_eq!(from_mont(mont[i]), plain[i], "coefficient {i}");
        }
    }
}
