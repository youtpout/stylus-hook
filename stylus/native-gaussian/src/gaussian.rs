// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Gaussian CDF and PDF in 1e18 fixed point: a bit-exact port of `primitivefinance/solstat`,
//! Solmate's `expWad` included, pinned by [`tests`] to values from the compiled Solidity.
//!
//! Hence the arithmetic below — wrapping multiplies, arithmetic shifts, truncating division — which
//! matches what the EVM does rather than what is natural in Rust.

use alloy_primitives::{I256, U256};

/// 1e18.
pub const ONE: I256 = I256::from_raw(U256::from_limbs([1_000_000_000_000_000_000, 0, 0, 0]));
/// 2e18.
pub const TWO: I256 = I256::from_raw(U256::from_limbs([2_000_000_000_000_000_000, 0, 0, 0]));

/// `sqrt(2)`, 18 decimals.
const SQRT2: I256 = I256::from_raw(U256::from_limbs([1_414_213_562_373_095_048, 0, 0, 0]));
/// `sqrt(2*pi)`, 18 decimals.
const SQRT_2PI: I256 = I256::from_raw(U256::from_limbs([2_506_628_274_631_000_502, 0, 0, 0]));
/// Above this, `erfc` is zero to 18 decimals.
const ERFC_DOMAIN_UPPER: I256 =
    I256::from_raw(U256::from_limbs([6_240_000_000_000_000_000, 0, 0, 0]));

// The Numerical Recipes 3e p265 coefficients, 18 decimals, as solstat spells them.
const ERFC_A: I256 = I256::from_raw(U256::from_limbs([1_265_512_230_000_000_000, 0, 0, 0]));
const ERFC_B: I256 = I256::from_raw(U256::from_limbs([1_000_023_680_000_000_000, 0, 0, 0]));
const ERFC_C: I256 = I256::from_raw(U256::from_limbs([374_091_960_000_000_000, 0, 0, 0]));
const ERFC_D: I256 = I256::from_raw(U256::from_limbs([96_784_180_000_000_000, 0, 0, 0]));
const ERFC_E: I256 = neg(186_288_060_000_000_000);
const ERFC_F: I256 = I256::from_raw(U256::from_limbs([278_868_070_000_000_000, 0, 0, 0]));
const ERFC_G: I256 = neg(1_135_203_980_000_000_000);
const ERFC_H: I256 = I256::from_raw(U256::from_limbs([1_488_515_870_000_000_000, 0, 0, 0]));
const ERFC_I: I256 = neg(822_152_230_000_000_000);
const ERFC_J: I256 = I256::from_raw(U256::from_limbs([170_872_770_000_000_000, 0, 0, 0]));

/// A negative constant, built by two's complement so it can be `const`.
const fn neg(magnitude: u64) -> I256 {
    I256::from_raw(U256::from_limbs([magnitude, 0, 0, 0]).wrapping_neg())
}

const fn from_limbs(limbs: [u64; 4]) -> I256 {
    I256::from_raw(U256::from_limbs(limbs))
}

/// The same, negated, for constants too wide for a single limb.
const fn neg_limbs(limbs: [u64; 4]) -> I256 {
    I256::from_raw(U256::from_limbs(limbs).wrapping_neg())
}

// Solmate `expWad`. `5**18`, the two domain bounds, and the rational approximation's coefficients.
const FIVE_POW_18: I256 = from_limbs([3_814_697_265_625, 0, 0, 0]);
const EXP_LOWER: I256 = neg_limbs([5_246_190_707_033_664_319, 2, 0, 0]); // -42139678854452767551
const EXP_UPPER: I256 = from_limbs([6_178_790_852_926_370_277, 7, 0, 0]); // 135305999368893231589
const LN2_X96: I256 = from_limbs([15_118_436_252_839_555_992, 2_977_044_471, 0, 0]);
const P1: I256 = from_limbs([4_021_714_997_422_814_800, 72_987_764_733, 0, 0]);
const P2: I256 = from_limbs([15_103_936_341_470_079_466, 3_098_401_593_211, 0, 0]);
const P3: I256 = from_limbs([10_850_458_028_954_966_636, 5_106_676_214_411, 0, 0]);
const P4: I256 = from_limbs([11_323_734_702_077_600_848, 1_556_861_282_905_762, 0, 0]);
const P5: I256 = from_limbs([11_023_459_180_255_945_820, 237_726_099_735_116, 0, 0]);
const Q1: I256 = from_limbs([12_887_555_761_442_761_468, 154_823_495_327, 0, 0]);
const Q2: I256 = from_limbs([14_438_380_032_906_984_665, 2_711_622_357_455, 0, 0]);
const Q3: I256 = from_limbs([7_660_730_384_882_190_788, 28_939_797_259_087, 0, 0]);
const Q4: I256 = from_limbs([14_958_195_001_563_919_525, 195_419_703_473_219, 0, 0]);
const Q5: I256 = from_limbs([5_884_290_600_356_364_053, 781_905_387_190_095, 0, 0]);
const Q6: I256 = from_limbs([5_163_562_563_322_150_231, 1_433_813_381_519_787, 0, 0]);
/// The combined scale factor, `2**k` and base conversion, applied in one unsigned multiply.
const EXP_SCALE: U256 = U256::from_limbs([
    17_181_495_799_676_635_891,
    7_188_640_403_681_034_642,
    11_234_296_709,
    0,
]);

