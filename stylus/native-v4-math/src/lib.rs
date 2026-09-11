// SPDX-License-Identifier: MIT OR Apache-2.0
//! v4-core's swap math as a Stylus contract, to be priced against the Solidity it was ported from.
//!
//! The arithmetic lives in [`stylus_v4_math`], which is tested against v4-core's own vectors. This
//! crate is only the ABI around it, shaped to match `uniswap/src/V4MathBench.sol` call for call —
//! and that twin calls the real `SwapMath`, `TickMath` and `FullMath`, not a reimplementation. So
//! the control side of this benchmark is Uniswap's production code.
//!
//! [`V4Math::walk_swap`] is the number that matters. It is `Pool.swap`'s loop without the storage:
//! find the next initialised tick, price the step up to it, cross, repeat. Replaying that loop is
//! what OpenZeppelin's `AntiSandwichHook` and Uniswap's own `alf/SwapSimulator` do on every swap,
//! and it is pure arithmetic — which is the one shape of hook where Stylus can win.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{
    aliases::{I24, U128, U160, U24},
    I256, U256,
};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{prelude::*, storage::StorageU8};

use stylus_v4_math::{
    full_math, sqrt_price_math,
    sqrt_price_math::PriceError,
    swap_math::{self, SwapStep},
    tick_math::{self, TickError},
};

sol! {
    /// A tick outside `[MIN_TICK, MAX_TICK]`.
    #[derive(Debug)]
    error InvalidTick(int24 tick);
    /// A price outside `[MIN_SQRT_PRICE, MAX_SQRT_PRICE)`.
    #[derive(Debug)]
    error InvalidSqrtPrice(uint160 sqrtPriceX96);
    /// A price or liquidity of zero, or an amount the liquidity cannot pay.
    #[derive(Debug)]
    error PriceMath(uint8 kind);
}

fn tick_err(e: TickError) -> Vec<u8> {
    match e {
        TickError::InvalidTick(t) => InvalidTick {
            tick: I24::unchecked_from(t),
        }
        .abi_encode(),
        TickError::InvalidSqrtPrice(p) => InvalidSqrtPrice {
            sqrtPriceX96: U160::from(p),
        }
        .abi_encode(),
    }
}

fn price_err(e: PriceError) -> Vec<u8> {
    let kind = match e {
        PriceError::InvalidPriceOrLiquidity => 0,
        PriceError::InvalidPrice => 1,
        PriceError::NotEnoughLiquidity => 2,
        PriceError::PriceOverflow => 3,
    };
    PriceMath { kind }.abi_encode()
}

#[storage]
#[entrypoint]
pub struct V4Math {
    _pad: StorageU8,
}

#[public]
impl V4Math {
    /// A no-op, to subtract the cost of being called at all.
    pub fn baseline(&self, _n: U256) -> U256 {
        U256::ZERO
    }

    // --- the primitives, priced one at a time ---------------------------------------------------

    pub fn mul_div(&self, a: U256, b: U256, denominator: U256) -> U256 {
        full_math::mul_div(a, b, denominator)
    }

    pub fn mul_div_rounding_up(&self, a: U256, b: U256, denominator: U256) -> U256 {
        full_math::mul_div_rounding_up(a, b, denominator)
    }

    pub fn get_sqrt_price_at_tick(&self, tick: I24) -> Result<U160, Vec<u8>> {
        tick_math::get_sqrt_price_at_tick(tick.as_i32())
            .map(U160::from)
            .map_err(tick_err)
    }

    pub fn get_tick_at_sqrt_price(&self, sqrt_price_x96: U160) -> Result<I24, Vec<u8>> {
        tick_math::get_tick_at_sqrt_price(U256::from(sqrt_price_x96))
            .map(I24::unchecked_from)
            .map_err(tick_err)
    }

    pub fn get_amount0_delta(
        &self,
        sqrt_price_a_x96: U160,
        sqrt_price_b_x96: U160,
        liquidity: U128,
        round_up: bool,
    ) -> Result<U256, Vec<u8>> {
        sqrt_price_math::get_amount0_delta(
            U256::from(sqrt_price_a_x96),
            U256::from(sqrt_price_b_x96),
            U256::from(liquidity),
            round_up,
        )
        .map_err(price_err)
    }

