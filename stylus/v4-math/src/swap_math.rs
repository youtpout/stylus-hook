//! One step of a swap inside a single tick, ported from v4-core's `SwapMath`.

use alloy_primitives::{uint, I256, U256};

use crate::full_math::{mul_div, mul_div_rounding_up};
use crate::sqrt_price_math::{
    get_amount0_delta, get_amount1_delta, get_next_sqrt_price_from_input,
    get_next_sqrt_price_from_output, PriceError,
};

/// A 100% fee, in hundredths of a bip.
pub const MAX_SWAP_FEE: u64 = 1_000_000;

const MAX_SWAP_FEE_U256: U256 = uint!(1_000_000_U256);

/// The result of one swap step: the price reached, and the three amounts it moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwapStep {
    pub sqrt_price_next: U256,
    pub amount_in: U256,
    pub amount_out: U256,
    pub fee_amount: U256,
}

/// Whichever of the next initialised tick and the caller's limit the swap hits first.
#[inline]
pub fn get_sqrt_price_target(
    zero_for_one: bool,
    sqrt_price_next: U256,
    sqrt_price_limit: U256,
) -> U256 {
    let limit_binds = if zero_for_one {
        sqrt_price_next < sqrt_price_limit
    } else {
        sqrt_price_next > sqrt_price_limit
    };
    if limit_binds {
        sqrt_price_limit
    } else {
        sqrt_price_next
    }
}