/// `x * y / 1e18`, as solstat's `muliWad`.
#[inline]
fn muli_wad(x: I256, y: I256) -> I256 {
    x.wrapping_mul(y) / ONE
}

/// `x * 1e18 / y`, as solstat's `diviWad`.
#[inline]
fn divi_wad(x: I256, y: I256) -> I256 {
    x.wrapping_mul(ONE) / y
}

/// `e^x` in 1e18 fixed point, as Solmate's `expWad`.
///
/// Converts to a base-2^96 fixed point for intermediate precision, factors out `2^k` so the
/// remainder lands in `(-ln2/2, ln2/2)`, evaluates a (6,7)-term rational approximation there, and
/// folds the scale factor, the `2^k` and the base conversion back in with one multiply.
pub fn exp_wad(x: I256) -> I256 {
    if x <= EXP_LOWER {
        return I256::ZERO;
    }
    // Solidity reverts here. Saturating instead would silently change the function, so this keeps
    // the same contract: the caller must not ask for more than an int256 can hold.
    if x >= EXP_UPPER {
        panic!("EXP_OVERFLOW");
    }

    let mut x = (x << 78usize) / FIVE_POW_18;

    let k = (((x << 96usize) / LN2_X96) + (I256::ONE << 95usize)).asr(96);
    x -= k.wrapping_mul(LN2_X96);

    let mut y = x + P1;
    y = y.wrapping_mul(x).asr(96) + P2;
    let mut p = y + x - P3;
    p = p.wrapping_mul(y).asr(96) + P4;
    p = p.wrapping_mul(x) + (P5 << 96usize);

    let mut q = x - Q1;
    q = q.wrapping_mul(x).asr(96) + Q2;
    q = q.wrapping_mul(x).asr(96) - Q3;
    q = q.wrapping_mul(x).asr(96) + Q4;
    q = q.wrapping_mul(x).asr(96) - Q5;
    q = q.wrapping_mul(x).asr(96) + Q6;

    let r = p / q;

    // k is in [-61, 195], so the shift is in [0, 256] and the final basis is always positive.
    let shift = (I256::unchecked_from(195i64) - k).into_raw().as_limbs()[0] as usize;
    I256::from_raw(r.into_raw().wrapping_mul(EXP_SCALE) >> shift)
}

/// The complementary error function, `erfc(x) = 1 - erf(x)`, in 1e18 fixed point.
///
/// The Chebyshev fit from Numerical Recipes 3e p265, as solstat spells it. Maximum error about
/// 1.2e-7 against the true value.
pub fn erfc(input: I256) -> I256 {
    if input.is_zero() {
        return ONE;
    }
    if input >= ERFC_DOMAIN_UPPER {
        return I256::ZERO;
    }
    if input <= -ERFC_DOMAIN_UPPER {
        return TWO;
    }

    let z = if input.is_negative() { -input } else { input };
    // `1 / (1 + z/2)`, with the inner quotient taken in unsigned arithmetic as solstat does.
    let t = divi_wad(
        ONE,
        ONE + I256::from_raw(z.into_raw() * ONE.into_raw() / TWO.into_raw()),
    );

    let step = ERFC_F
        + muli_wad(
            t,
            ERFC_G + muli_wad(t, ERFC_H + muli_wad(t, ERFC_I + muli_wad(t, ERFC_J))),
        );
    let step = muli_wad(
        t,
        ERFC_B
            + muli_wad(
                t,
                ERFC_C + muli_wad(t, ERFC_D + muli_wad(t, ERFC_E + muli_wad(t, step))),
            ),
    );

    let k = (-muli_wad(z, z) - ERFC_A) + step;
    let r = muli_wad(t, exp_wad(k));

    if input.is_negative() {
        TWO - r
    } else {
        r
    }
}

/// The standard normal cumulative distribution function, in 1e18 fixed point.
pub fn cdf(x: I256) -> I256 {
    let input = x.wrapping_mul(ONE) / SQRT2;
    erfc(-input).wrapping_mul(ONE) / TWO
}