    pub fn get_amount1_delta(
        &self,
        sqrt_price_a_x96: U160,
        sqrt_price_b_x96: U160,
        liquidity: U128,
        round_up: bool,
    ) -> U256 {
        sqrt_price_math::get_amount1_delta(
            U256::from(sqrt_price_a_x96),
            U256::from(sqrt_price_b_x96),
            U256::from(liquidity),
            round_up,
        )
    }

    /// One step of a swap: the price reached, the amounts in and out, and the fee.
    pub fn compute_swap_step(
        &self,
        sqrt_price_current_x96: U160,
        sqrt_price_target_x96: U160,
        liquidity: U128,
        amount_remaining: I256,
        fee_pips: U24,
    ) -> Result<(U160, U256, U256, U256), Vec<u8>> {
        let step = swap_math::compute_swap_step(
            U256::from(sqrt_price_current_x96),
            U256::from(sqrt_price_target_x96),
            U256::from(liquidity),
            amount_remaining,
            fee_pips.to::<u32>(),
        )
        .map_err(price_err)?;
        Ok((
            U160::from(step.sqrt_price_next),
            step.amount_in,
            step.amount_out,
            step.fee_amount,
        ))
    }

    // --- the workload --------------------------------------------------------------------------

    /// `Pool.swap`'s loop, without the storage: price a step up to the next initialised tick, cross
    /// it, repeat until the amount runs out or the limit is reached.
    ///
    /// Every multiple of `tick_spacing` is taken as initialised, which is the dense case and so the
    /// honest upper bound on how many steps a swap takes. Returns the price reached and the three
    /// totals, so neither side can have the loop optimised away.
    #[allow(clippy::too_many_arguments)]
    pub fn walk_swap(
        &self,
        start_sqrt_price_x96: U160,
        sqrt_price_limit_x96: U160,
        tick_spacing: I24,
        liquidity: U128,
        amount_specified: I256,
        fee_pips: U24,
        max_steps: U256,
    ) -> Result<(U160, U256, U256, U256), Vec<u8>> {
        let limit = U256::from(sqrt_price_limit_x96);
        let spacing = tick_spacing.as_i32();
        let l = U256::from(liquidity);
        let fee = fee_pips.to::<u32>();

        let mut sqrt_price = U256::from(start_sqrt_price_x96);
        let mut tick = tick_math::get_tick_at_sqrt_price(sqrt_price).map_err(tick_err)?;
        let zero_for_one = limit < sqrt_price;

        let mut remaining = amount_specified;
        let (mut total_in, mut total_out, mut total_fee) = (U256::ZERO, U256::ZERO, U256::ZERO);

        let mut steps = U256::ZERO;
        while !remaining.is_zero() && sqrt_price != limit && steps < max_steps {
            // The next initialised tick in the direction of travel.
            let next_tick = if zero_for_one {
                (tick.div_euclid(spacing) - 1) * spacing
            } else {
                (tick.div_euclid(spacing) + 1) * spacing
            };
            if !(tick_math::MIN_TICK..=tick_math::MAX_TICK).contains(&next_tick) {
                break;
            }
            let next_price = tick_math::get_sqrt_price_at_tick(next_tick).map_err(tick_err)?;
            let target = swap_math::get_sqrt_price_target(zero_for_one, next_price, limit);

            let SwapStep {
                sqrt_price_next,
                amount_in,
                amount_out,
                fee_amount,
            } = swap_math::compute_swap_step(sqrt_price, target, l, remaining, fee)
                .map_err(price_err)?;

            if remaining.is_negative() {
                remaining += I256::try_from(amount_in + fee_amount).unwrap_or(I256::ZERO);
            } else {
                remaining -= I256::try_from(amount_out).unwrap_or(I256::ZERO);
            }
            total_in += amount_in;
            total_out += amount_out;
            total_fee += fee_amount;

            // Exactly v4's branch: a step that reached the tick crosses it, one that stopped short
            // has to be located.
            tick = if sqrt_price_next == next_price {
                next_tick
            } else if sqrt_price_next != sqrt_price {
                tick_math::get_tick_at_sqrt_price(sqrt_price_next).map_err(tick_err)?
            } else {
                tick
            };
            sqrt_price = sqrt_price_next;
            steps += U256::from(1);
        }

        Ok((U160::from(sqrt_price), total_in, total_out, total_fee))
    }