/// How far a swap gets before it runs out of the amount specified or the price target.
///
/// `amount_remaining` follows v4's sign convention: negative is exact input. `fee_pips` must
/// already be at most [`MAX_SWAP_FEE`], and exact-output callers strictly below it.
pub fn compute_swap_step(
    sqrt_price_current: U256,
    sqrt_price_target: U256,
    liquidity: U256,
    amount_remaining: I256,
    fee_pips: u32,
) -> Result<SwapStep, PriceError> {
    let fee = U256::from(fee_pips);
    let zero_for_one = sqrt_price_current >= sqrt_price_target;
    let exact_in = amount_remaining.is_negative();

    // `unsigned_abs` is what Solidity's unchecked `uint256(-amountRemaining)` computes, including
    // at `int256::MIN` where the negation wraps back onto itself.
    let specified = amount_remaining.unsigned_abs();

    let (sqrt_price_next, amount_in, amount_out, fee_amount) = if exact_in {
        let remaining_less_fee = mul_div(specified, MAX_SWAP_FEE_U256 - fee, MAX_SWAP_FEE_U256);
        let max_in = if zero_for_one {
            get_amount0_delta(sqrt_price_target, sqrt_price_current, liquidity, true)?
        } else {
            get_amount1_delta(sqrt_price_current, sqrt_price_target, liquidity, true)
        };

        let (next, amount_in, fee_amount) = if remaining_less_fee >= max_in {
            // The target price caps the input; the fee is grossed back up from it.
            let fee_amount = if fee == MAX_SWAP_FEE_U256 {
                // `max_in` is necessarily zero here, so there is nothing to gross up — and the
                // usual formula would divide by zero.
                max_in
            } else {
                mul_div_rounding_up(max_in, fee, MAX_SWAP_FEE_U256 - fee)
            };
            (sqrt_price_target, max_in, fee_amount)
        } else {
            // The amount runs out first, so everything left over after the input is the fee.
            let next = get_next_sqrt_price_from_input(
                sqrt_price_current,
                liquidity,
                remaining_less_fee,
                zero_for_one,
            )?;
            (next, remaining_less_fee, specified - remaining_less_fee)
        };

        let amount_out = if zero_for_one {
            get_amount1_delta(next, sqrt_price_current, liquidity, false)
        } else {
            get_amount0_delta(sqrt_price_current, next, liquidity, false)?
        };
        (next, amount_in, amount_out, fee_amount)
    } else {
        let max_out = if zero_for_one {
            get_amount1_delta(sqrt_price_target, sqrt_price_current, liquidity, false)
        } else {
            get_amount0_delta(sqrt_price_current, sqrt_price_target, liquidity, false)?
        };

        let (next, amount_out) = if specified >= max_out {
            (sqrt_price_target, max_out)
        } else {
            let next = get_next_sqrt_price_from_output(
                sqrt_price_current,
                liquidity,
                specified,
                zero_for_one,
            )?;
            (next, specified)
        };

        let amount_in = if zero_for_one {
            get_amount0_delta(next, sqrt_price_current, liquidity, true)?
        } else {
            get_amount1_delta(sqrt_price_current, next, liquidity, true)
        };
        // Exact output cannot carry a 100% fee, so this division is safe.
        let fee_amount = mul_div_rounding_up(amount_in, fee, MAX_SWAP_FEE_U256 - fee);
        (next, amount_in, amount_out, fee_amount)
    };

    Ok(SwapStep {
        sqrt_price_next,
        amount_in,
        amount_out,
        fee_amount,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> U256 {
        s.parse().unwrap()
    }

    fn eth(n: u64) -> U256 {
        U256::from(n) * U256::from(1_000_000_000_000_000_000u64)
    }

    fn price_1_1() -> U256 {
        d("79228162514264337593543950336")
    }
    fn price_101_100() -> U256 {
        d("79623317895830914510639640423")
    }
    fn price_1000_100() -> U256 {
        d("250541448375047931186413801569")
    }
    fn price_10000_100() -> U256 {
        d("792281625142643375935439503360")
    }
    fn price_1_4() -> U256 {
        d("39614081257132168796771975168")
    }

    fn step(price: U256, target: U256, liquidity: U256, amount: I256, fee: u32) -> SwapStep {
        compute_swap_step(price, target, liquidity, amount, fee).unwrap()
    }

    fn neg(u: U256) -> I256 {
        -I256::try_from(u).unwrap()
    }

    /// The assembly version has to agree with the plain conditional v4-core tests it against.
    #[test]
    fn sqrt_price_target_picks_the_binding_bound() {
        let cases = [(1u64, 2u64), (2, 1), (7, 7), (0, 5), (5, 0)];
        for (next, limit) in cases {
            let (n, l) = (U256::from(next), U256::from(limit));
            for zero_for_one in [true, false] {
                let expected = if zero_for_one { n < l } else { n > l };
                assert_eq!(
                    get_sqrt_price_target(zero_for_one, n, l),
                    if expected { l } else { n },
                    "next={next} limit={limit} zeroForOne={zero_for_one}"
                );
            }
        }
    }

    #[test]
    fn exact_in_one_for_zero_capped_at_the_target() {
        let s = step(price_1_1(), price_101_100(), eth(2), neg(eth(1)), 600);
        assert_eq!(s.amount_in, U256::from(9975124224178055u64));
        assert_eq!(s.amount_out, U256::from(9925619580021728u64));
        assert_eq!(s.fee_amount, U256::from(5988667735148u64));
        assert!(s.amount_in + s.fee_amount < eth(1));
        assert_eq!(s.sqrt_price_next, price_101_100());

        let after_whole_input =
            get_next_sqrt_price_from_input(price_1_1(), eth(2), eth(1), false).unwrap();
        assert!(s.sqrt_price_next < after_whole_input);
    }

    #[test]
    fn exact_out_one_for_zero_capped_at_the_target() {
        let s = step(
            price_1_1(),
            price_101_100(),
            eth(2),
            I256::try_from(eth(1)).unwrap(),
            600,
        );
        assert_eq!(s.amount_in, U256::from(9975124224178055u64));
        assert_eq!(s.amount_out, U256::from(9925619580021728u64));
        assert_eq!(s.fee_amount, U256::from(5988667735148u64));
        assert!(s.amount_out < eth(1));
        assert_eq!(s.sqrt_price_next, price_101_100());

        let after_whole_output =
            get_next_sqrt_price_from_output(price_1_1(), eth(2), eth(1), false).unwrap();
        assert!(s.sqrt_price_next < after_whole_output);
    }

    #[test]
    fn exact_in_one_for_zero_fully_spent() {
        let s = step(price_1_1(), price_1000_100(), eth(2), neg(eth(1)), 600);
        assert_eq!(s.amount_in, U256::from(999400000000000000u64));
        assert_eq!(s.amount_out, U256::from(666399946655997866u64));
        assert_eq!(s.fee_amount, U256::from(600000000000000u64));
        assert_eq!(s.amount_in + s.fee_amount, eth(1));
        assert!(s.sqrt_price_next < price_1000_100());

        let after_input_less_fee =
            get_next_sqrt_price_from_input(price_1_1(), eth(2), eth(1) - s.fee_amount, false)
                .unwrap();
        assert_eq!(s.sqrt_price_next, after_input_less_fee);
    }

    #[test]
    fn exact_out_one_for_zero_fully_received() {
        let s = step(
            price_1_1(),
            price_10000_100(),
            eth(2),
            I256::try_from(eth(1)).unwrap(),
            600,
        );
        assert_eq!(s.amount_in, eth(2));
        assert_eq!(s.fee_amount, U256::from(1200720432259356u64));
        assert_eq!(s.amount_out, eth(1));
        assert!(s.sqrt_price_next < price_10000_100());

        let after_whole_output =
            get_next_sqrt_price_from_output(price_1_1(), eth(2), eth(1), false).unwrap();
        assert_eq!(s.sqrt_price_next, after_whole_output);
    }

    #[test]
    fn amount_out_is_capped_at_the_desired_amount_out() {
        let s = step(
            d("417332158212080721273783715441582"),
            d("1452870262520218020823638996"),
            d("159344665391607089467575320103"),
            I256::ONE,
            1,
        );
        assert_eq!(s.amount_in, U256::from(1));
        assert_eq!(s.fee_amount, U256::from(1));
        assert_eq!(s.amount_out, U256::from(1)); // would be 2 uncapped
        assert_eq!(s.sqrt_price_next, d("417332158212080721273783715441581"));
    }

    #[test]
    fn a_target_price_of_one_uses_only_part_of_the_input() {
        let specified = d("3915081100057732413702495386755767");
        let s = step(
            U256::from(2),
            U256::from(1),
            U256::from(1),
            neg(specified),
            1,
        );
        assert_eq!(s.amount_in, price_1_4());
        assert_eq!(s.fee_amount, d("39614120871253040049813"));
        assert!(s.amount_in + s.fee_amount <= specified);
        assert_eq!(s.amount_out, U256::ZERO);
        assert_eq!(s.sqrt_price_next, U256::from(1));
    }

    #[test]
    fn the_whole_input_is_not_taken_as_fee() {
        let s = step(
            U256::from(2413),
            U256::from(79887613182836312u64),
            d("1985041575832132834610021537970"),
            I256::try_from(-10).unwrap(),
            1872,
        );
        assert_eq!(s.amount_in, U256::from(9));
        assert_eq!(s.fee_amount, U256::from(1));
        assert_eq!(s.amount_out, U256::ZERO);
        assert_eq!(s.sqrt_price_next, U256::from(2413));
    }

    #[test]
    fn zero_for_one_with_intermediate_insufficient_liquidity_on_exact_output() {
        let p = d("20282409603651670423947251286016");
        let target = p * U256::from(11) / U256::from(10);
        let s = step(
            p,
            target,
            U256::from(1024),
            I256::try_from(4).unwrap(),
            3000,
        );
        assert_eq!(s.amount_out, U256::ZERO);
        assert_eq!(s.sqrt_price_next, target);
        assert_eq!(s.amount_in, U256::from(26215));
        assert_eq!(s.fee_amount, U256::from(79));
    }

    #[test]
    fn one_for_zero_with_intermediate_insufficient_liquidity_on_exact_output() {
        let p = d("20282409603651670423947251286016");
        let target = p * U256::from(9) / U256::from(10);
        let s = step(
            p,
            target,
            U256::from(1024),
            I256::try_from(263000).unwrap(),
            3000,
        );
        assert_eq!(s.amount_out, U256::from(26214));
        assert_eq!(s.sqrt_price_next, target);
        assert_eq!(s.amount_in, U256::from(1));
        assert_eq!(s.fee_amount, U256::from(1));
    }

    /// v4-core's `test_fuzz_computeSwapStep` invariants, over a deterministic sweep rather than
    /// forge's fuzzer: the step never overspends the amount specified, it consumes the amount
    /// exactly when it stops short of the target, and it never steps past the target.
    #[test]
    fn the_fuzz_invariants_hold_over_a_sweep() {
        let prices = [
            U256::from(1u64),
            U256::from(2413u64),
            price_1_4(),
            price_1_1(),
            price_101_100(),
            d("20282409603651670423947251286016"),
        ];
        let liquidities = [
            U256::ZERO,
            U256::from(1024u64),
            eth(2),
            d("159344665391607089467575320103"),
        ];
        let amounts = [-1i64, 1, -1_000_000, 1_000_000, i64::MIN + 1, i64::MAX];
        let fees = [0u32, 1, 600, 3000, 999_999];

        for &price in &prices {
            for &target in &prices {
                for &liquidity in &liquidities {
                    for &amount in &amounts {
                        for &fee in &fees {
                            let a = I256::try_from(amount).unwrap();
                            let Ok(s) = compute_swap_step(price, target, liquidity, a, fee) else {
                                continue; // v4-core reverts here too
                            };
                            let specified = a.unsigned_abs();

                            assert!(s.amount_in <= U256::MAX - s.fee_amount);
                            if amount >= 0 {
                                assert!(s.amount_out <= specified);
                            } else {
                                assert!(s.amount_in + s.fee_amount <= specified);
                            }

                            if price == target {
                                assert_eq!(s.amount_in, U256::ZERO);
                                assert_eq!(s.amount_out, U256::ZERO);
                                assert_eq!(s.fee_amount, U256::ZERO);
                                assert_eq!(s.sqrt_price_next, target);
                            }

                            if s.sqrt_price_next != target {
                                if amount > 0 {
                                    assert_eq!(s.amount_out, specified);
                                } else {
                                    assert_eq!(s.amount_in + s.fee_amount, specified);
                                }
                            }

                            if target <= price {
                                assert!(s.sqrt_price_next <= price && s.sqrt_price_next >= target);
                            } else {
                                assert!(s.sqrt_price_next >= price && s.sqrt_price_next <= target);
                            }
                        }
                    }
                }
            }
        }
    }
}