/// The standard normal probability density function, in 1e18 fixed point.
pub fn pdf(x: I256) -> I256 {
    let e = exp_wad(-x.wrapping_mul(x) / TWO);
    e.wrapping_mul(ONE) / SQRT_2PI
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> I256 {
        s.parse().unwrap()
    }

    /// EVM semantics this port depends on. If any of these ever change under us, the bit-exactness
    /// below would drift silently, so they are asserted rather than assumed.
    #[test]
    fn the_evm_rounding_rules_hold_in_alloy() {
        // alloy's `>>` on a signed integer is a *logical* shift: it turns -1 into 2^255 - 1.
        // Solidity's `>>` on an int256 is `SAR`. Getting this wrong is silent, so the port uses
        // `asr` everywhere and this pins the difference.
        assert_ne!(
            I256::MINUS_ONE >> 1usize,
            I256::MINUS_ONE,
            "`>>` is logical, not arithmetic"
        );
        assert_eq!(
            I256::MINUS_ONE.asr(1),
            I256::MINUS_ONE,
            "`asr` is the arithmetic one"
        );
        assert_eq!(
            I256::unchecked_from(-7i64).asr(1),
            I256::unchecked_from(-4i64),
            "`sar` rounds toward negative infinity"
        );
        assert_eq!(
            I256::unchecked_from(-7i64) / I256::unchecked_from(2i64),
            I256::unchecked_from(-3i64),
            "`sdiv` truncates toward zero"
        );
        assert_eq!(LN2_X96.to_string(), "54916777467707473351141471128");
        assert_eq!(EXP_LOWER.to_string(), "-42139678854452767551");
    }

    /// Every value here came out of the compiled Solidity — Solmate's `expWad` and solstat's
    /// `Gaussian` — and the port has to reproduce all of them exactly.
    #[test]
    fn the_port_matches_the_solidity_to_the_wei() {
        let cases: [(&str, &str, &str, &str, &str); 9] = [
            // x, expWad, erfc, cdf, pdf
            (
                "-3000000000000000000",
                "49787068367863942",
                "1999977909501578499",
                "1349898073255602",
                "4431848411938006",
            ),
            (
                "-1000000000000000000",
                "367879441171442321",
                "1842700787760006725",
                "158655261395674625",
                "241970724519143349",
            ),
            (
                "-400000000000000000",
                "670320046035639300",
                "1428392402898957764",
                "344578250551125821",
                "368270140303323307",
            ),
            (
                "-1",
                "999999999999999999",
                "999999969999999550",
                "500000000000000000",
                "398942280401432678",
            ),
            (
                "0",
                "1000000000000000000",
                "1000000000000000000",
                "500000000000000000",
                "398942280401432678",
            ),
            (
                "1",
                "1000000000000000001",
                "1000000030000000450",
                "500000000000000000",
                "398942280401432678",
            ),
            (
                "400000000000000000",
                "1491824697641270317",
                "571607597101042236",
                "655421749448874179",
                "368270140303323307",
            ),
            (
                "1000000000000000000",
                "2718281828459045235",
                "157299212239993275",
                "841344738604325374",
                "241970724519143349",
            ),
            (
                "3000000000000000000",
                "20085536923187667741",
                "22090498421501",
                "998650101926744398",
                "4431848411938006",
            ),
        ];
        for (x, e, ec, c, p) in cases {
            let x = d(x);
            assert_eq!(exp_wad(x), d(e), "expWad({x})");
            assert_eq!(erfc(x), d(ec), "erfc({x})");
            assert_eq!(cdf(x), d(c), "cdf({x})");
            assert_eq!(pdf(x), d(p), "pdf({x})");
        }
    }

    /// The `2^k` range reduction is where a port goes wrong, so the far ends get their own cases.
    #[test]
    fn the_range_reduction_matches_at_the_extremes() {
        assert_eq!(
            exp_wad(d("20000000000000000000")),
            d("485165195409790277974777105")
        );
        assert_eq!(exp_wad(d("-20000000000000000000")), d("2061153622"));
        assert_eq!(
            exp_wad(d("100000000000000000000")),
            d("26881171418161354484134666106240937146178367581647816351662017")
        );
        assert_eq!(
            exp_wad(d("-42139678854452767551")),
            I256::ZERO,
            "below the domain"
        );
    }

    #[test]
    fn the_distribution_behaves_like_one() {
        assert_eq!(cdf(I256::ZERO), ONE / I256::unchecked_from(2i64));
        // Symmetry, to the wei the truncations allow.
        let a = cdf(d("700000000000000000"));
        let b = cdf(d("-700000000000000000"));
        assert!((a + b - ONE).abs() < I256::unchecked_from(10i64));
        assert!(cdf(d("2000000000000000000")) > cdf(d("1000000000000000000")));
        assert!(pdf(I256::ZERO) > pdf(ONE));
    }
}
