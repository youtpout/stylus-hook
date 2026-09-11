//! Ticks to sqrt prices and back, ported from v4-core's `TickMath` (and the `BitMath` it uses).

use alloy_primitives::{uint, I256, U256};

/// The lowest tick, `log_1.0001(2^-128)`.
pub const MIN_TICK: i32 = -887272;
/// The highest tick, `log_1.0001(2^128)`.
pub const MAX_TICK: i32 = 887272;

/// `get_sqrt_price_at_tick(MIN_TICK)`.
pub const MIN_SQRT_PRICE: u64 = 4295128739;

/// `get_sqrt_price_at_tick(MAX_TICK)`, which a live price never actually reaches.
pub const MAX_SQRT_PRICE: U256 = uint!(1461446703485210103287273052203988822378723970342_U256);

/// Q128.128 one, the seed of the price product.
const ONE_Q128: U256 = uint!(0x100000000000000000000000000000000_U256);

/// v4-core's rescaling constant from `log_2` to `log_sqrt(1.0001)`, Q22.128.
const LOG_SCALE: I256 = I256::from_raw(uint!(255738958999603826347141_U256));

/// The ceiling and floor of the error in that approximation, v4-core's two magic numbers.
const ERROR_CEIL: I256 = I256::from_raw(uint!(3402992956809132418596140100660247210_U256));
const ERROR_FLOOR: I256 = I256::from_raw(uint!(291339464771989622907027621153398088495_U256));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickError {
    InvalidTick(i32),
    InvalidSqrtPrice(U256),
}

/// The largest tick that is a multiple of `tick_spacing`.
pub fn max_usable_tick(tick_spacing: i32) -> i32 {
    (MAX_TICK / tick_spacing) * tick_spacing
}

/// The smallest tick that is a multiple of `tick_spacing`.
pub fn min_usable_tick(tick_spacing: i32) -> i32 {
    (MIN_TICK / tick_spacing) * tick_spacing
}

/// `1/sqrt(1.0001)` in Q128.128 -- the factor for bit 0, which seeds `price` rather than
/// multiplying into it.
const FACTOR_0: U256 = uint!(0xfffcb933bd6fad37aa2d162d1a594001_U256);

/// The remaining 19 Q128.128 factors of `1/sqrt(1.0001^(2^i))`, one per higher bit of `|tick|`.
///
/// These have to be constants and not parsed strings: reading them from hex on every iteration cost
/// more than the whole multiplication chain, and turned a 2.2x win into a 3.6x loss.
const FACTORS: [U256; 19] = uint!([
    0xfff97272373d413259a46990580e213a_U256,
    0xfff2e50f5f656932ef12357cf3c7fdcc_U256,
    0xffe5caca7e10e4e61c3624eaa0941cd0_U256,
    0xffcb9843d60f6159c9db58835c926644_U256,
    0xff973b41fa98c081472e6896dfb254c0_U256,
    0xff2ea16466c96a3843ec78b326b52861_U256,
    0xfe5dee046a99a2a811c461f1969c3053_U256,
    0xfcbe86c7900a88aedcffc83b479aa3a4_U256,
    0xf987a7253ac413176f2b074cf7815e54_U256,
    0xf3392b0822b70005940c7a398e4b70f3_U256,
    0xe7159475a2c29b7443b29c7fa6e889d9_U256,
    0xd097f3bdfd2022b8845ad8f792aa5825_U256,
    0xa9f746462d870fdf8a65dc1f90e061e5_U256,
    0x70d869a156d2a1b890bb3df62baf32f7_U256,
    0x31be135f97d08fd981231505542fcfa6_U256,
    0x9aa508b5b7a84e1c677de54f3e99bc9_U256,
    0x5d6af8dedb81196699c329225ee604_U256,
    0x2216e584f5fa1ea926041bedfe98_U256,
    0x48a170391f7dc42444e8fa2_U256,
]);

/// `sqrt(1.0001^tick) * 2^96`.
///
/// The tick is decomposed into bits and one rounded Q128.128 factor is folded in per set bit, so
/// the whole price curve costs at most 19 multiplications and no exponentiation.
pub fn get_sqrt_price_at_tick(tick: i32) -> Result<U256, TickError> {
    let abs_tick = tick.unsigned_abs();
    if abs_tick > MAX_TICK as u32 {
        return Err(TickError::InvalidTick(tick));
    }

    let mut price = if abs_tick & 0x1 != 0 {
        FACTOR_0
    } else {
        ONE_Q128
    };
    for (i, factor) in FACTORS.iter().enumerate() {
        if abs_tick & (1 << (i + 1)) != 0 {
            // Both operands are below 2^128, so the product is exact and the wrap never happens.
            price = price.wrapping_mul(*factor) >> 128;
        }
    }

    // The factors are all below 1, so a positive tick is the reciprocal of a negative one.
    if tick > 0 {
        price = U256::MAX / price;
    }

    // Q128.128 to Q64.96, rounding up so that `get_tick_at_sqrt_price` of the result agrees.
    Ok((price + U256::from(u32::MAX)) >> 32)
}

