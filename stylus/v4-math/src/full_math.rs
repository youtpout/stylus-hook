// SPDX-License-Identifier: MIT OR Apache-2.0
//! `FullMath`, as v4-core spells it: `a * b / denominator` with the product allowed to exceed 256
//! bits, and the result required not to.
//!
//! Solidity gets there through Remco Bloemen's trick — recover the high word with
//! `mulmod(a, b, not(0))`, then divide by inverting the denominator modulo 2^256, because the EVM has
//! no 512-bit division. None of that is necessary here: `widening_mul` gives the exact 512-bit
//! product and `ruint` can divide it. The result is the same value by a shorter route, which is the
//! whole reason this port exists.

use alloy_primitives::{U256, U512};

/// A 512-bit value back down to 256, or `None` if it does not fit.
///
/// `ruint` has no conversion between widths, so this checks the upper four limbs by hand. It is what
/// stands in for Solidity's implicit overflow.
#[inline]
fn narrow(value: U512) -> Option<U256> {
    let limbs = value.as_limbs();
    if limbs[4] | limbs[5] | limbs[6] | limbs[7] != 0 {
        return None;
    }
    Some(U256::from_limbs([limbs[0], limbs[1], limbs[2], limbs[3]]))
}

/// `floor(a * b / denominator)`.
///
/// # Panics
///
/// If `denominator` is zero, or if the result would not fit in 256 bits — matching the `require`
/// and the implicit overflow in v4-core's `mulDiv`.
#[inline]
pub fn mul_div(a: U256, b: U256, denominator: U256) -> U256 {
    assert!(!denominator.is_zero(), "FullMath: division by zero");
    let product: U512 = a.widening_mul(b);
    let quotient = product / U512::from(denominator);
    narrow(quotient).expect("FullMath: result overflows 256 bits")
}

/// `ceil(a * b / denominator)`.
///
/// # Panics
///
/// As [`mul_div`], and also if rounding up would carry the result past 256 bits — v4-core's
/// `require(++result > 0)`.
#[inline]
pub fn mul_div_rounding_up(a: U256, b: U256, denominator: U256) -> U256 {
    assert!(!denominator.is_zero(), "FullMath: division by zero");
    let product: U512 = a.widening_mul(b);
    let d = U512::from(denominator);
    let (quotient, remainder) = (product / d, product % d);
    let quotient = if remainder.is_zero() {
        quotient
    } else {
        quotient + U512::from(1)
    };
    narrow(quotient).expect("FullMath: result overflows 256 bits")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q128() -> U256 {
        U256::from(1u8) << 128
    }

    fn d(s: &str) -> U256 {
        U256::from_str_radix(s, 10).unwrap()
    }

    // The cases below are v4-core's `test/libraries/FullMath.t.sol`, value for value.

    #[test]
    #[should_panic(expected = "division by zero")]
    fn mul_div_reverts_with_zero_denominator() {
        mul_div(q128(), q128(), U256::ZERO);
    }

    #[test]
    #[should_panic(expected = "overflows 256 bits")]
    fn mul_div_reverts_if_output_overflows() {
        mul_div(q128(), q128(), U256::from(1));
    }

    #[test]
    #[should_panic(expected = "overflows 256 bits")]
    fn mul_div_reverts_on_overflow_with_all_max_inputs() {
        mul_div(U256::MAX, U256::MAX, U256::MAX - U256::from(1));
    }

    #[test]
    fn mul_div_valid_with_all_max_inputs() {
        assert_eq!(mul_div(U256::MAX, U256::MAX, U256::MAX), U256::MAX);
    }

    #[test]
    fn mul_div_valid_without_phantom_overflow() {
        let expected = q128() / U256::from(3);
        let got = mul_div(
            q128(),
            U256::from(50) * q128() / U256::from(100),
            U256::from(150) * q128() / U256::from(100),
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn mul_div_valid_with_phantom_overflow() {
        let expected = U256::from(4375) * q128() / U256::from(1000);
        let got = mul_div(q128(), U256::from(35) * q128(), U256::from(8) * q128());
        assert_eq!(got, expected);
    }

    #[test]
    fn mul_div_phantom_overflow_repeating_decimal() {
        let expected = q128() / U256::from(3);
        let got = mul_div(q128(), U256::from(1000) * q128(), U256::from(3000) * q128());
        assert_eq!(got, expected);
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn mul_div_rounding_up_reverts_with_zero_denominator() {
        mul_div_rounding_up(q128(), q128(), U256::ZERO);
    }

    #[test]
    fn mul_div_rounding_up_valid_with_all_max_inputs() {
        assert_eq!(
            mul_div_rounding_up(U256::MAX, U256::MAX, U256::MAX),
            U256::MAX
        );
    }

    #[test]
    fn mul_div_rounding_up_valid_with_no_phantom_overflow() {
        let expected = q128() / U256::from(3) + U256::from(1);
        let got = mul_div_rounding_up(
            q128(),
            U256::from(50) * q128() / U256::from(100),
            U256::from(150) * q128() / U256::from(100),
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn mul_div_rounding_up_valid_with_phantom_overflow_repeating_decimal() {
        let expected = q128() / U256::from(3) + U256::from(1);
        let got = mul_div_rounding_up(q128(), U256::from(1000) * q128(), U256::from(3000) * q128());
        assert_eq!(got, expected);
    }

    #[test]
    #[should_panic(expected = "overflows 256 bits")]
    fn mul_div_rounding_up_reverts_if_it_overflows_after_rounding_up() {
        mul_div_rounding_up(
            d("535006138814359"),
            d("432862656469423142931042426214547535783388063929571229938474969"),
            U256::from(2),
        );
    }

    #[test]
    #[should_panic(expected = "overflows 256 bits")]
    fn mul_div_rounding_up_reverts_if_it_overflows_after_rounding_up_case_2() {
        mul_div_rounding_up(
            d("115792089237316195423570985008687907853269984659341747863450311749907997002549"),
            d("115792089237316195423570985008687907853269984659341747863450311749907997002550"),
            d("115792089237316195423570985008687907853269984653042931687443039491902864365164"),
        );
    }

    /// v4-core checks this by fuzzing; here it is a sweep, which is the same property.
    #[test]
    fn rounding_up_is_never_more_than_one_above_rounding_down() {
        for x in [1u64, 7, 1_000_003, u64::MAX] {
            for y in [1u64, 3, 999_999, u64::MAX] {
                for den in [1u64, 2, 3, 7, u64::MAX] {
                    let (x, y, den) = (U256::from(x), U256::from(y), U256::from(den));
                    let floored = mul_div(x, y, den);
                    let ceiled = mul_div_rounding_up(x, y, den);
                    let exact = (x.widening_mul(y) % U512::from(den)).is_zero();
                    if exact {
                        assert_eq!(ceiled, floored);
                    } else {
                        assert_eq!(ceiled - floored, U256::from(1));
                    }
                }
            }
        }
    }
}