    /// Split an exact-input swap across `prices.len()` pools, by greedy marginal allocation.
    ///
    /// This is what a router does off-chain today, and the reason it is done off-chain is that
    /// evaluating the alternatives costs more than the swap. Each round walks every pool once to
    /// price one more chunk, and gives the chunk to whichever pool returns the most for it -- so the
    /// cost is `rounds x pools` swap replays, which is the quantity this measures.
    ///
    /// Returns the total output and the amount allocated to each pool.
    #[allow(clippy::too_many_arguments)]
    pub fn route_exact_in(
        &self,
        start_prices: Vec<U160>,
        liquidities: Vec<U128>,
        tick_spacing: I24,
        amount_in: U256,
        fee_pips: U24,
        rounds: U256,
        max_steps: U256,
    ) -> Result<(U256, Vec<U256>), Vec<u8>> {
        let n = start_prices.len();
        if n == 0 || liquidities.len() != n || rounds.is_zero() {
            return Ok((U256::ZERO, Vec::new()));
        }

        let chunk = amount_in / rounds;
        let mut allocated = alloc::vec![U256::ZERO; n];
        let mut current_out = alloc::vec![U256::ZERO; n];

        let mut r = U256::ZERO;
        while r < rounds {
            let (mut best_pool, mut best_gain, mut best_out) = (0usize, U256::ZERO, U256::ZERO);
            for (p, allocation) in allocated.iter().enumerate() {
                // Exact input is negative in v4's convention, and the limit is the far end of the
                // range so only the amount binds.
                let amount = -I256::try_from(*allocation + chunk).unwrap_or(I256::ZERO);
                let (_, _, out, _) = self.walk_swap(
                    start_prices[p],
                    U160::from(tick_math::MIN_SQRT_PRICE + 1),
                    tick_spacing,
                    liquidities[p],
                    amount,
                    fee_pips,
                    max_steps,
                )?;
                let gain = out.saturating_sub(current_out[p]);
                if gain > best_gain {
                    (best_pool, best_gain, best_out) = (p, gain, out);
                }
            }
            allocated[best_pool] += chunk;
            current_out[best_pool] = best_out;
            r += U256::from(1);
        }

        Ok((current_out.iter().copied().sum(), allocated))
    }

    // --- dials, to attribute the result to its parts --------------------------------------------

    /// `n` swap steps at a fixed price and target, so the loop overhead is the same on both sides.
    pub fn swap_step_loop(
        &self,
        n: U256,
        sqrt_price_current_x96: U160,
        sqrt_price_target_x96: U160,
        liquidity: U128,
        amount_remaining: I256,
        fee_pips: U24,
    ) -> Result<U256, Vec<u8>> {
        let (p, t, l, fee) = (
            U256::from(sqrt_price_current_x96),
            U256::from(sqrt_price_target_x96),
            U256::from(liquidity),
            fee_pips.to::<u32>(),
        );
        let mut acc = U256::ZERO;
        let mut i = U256::ZERO;
        while i < n {
            let step =
                swap_math::compute_swap_step(p, t, l, amount_remaining, fee).map_err(price_err)?;
            acc = acc
                .wrapping_add(step.amount_in)
                .wrapping_add(step.amount_out);
            i += U256::from(1);
        }
        Ok(acc)
    }

    /// `n` prices from ticks, walking forward so no two calls share an answer.
    pub fn sqrt_price_at_tick_loop(&self, n: U256, start_tick: I24) -> Result<U256, Vec<u8>> {
        let mut acc = U256::ZERO;
        let mut tick = start_tick.as_i32();
        let mut i = U256::ZERO;
        while i < n {
            acc = acc.wrapping_add(tick_math::get_sqrt_price_at_tick(tick).map_err(tick_err)?);
            tick += 1;
            i += U256::from(1);
        }
        Ok(acc)
    }

