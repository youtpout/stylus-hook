// SPDX-License-Identifier: MIT OR Apache-2.0
//! `SqrtPriceMath`, as v4-core spells it: where a `sqrt(price)` lands after an amount goes in or out,
//! and the token deltas between two prices.
//!
//! Rounding is the whole substance of this module, and it is not symmetric: token0 rounds up and
//! token1 rounds down, so the pool never rounds in the trader's favour. Each function keeps v4-core's
//! direction exactly, because a hook that replays a swap and disagrees by one wei disagrees.

use alloy_primitives::{uint, U256};

use crate::full_math::{mul_div, mul_div_rounding_up};
use crate::unsafe_math::div_rounding_up;
use crate::{Q96, RESOLUTION};

/// Why a price move was refused. v4-core reverts with these as custom errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceError {
    /// `sqrtPX96` or `liquidity` was zero.
    InvalidPriceOrLiquidity,
    /// A price of zero has no reciprocal.
    InvalidPrice,
    /// The amount out is more than the liquidity can pay.
    NotEnoughLiquidity,
    /// The next price does not fit in a `uint160`.
    PriceOverflow,
}

/// `type(uint160).max`.
const MAX_U160: U256 = uint!(0xffffffffffffffffffffffffffffffffffffffff_U256);

fn to_u160(value: U256) -> Result<U256, PriceError> {
    if value > MAX_U160 {
        Err(PriceError::PriceOverflow)
    } else {
        Ok(value)
    }
}

/// The price after `amount` of token0 goes in or out, rounded up.
///
/// Rounds up so that adding token0 never moves the price less than it should, and removing it never
/// moves it more.
pub fn get_next_sqrt_price_from_amount0_rounding_up(
    sqrt_px96: U256,
    liquidity: U256,
    amount: U256,
    add: bool,
) -> Result<U256, PriceError> {
    if amount.is_zero() {
        return Ok(sqrt_px96);
    }
    let numerator1 = liquidity << RESOLUTION;

    if add {
        // The fast path needs the product not to have wrapped, exactly as v4-core checks.
        let product = amount.wrapping_mul(sqrt_px96);
        if product / amount == sqrt_px96 {
            let denominator = numerator1.wrapping_add(product);
            if denominator >= numerator1 {
                return to_u160(mul_div_rounding_up(numerator1, sqrt_px96, denominator));
            }
        }
        // Otherwise fall back to a form that cannot overflow, at the cost of a wei of precision.
        to_u160(div_rounding_up(
            numerator1,
            (numerator1 / sqrt_px96) + amount,
        ))
    } else {
        let product = amount.wrapping_mul(sqrt_px96);
        if product / amount != sqrt_px96 || numerator1 <= product {
            return Err(PriceError::PriceOverflow);
        }
        let denominator = numerator1 - product;
        to_u160(mul_div_rounding_up(numerator1, sqrt_px96, denominator))
    }
}

/// The price after `amount` of token1 goes in or out, rounded down.
pub fn get_next_sqrt_price_from_amount1_rounding_down(
    sqrt_px96: U256,
    liquidity: U256,
    amount: U256,
    add: bool,
) -> Result<U256, PriceError> {
    if add {
        let quotient = if amount <= MAX_U160 {
            (amount << RESOLUTION) / liquidity
        } else {
            mul_div(amount, Q96, liquidity)
        };
        to_u160(sqrt_px96 + quotient)
    } else {
        let quotient = if amount <= MAX_U160 {
            div_rounding_up(amount << RESOLUTION, liquidity)
        } else {
            mul_div_rounding_up(amount, Q96, liquidity)
        };
        if sqrt_px96 <= quotient {
            return Err(PriceError::NotEnoughLiquidity);
        }
        Ok(sqrt_px96 - quotient)
    }
}

