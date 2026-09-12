// SPDX-License-Identifier: MIT OR Apache-2.0
//! `UnsafeMath`, as v4-core spells it. The name is Uniswap's: these skip the division-by-zero
//! check, so a zero denominator is the caller's problem.
//!
//! Here a zero `y` panics where an EVM `div` returns zero — the one place this port cannot match,
//! and the reason every caller in this crate guards the denominator first.

use alloy_primitives::U256;

/// `ceil(x / y)`, as `UnsafeMath.divRoundingUp`.
#[inline]
pub fn div_rounding_up(x: U256, y: U256) -> U256 {
    let (q, r) = (x / y, x % y);
    if r.is_zero() {
        q
    } else {
        q + U256::from(1)
    }
}

/// `(a * b) / denominator` with the product allowed to wrap, as `UnsafeMath.simpleMulDiv`.
///
/// Only sound where the caller knows the product cannot exceed 256 bits.
#[inline]
pub fn simple_mul_div(a: U256, b: U256, denominator: U256) -> U256 {
    a.wrapping_mul(b) / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(v: u64) -> U256 {
        U256::from(v)
    }

    /// v4-core `test/libraries/UnsafeMath.t.sol`.
    #[test]
    fn div_rounding_up_matches_v4_core() {
        assert_eq!(div_rounding_up(U256::ZERO, u(1)), U256::ZERO);
        assert_eq!(div_rounding_up(u(1), u(1)), u(1));
        assert_eq!(div_rounding_up(u(1), u(2)), u(1), "rounds up");
        assert_eq!(div_rounding_up(u(2), u(2)), u(1));
        assert_eq!(div_rounding_up(u(3), u(2)), u(2), "rounds up");
        assert_eq!(div_rounding_up(U256::MAX, U256::MAX), u(1));
        assert_eq!(div_rounding_up(U256::MAX, u(1)), U256::MAX);
        assert_eq!(div_rounding_up(U256::MAX, u(2)), U256::MAX / u(2) + u(1));
    }
}