    /// `n` ticks from prices, walking the price up so no two calls share an answer.
    pub fn tick_at_sqrt_price_loop(&self, n: U256, start_price_x96: U160) -> Result<I256, Vec<u8>> {
        let mut acc = I256::ZERO;
        let mut price = U256::from(start_price_x96);
        let mut i = U256::ZERO;
        while i < n {
            acc += I256::try_from(tick_math::get_tick_at_sqrt_price(price).map_err(tick_err)?)
                .unwrap_or(I256::ZERO);
            price += U256::from(1_000_000_000_000_000u64);
            i += U256::from(1);
        }
        Ok(acc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    fn d(s: &str) -> U256 {
        s.parse().unwrap()
    }

    const P11: &str = "79228162514264337593543950336";
    const MIN_P: u64 = 4295128739;

    fn walk(c: &V4Math, amount: i64, max_steps: u64) -> (U160, U256, U256, U256) {
        c.walk_swap(
            U160::from(d(P11)),
            U160::from(MIN_P + 1),
            I24::unchecked_from(60),
            U128::from(1_000_000_000_000_000_000u64),
            I256::try_from(amount).unwrap() * I256::try_from(1_000_000_000_000_000_000u64).unwrap(),
            U24::from(3000),
            U256::from(max_steps),
        )
        .unwrap()
    }

    /// The same table `uniswap/test/V4MathBench.t.sol` asserts, computed there by v4-core's own
    /// libraries. If these two drift apart, `bench-v4-math.bash` stops being a language comparison
    /// and one of the suites says so first.
    #[test]
    fn the_walk_matches_the_solidity_twin() {
        let vm = TestVM::default();
        let c = V4Math::from(&vm);
        for amount in [-1000i64, 1000] {
            let (price, amount_in, amount_out, fee) = walk(&c, amount, 8);
            assert_eq!(U256::from(price), d("77349415686109193705893883426"));
            assert_eq!(amount_in, U256::from(24289088824914538u64));
            assert_eq!(amount_out, U256::from(23713118776633140u64));
            assert_eq!(fee, U256::from(73086526052907u64));
        }
    }

    #[test]
    fn the_walk_stops_when_the_amount_is_spent() {
        let vm = TestVM::default();
        let c = V4Math::from(&vm);
        let milli = I256::try_from(1_000_000_000_000_000u64).unwrap();

        let (price, amount_in, _, fee) = c
            .walk_swap(
                U160::from(d(P11)),
                U160::from(MIN_P + 1),
                I24::unchecked_from(60),
                U128::from(1_000_000_000_000_000_000u64),
                -milli,
                U24::from(3000),
                U256::from(64),
            )
            .unwrap();
        assert_eq!(amount_in + fee, U256::from(1_000_000_000_000_000u64));
        assert_eq!(U256::from(price), d("79149250711305166342700278159"));

        let (out_price, _, amount_out, _) = c
            .walk_swap(
                U160::from(d(P11)),
                U160::from(MIN_P + 1),
                I24::unchecked_from(60),
                U128::from(1_000_000_000_000_000_000u64),
                milli,
                U24::from(3000),
                U256::from(64),
            )
            .unwrap();
        assert_eq!(amount_out, U256::from(1_000_000_000_000_000u64));
        assert!(out_price < price);
    }

    /// The same split `uniswap/test/V4MathBench.t.sol` asserts, computed there by v4-core's own
    /// libraries. The greedy allocation is deterministic, so any disagreement between the two
    /// routers shows up as a different split rather than as a rounding difference.
    #[test]
    fn the_route_matches_the_solidity_twin() {
        let vm = TestVM::default();
        let c = V4Math::from(&vm);
        let eth = |n: u64| U256::from(n) * U256::from(1_000_000_000_000_000_000u64);

        let prices = alloc::vec![
            U160::from(d("79228162514264337593543950336")),
            U160::from(d("79623317895830914510639640423")),
            U160::from(d("78831026366734652303669917531")),
        ];
        let liquidities = alloc::vec![
            U128::from(1_000u64) * U128::from(1_000_000_000_000_000_000u64),
            U128::from(2_000u64) * U128::from(1_000_000_000_000_000_000u64),
            U128::from(500u64) * U128::from(1_000_000_000_000_000_000u64),
        ];

        let (total, split) = c
            .route_exact_in(
                prices,
                liquidities,
                I24::unchecked_from(60),
                eth(100),
                U24::from(3000),
                U256::from(8),
                U256::from(8),
            )
            .unwrap();

        assert_eq!(total, U256::from(87287557262511096767u128));
        assert_eq!(
            split,
            alloc::vec![
                eth(25),
                eth(62) + eth(1) / U256::from(2),
                eth(12) + eth(1) / U256::from(2)
            ]
        );
    }

    /// The step count has to be the only thing that changes, or the benchmark's dial is not a dial.
    #[test]
    fn each_step_consumes_more_input() {
        let vm = TestVM::default();
        let c = V4Math::from(&vm);
        let mut previous = U256::ZERO;
        for n in 1..=16 {
            let (_, amount_in, _, _) = walk(&c, -1000, n);
            assert!(amount_in > previous, "step {n} consumed nothing new");
            previous = amount_in;
        }
    }
}