/// The price after `amount_in` goes in, on whichever side `zero_for_one` names.
pub fn get_next_sqrt_price_from_input(
    sqrt_px96: U256,
    liquidity: U256,
    amount_in: U256,
    zero_for_one: bool,
) -> Result<U256, PriceError> {
    if sqrt_px96.is_zero() || liquidity.is_zero() {
        return Err(PriceError::InvalidPriceOrLiquidity);
    }
    if zero_for_one {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_in, true)
    }
}

/// The price after `amount_out` comes out.
pub fn get_next_sqrt_price_from_output(
    sqrt_px96: U256,
    liquidity: U256,
    amount_out: U256,
    zero_for_one: bool,
) -> Result<U256, PriceError> {
    if sqrt_px96.is_zero() || liquidity.is_zero() {
        return Err(PriceError::InvalidPriceOrLiquidity);
    }
    if zero_for_one {
        get_next_sqrt_price_from_amount1_rounding_down(sqrt_px96, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount0_rounding_up(sqrt_px96, liquidity, amount_out, false)
    }
}

/// The token0 between two prices, for `liquidity`.
pub fn get_amount0_delta(
    sqrt_price_a: U256,
    sqrt_price_b: U256,
    liquidity: U256,
    round_up: bool,
) -> Result<U256, PriceError> {
    let (lower, upper) = if sqrt_price_a > sqrt_price_b {
        (sqrt_price_b, sqrt_price_a)
    } else {
        (sqrt_price_a, sqrt_price_b)
    };
    if lower.is_zero() {
        return Err(PriceError::InvalidPrice);
    }
    let numerator1 = liquidity << RESOLUTION;
    let numerator2 = upper - lower;
    Ok(if round_up {
        div_rounding_up(mul_div_rounding_up(numerator1, numerator2, upper), lower)
    } else {
        mul_div(numerator1, numerator2, upper) / lower
    })
}

/// `|a - b|`, as v4-core's `absDiff`.
#[inline]
pub fn abs_diff(a: U256, b: U256) -> U256 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

/// The token1 between two prices, for `liquidity`.
///
/// Unlike token0 this needs no zero-price guard: the product cannot overflow, because
/// `uint128::MAX * uint160::MAX >> 96 < 1 << 192`.
pub fn get_amount1_delta(
    sqrt_price_a: U256,
    sqrt_price_b: U256,
    liquidity: U256,
    round_up: bool,
) -> U256 {
    let numerator = abs_diff(sqrt_price_a, sqrt_price_b);
    if round_up {
        mul_div_rounding_up(liquidity, numerator, Q96)
    } else {
        mul_div(liquidity, numerator, Q96)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> U256 {
        U256::from_str_radix(s, 10).unwrap()
    }

    /// `encodeSqrtPrice(1, 1)` in v4-core's tests.
    fn price_1_1() -> U256 {
        Q96
    }

    fn eth(n: u64) -> U256 {
        U256::from(n) * U256::from(10u64).pow(U256::from(18))
    }

    // The cases below are v4-core's `test/libraries/SqrtPriceMath.t.sol`, value for value.

    #[test]
    fn from_input_rejects_a_zero_price_or_liquidity() {
        assert_eq!(
            get_next_sqrt_price_from_input(U256::ZERO, U256::from(1), U256::from(1), true),
            Err(PriceError::InvalidPriceOrLiquidity)
        );
        assert_eq!(
            get_next_sqrt_price_from_input(U256::from(1), U256::ZERO, U256::from(1), true),
            Err(PriceError::InvalidPriceOrLiquidity)
        );
    }

    #[test]
    fn from_input_returns_the_price_unchanged_for_a_zero_amount() {
        for zero_for_one in [true, false] {
            assert_eq!(
                get_next_sqrt_price_from_input(price_1_1(), eth(1), U256::ZERO, zero_for_one),
                Ok(price_1_1())
            );
        }
    }

    /// "returns the minimum price for max inputs".
    #[test]
    fn from_input_returns_the_minimum_price_for_max_inputs() {
        let sqrt_p = MAX_U160;
        let liquidity = (U256::from(1u8) << 128) - U256::from(1);
        let max_amount_no_overflow = U256::MAX - ((liquidity << RESOLUTION) / sqrt_p);
        assert_eq!(
            get_next_sqrt_price_from_input(sqrt_p, liquidity, max_amount_no_overflow, true),
            Ok(U256::from(1))
        );
    }

    #[test]
    fn from_input_one_for_zero_on_a_tenth_of_an_eth() {
        assert_eq!(
            get_next_sqrt_price_from_input(price_1_1(), eth(1), eth(1) / U256::from(10), false),
            Ok(d("87150978765690771352898345369"))
        );
    }

    #[test]
    fn from_input_zero_for_one_on_a_tenth_of_an_eth() {
        assert_eq!(
            get_next_sqrt_price_from_input(price_1_1(), eth(1), eth(1) / U256::from(10), true),
            Ok(d("72025602285694852357767227579"))
        );
    }

    #[test]
    fn from_input_with_an_amount_that_overflows_the_fast_path() {
        assert_eq!(
            get_next_sqrt_price_from_input(
                price_1_1(),
                U256::from(10u64),
                U256::MAX / U256::from(2),
                true
            ),
            Ok(U256::from(1))
        );
    }

    #[test]
    fn from_output_rejects_a_zero_price_or_liquidity() {
        assert_eq!(
            get_next_sqrt_price_from_output(U256::ZERO, U256::from(1), U256::from(1), true),
            Err(PriceError::InvalidPriceOrLiquidity)
        );
        assert_eq!(
            get_next_sqrt_price_from_output(U256::from(1), U256::ZERO, U256::from(1), true),
            Err(PriceError::InvalidPriceOrLiquidity)
        );
    }

    #[test]
    fn from_output_reverts_if_the_amount_out_exceeds_the_reserves() {
        // token1 out, more than liquidity can pay.
        assert_eq!(
            get_next_sqrt_price_from_output(price_1_1(), eth(1), eth(2), true),
            Err(PriceError::NotEnoughLiquidity)
        );
    }

    #[test]
    fn from_output_one_for_zero_on_a_tenth_of_an_eth() {
        assert_eq!(
            get_next_sqrt_price_from_output(price_1_1(), eth(1), eth(1) / U256::from(10), false),
            Ok(d("88031291682515930659493278152"))
        );
    }

    #[test]
    fn from_output_zero_for_one_on_a_tenth_of_an_eth() {
        assert_eq!(
            get_next_sqrt_price_from_output(price_1_1(), eth(1), eth(1) / U256::from(10), true),
            Ok(d("71305346262837903834189555302"))
        );
    }

    #[test]
    fn amount0_delta_is_zero_for_a_zero_price_difference() {
        assert_eq!(
            get_amount0_delta(price_1_1(), price_1_1(), eth(1), true),
            Ok(U256::ZERO)
        );
    }

    #[test]
    fn amount0_delta_is_zero_for_zero_liquidity() {
        assert_eq!(
            get_amount0_delta(price_1_1(), price_1_1() * U256::from(2), U256::ZERO, true),
            Ok(U256::ZERO)
        );
    }

    #[test]
    fn amount0_delta_rounds_up_by_one_wei_against_rounding_down() {
        let a = price_1_1();
        let b = price_1_1() * U256::from(121) / U256::from(100);
        let up = get_amount0_delta(a, b, eth(1), true).unwrap();
        let down = get_amount0_delta(a, b, eth(1), false).unwrap();
        assert_eq!(up, down + U256::from(1));
    }

    #[test]
    fn amount0_delta_rejects_a_zero_price() {
        assert_eq!(
            get_amount0_delta(U256::ZERO, price_1_1(), eth(1), true),
            Err(PriceError::InvalidPrice)
        );
    }

    #[test]
    fn amount1_delta_is_zero_for_a_zero_price_difference() {
        assert_eq!(
            get_amount1_delta(price_1_1(), price_1_1(), eth(1), true),
            U256::ZERO
        );
    }

    #[test]
    fn amount1_delta_is_zero_for_zero_liquidity() {
        assert_eq!(
            get_amount1_delta(price_1_1(), price_1_1() * U256::from(2), U256::ZERO, true),
            U256::ZERO
        );
    }

    #[test]
    fn amount1_delta_rounds_up_by_one_wei_against_rounding_down() {
        let a = price_1_1();
        let b = price_1_1() * U256::from(121) / U256::from(100);
        let up = get_amount1_delta(a, b, eth(1), true);
        let down = get_amount1_delta(a, b, eth(1), false);
        assert_eq!(up, down + U256::from(1));
    }

    /// `absDiff` is symmetric, which is the only thing it promises.
    #[test]
    fn abs_diff_is_symmetric() {
        assert_eq!(abs_diff(U256::from(5), U256::from(3)), U256::from(2));
        assert_eq!(abs_diff(U256::from(3), U256::from(5)), U256::from(2));
        assert_eq!(abs_diff(MAX_U160, U256::ZERO), MAX_U160);
    }

    /// v4-core's echidna-derived boundary cases. At this price and liquidity the
    /// virtual reserves are exactly 4 of token0 and 262144 of token1, so the two
    /// libraries have to agree on which side of the boundary reverts.
    const ECHIDNA_PRICE: &str = "20282409603651670423947251286016";

    #[test]
    fn from_output_reverts_when_amount_out_is_the_whole_currency0_reserve() {
        let l = U256::from(1024u64);
        assert_eq!(
            get_next_sqrt_price_from_output(d(ECHIDNA_PRICE), l, U256::from(4u64), false),
            Err(PriceError::PriceOverflow)
        );
        assert_eq!(
            get_next_sqrt_price_from_output(d(ECHIDNA_PRICE), l, U256::from(5u64), false),
            Err(PriceError::PriceOverflow)
        );
    }

    #[test]
    fn from_output_reverts_when_amount_out_reaches_the_currency1_reserve() {
        let l = U256::from(1024u64);
        assert_eq!(
            get_next_sqrt_price_from_output(d(ECHIDNA_PRICE), l, U256::from(262144u64), true),
            Err(PriceError::NotEnoughLiquidity)
        );
        assert_eq!(
            get_next_sqrt_price_from_output(d(ECHIDNA_PRICE), l, U256::from(262145u64), true),
            Err(PriceError::NotEnoughLiquidity)
        );
    }

    #[test]
    fn from_output_succeeds_one_wei_below_the_currency1_reserve() {
        assert_eq!(
            get_next_sqrt_price_from_output(
                d(ECHIDNA_PRICE),
                U256::from(1024u64),
                U256::from(262143u64),
                true
            ),
            Ok(d("77371252455336267181195264"))
        );
    }

    #[test]
    fn from_input_handles_an_amount_wider_than_uint96() {
        assert_eq!(
            get_next_sqrt_price_from_input(price_1_1(), eth(10), U256::from(1u64) << 100, true),
            Ok(U256::from(624999999995069620u64))
        );
    }

    #[test]
    fn from_input_returns_the_input_price_for_a_zero_amount() {
        assert_eq!(
            get_next_sqrt_price_from_input(price_1_1(), eth(1) / U256::from(10), U256::ZERO, true),
            Ok(price_1_1())
        );
        assert_eq!(
            get_next_sqrt_price_from_input(price_1_1(), eth(1) / U256::from(10), U256::ZERO, false),
            Ok(price_1_1())
        );
    }

    #[test]
    fn from_output_returns_the_input_price_for_a_zero_amount() {
        assert_eq!(
            get_next_sqrt_price_from_output(price_1_1(), eth(1) / U256::from(10), U256::ZERO, true),
            Ok(price_1_1())
        );
        assert_eq!(
            get_next_sqrt_price_from_output(
                price_1_1(),
                eth(1) / U256::from(10),
                U256::ZERO,
                false
            ),
            Ok(price_1_1())
        );
    }
}