/// The index of the highest set bit, as v4-core's `BitMath.mostSignificantBit`.
///
/// # Panics
///
/// If `x` is zero, matching the `require` there.
#[inline]
pub fn most_significant_bit(x: U256) -> u32 {
    assert!(!x.is_zero(), "BitMath: zero has no most significant bit");
    x.bit_len() as u32 - 1
}

/// The greatest tick whose price is at most `sqrt_price`.
///
/// This is a base-2 logarithm — 14 squarings extract 14 fractional bits — rescaled to base
/// `sqrt(1.0001)`. The approximation is good to within one tick, so the two candidates are
/// compared against the forward direction to pick the right one.
pub fn get_tick_at_sqrt_price(sqrt_price: U256) -> Result<i32, TickError> {
    if sqrt_price < U256::from(MIN_SQRT_PRICE) || sqrt_price >= MAX_SQRT_PRICE {
        return Err(TickError::InvalidSqrtPrice(sqrt_price));
    }

    let price = sqrt_price << 32;
    let msb = most_significant_bit(price);

    let mut r = if msb >= 128 {
        price >> (msb - 127)
    } else {
        price << (127 - msb)
    };

    let mut log_2 = (I256::try_from(msb).unwrap() - I256::try_from(128).unwrap()) << 64;
    for shift in (50u32..=63).rev() {
        // `r` is below 2^128 by construction, so squaring it is exact.
        r = r.wrapping_mul(r) >> 127;
        // One bit of the fraction: the square either crossed 2 or it did not.
        let f = usize::from(r.bit(128));
        log_2 |= I256::try_from(f).unwrap() << shift;
        r >>= f;
    }

    let log_sqrt10001: I256 = log_2 * LOG_SCALE;

    // The two magic numbers bound the error of the approximation above.
    let tick_low = to_i24((log_sqrt10001 - ERROR_CEIL).asr(128));
    let tick_hi = to_i24((log_sqrt10001 + ERROR_FLOOR).asr(128));

    Ok(if tick_low == tick_hi {
        tick_low
    } else if get_sqrt_price_at_tick(tick_hi)? <= sqrt_price {
        tick_hi
    } else {
        tick_low
    })
}

/// Solidity's `int24(...)` cast: keep the low 24 bits and sign-extend them.
fn to_i24(value: I256) -> i32 {
    let low = (value.into_raw() & U256::from(0xffffffu32)).as_limbs()[0] as i32;
    if low & 0x800000 != 0 {
        low - 0x1000000
    } else {
        low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> U256 {
        s.parse().unwrap()
    }

    #[test]
    fn the_tick_range_is_symmetric() {
        assert_eq!(MIN_TICK, -MAX_TICK);
        assert_eq!(MAX_TICK, -MIN_TICK);
    }

    #[test]
    fn sqrt_price_at_tick_rejects_ticks_out_of_range() {
        assert_eq!(
            get_sqrt_price_at_tick(MIN_TICK - 1),
            Err(TickError::InvalidTick(MIN_TICK - 1))
        );
        assert_eq!(
            get_sqrt_price_at_tick(MAX_TICK + 1),
            Err(TickError::InvalidTick(MAX_TICK + 1))
        );
        // v4-core tests i24::MIN specifically, because the `absTick` trick has to survive it.
        let i24_min = -0x800000;
        assert_eq!(
            get_sqrt_price_at_tick(i24_min),
            Err(TickError::InvalidTick(i24_min))
        );
    }

    #[test]
    fn sqrt_price_at_the_extreme_ticks() {
        assert_eq!(
            get_sqrt_price_at_tick(MIN_TICK),
            Ok(U256::from(MIN_SQRT_PRICE))
        );
        assert_eq!(
            get_sqrt_price_at_tick(MIN_TICK + 1),
            Ok(U256::from(4295343490u64))
        );
        assert_eq!(get_sqrt_price_at_tick(MAX_TICK), Ok(MAX_SQRT_PRICE));
        assert_eq!(
            get_sqrt_price_at_tick(MAX_TICK - 1),
            Ok(d("1461373636630004318706518188784493106690254656249"))
        );
    }

    /// The range is deliberately wider than the JS reference implementation's.
    #[test]
    fn the_range_exceeds_the_javascript_implementation() {
        assert!(get_sqrt_price_at_tick(MIN_TICK).unwrap() < U256::from(6085630636u64));
        assert!(
            get_sqrt_price_at_tick(MAX_TICK).unwrap()
                > d("1033437718471923706666374484006904511252097097914")
        );
    }

    #[test]
    fn tick_at_sqrt_price_rejects_prices_out_of_range() {
        let too_low = U256::from(MIN_SQRT_PRICE - 1);
        assert_eq!(
            get_tick_at_sqrt_price(too_low),
            Err(TickError::InvalidSqrtPrice(too_low))
        );
        assert_eq!(
            get_tick_at_sqrt_price(MAX_SQRT_PRICE),
            Err(TickError::InvalidSqrtPrice(MAX_SQRT_PRICE))
        );
        assert_eq!(
            get_tick_at_sqrt_price(U256::ZERO),
            Err(TickError::InvalidSqrtPrice(U256::ZERO))
        );
    }

    #[test]
    fn tick_at_the_extreme_prices() {
        assert_eq!(
            get_tick_at_sqrt_price(U256::from(MIN_SQRT_PRICE)),
            Ok(MIN_TICK)
        );
        assert_eq!(
            get_tick_at_sqrt_price(U256::from(4295343490u64)),
            Ok(MIN_TICK + 1)
        );
        assert_eq!(
            get_tick_at_sqrt_price(MAX_SQRT_PRICE - U256::from(1)),
            Ok(MAX_TICK - 1)
        );
        assert_eq!(
            get_tick_at_sqrt_price(d("1461373636630004318706518188784493106690254656249")),
            Ok(MAX_TICK - 1)
        );
    }

    /// v4-core's round-trip property: a tick's own price, the midpoint of its range, and one wei
    /// below the next tick must all map back to it, and the next tick's price to the next tick.
    ///
    /// Only valid up to `MAX_TICK - 2`. v4-core's own fuzz test bounds the tick at `MAX_TICK - 1`,
    /// where the last assertion asks for the tick of `MAX_SQRT_PRICE` — which both
    /// implementations reject by design. Forge has simply never guessed that one input.
    fn assert_round_trip(tick: i32) {
        let at = get_sqrt_price_at_tick(tick).unwrap();
        let next = get_sqrt_price_at_tick(tick + 1).unwrap();
        assert_eq!(
            get_tick_at_sqrt_price(at),
            Ok(tick),
            "lower price at {tick}"
        );
        assert_eq!(
            get_tick_at_sqrt_price((at + next) / U256::from(2)),
            Ok(tick),
            "mid price at {tick}"
        );
        assert_eq!(
            get_tick_at_sqrt_price(next - U256::from(1)),
            Ok(tick),
            "upper price at {tick}"
        );
        assert_eq!(
            get_tick_at_sqrt_price(next),
            Ok(tick + 1),
            "next tick at {tick}"
        );
    }

    #[test]
    fn the_round_trip_holds_across_the_range() {
        // A prime stride hits every residue class, and the boundaries matter most.
        let mut ticks = vec![MIN_TICK, MIN_TICK + 1, -1, 0, 1, MAX_TICK - 3, MAX_TICK - 2];
        let mut tick = MIN_TICK;
        while tick <= MAX_TICK - 2 {
            ticks.push(tick);
            tick += 997;
        }
        for tick in ticks {
            assert_round_trip(tick);
        }
    }

    /// The exhaustive version of the property above — every tick the property is defined at. Run it with
    /// `cargo test -p stylus-v4-math --release -- --ignored`.
    #[test]
    #[ignore = "exhaustive; minutes in debug"]
    fn the_round_trip_holds_at_every_tick() {
        for tick in MIN_TICK..=MAX_TICK - 2 {
            assert_round_trip(tick);
        }
    }

    #[test]
    fn usable_ticks_are_multiples_of_the_spacing() {
        assert_eq!(max_usable_tick(1), MAX_TICK);
        assert_eq!(min_usable_tick(1), MIN_TICK);
        assert_eq!(max_usable_tick(60), 887220);
        assert_eq!(min_usable_tick(60), -887220);
        for spacing in [1, 10, 60, 200, 32767] {
            assert_eq!(max_usable_tick(spacing) % spacing, 0);
            assert!(max_usable_tick(spacing) <= MAX_TICK);
            assert_eq!(min_usable_tick(spacing), -max_usable_tick(spacing));
        }
    }

    #[test]
    fn most_significant_bit_matches_bit_math() {
        assert_eq!(most_significant_bit(U256::from(1)), 0);
        assert_eq!(most_significant_bit(U256::from(2)), 1);
        assert_eq!(most_significant_bit(U256::from(3)), 1);
        assert_eq!(most_significant_bit(U256::MAX), 255);
        for i in 0..256u32 {
            let x = U256::from(1) << i;
            assert_eq!(most_significant_bit(x), i);
            if i > 0 {
                assert_eq!(most_significant_bit(x - U256::from(1)), i - 1);
            }
        }
    }

    #[test]
    #[should_panic(expected = "most significant bit")]
    fn most_significant_bit_rejects_zero() {
        most_significant_bit(U256::ZERO);
    }
}
