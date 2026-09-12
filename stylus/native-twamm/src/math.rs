// SPDX-License-Identifier: MIT
//! The arithmetic of a TWAMM interval, in 1e18 fixed point, because Stylus forbids floating point
//! outright and `TwammHook.sol` carries the identical form — so the benchmark compares languages
//! rather than algorithms. Both agree with the quad-float original to two parts in 10^18.
//!
//! Free functions over `U256`/`I256`, no storage and no host access, so they test natively.

use alloy_primitives::{I256, U256};

/// 1e18.
pub const WAD_U64: u64 = 1_000_000_000_000_000_000;

/// `ln(2)`, scaled by 1e18.
const LN2_I64: i64 = 693_147_180_559_945_309;

/// The largest exponent argument the series is evaluated at.
///
/// Past this the closed form's `(e^p - c) / (e^p + c)` is one to well within a wei, so clamping
/// changes no representable answer — it only keeps `e^p` inside an `I256`.
const MAX_EXP_ARG: u64 = 88;

/// v4 prices are `sqrt(price) * 2^96`; this crate's are `sqrt(price) * 1e18`.
const Q96_SHIFT: usize = 96;

/// `TickMath.MIN_SQRT_PRICE`.
pub const MIN_SQRT_PRICE_X96: u64 = 4_295_128_739;

/// `TickMath.MAX_SQRT_PRICE`, which is wider than any primitive integer.
pub fn max_sqrt_price_x96() -> U256 {
    U256::from_limbs([
        6_743_328_256_752_651_557,
        17_280_870_778_742_802_505,
        4_294_805_859,
        0,
    ])
}

#[inline]
pub fn wad() -> U256 {
    U256::from(WAD_U64)
}

#[inline]
fn wad_i() -> I256 {
    I256::unchecked_from(WAD_U64)
}

#[inline]
fn ln2() -> I256 {
    I256::unchecked_from(LN2_I64)
}

/// `e^x` for `x >= 0`, in 1e18 fixed point.
///
/// Range-reduce to `e^x = 2^k · e^r` with `|r| <= ln2/2`, then a Taylor series for `e^r`, which
/// converges to well under a wei of WAD in twelve terms at that magnitude. Chosen over a minimax
/// polynomial because it can be written identically in both languages and checked term for term.
pub fn exp_wad(x: I256) -> I256 {
    let w = wad_i();
    let l = ln2();
    let limit = w * I256::unchecked_from(MAX_EXP_ARG);
    let x = if x > limit { limit } else { x };

    let k = (x + l / I256::unchecked_from(2)) / l;
    let r = x - k * l;

    let mut term = w;
    let mut sum = w;
    let mut i = 1i64;
    while i <= 12 {
        term = (term * r) / (w * I256::unchecked_from(i));
        sum += term;
        i += 1;
    }
    sum << k.as_i64() as usize
}

/// Square root of a 1e18 fixed-point number, in 1e18 fixed point.
///
/// `U256` has no `sqrt` on `wasm32`, so this is Newton's method seeded from the bit length — seven
/// iterations, which is one more than the worst case over the range this is called on.
pub fn sqrt_wad(x: U256) -> U256 {
    if x.is_zero() {
        return U256::ZERO;
    }
    let v = x * wad();
    let bits = 256 - v.leading_zeros();
    let mut z = U256::from(1u8) << bits.div_ceil(2);
    for _ in 0..7 {
        z = (z + v / z) >> 1;
    }
    let zz = v / z;
    if z <= zz {
        z
    } else {
        zz
    }
}

/// `sqrt(price) * 2^96` as v4 stores it, to `sqrt(price) * 1e18` as this crate uses it.
pub fn from_sqrt_x96(sqrt_price_x96: U256) -> U256 {
    (sqrt_price_x96 * wad()) >> Q96_SHIFT
}

/// The inverse of [`from_sqrt_x96`], clamped into the range v4 will accept as a swap limit.
pub fn to_sqrt_x96(sqrt_price: U256) -> U256 {
    let raw = (sqrt_price << Q96_SHIFT) / wad();
    let min = U256::from(MIN_SQRT_PRICE_X96);
    let max = max_sqrt_price_x96();
    if raw < min {
        min
    } else if raw > max {
        max
    } else {
        raw
    }
}

/// The price the pool reaches after `elapsed` seconds of both order pools selling into it, and what
/// each earned getting there.
///
/// `sqrt_price` and the rates are WAD-scaled, `liquidity` is raw as v4 stores it, and the earnings
/// come back raw. Reserves are `x = L/sqrt(P)` and `y = L*sqrt(P)`, liquidity constant across the span.
pub fn advance(
    sqrt_price: U256,
    liquidity: U256,
    rate0: U256,
    rate1: U256,
    elapsed: U256,
) -> (U256, U256, U256) {
    if liquidity.is_zero() || (rate0.is_zero() && rate1.is_zero()) {
        return (sqrt_price, U256::ZERO, U256::ZERO);
    }
    let w = wad();
    let sold0 = rate0 * elapsed / w;
    let sold1 = rate1 * elapsed / w;

    let x_before = liquidity * w / sqrt_price;
    let y_before = liquidity * sqrt_price / w;

    let next = if rate0.is_zero() {
        // Only token1 is being sold: the pool just absorbs it along the curve.
        sqrt_price + sold1 * w / liquidity
    } else if rate1.is_zero() {
        liquidity * w / (x_before + sold0)
    } else {
        // `two_sided` divides by a WAD-scaled liquidity; everything else here is in raw units.
        two_sided(sqrt_price, liquidity * w, rate0, rate1, elapsed)
    };

    let x_after = liquidity * w / next;
    let y_after = liquidity * next / w;

    // Neither of these can go negative in exact arithmetic — a pool cannot hand out more of a token
    // than was sold into it — but each is a difference of two rounded quotients, so the last wei is
    // clamped rather than trusted.
    let earned0 = (sold1 + y_before).saturating_sub(y_after);
    let earned1 = (sold0 + x_before).saturating_sub(x_after);
    (next, earned0, earned1)
}

/// The closed-form TWAMM price for a span where both pools are selling (Paradigm, 2021).
///
/// `liquidity` is WAD-scaled here, unlike in [`advance`], because that is what makes `pow`
/// dimensionless against WAD-scaled sell rates.
pub fn two_sided(
    sqrt_price: U256,
    liquidity: U256,
    rate0: U256,
    rate1: U256,
    elapsed: U256,
) -> U256 {
    let w = wad();
    // Both rates are WAD-scaled, so their product carries WAD twice and one comes back out.
    let sqrt_sell_rate = sqrt_wad(rate0 * rate1 / w);
    let sqrt_sell_ratio = sqrt_wad(rate1 * w / rate0);

    // `elapsed` is a plain count of seconds, so the WAD the quotient loses is put back.
    let pow = U256::from(2) * sqrt_sell_rate * elapsed * w / liquidity;

    let ratio = I256::from_raw(sqrt_sell_ratio);
    let price = I256::from_raw(sqrt_price);
    let c = (ratio - price) * wad_i() / (ratio + price);
    let e_pow = exp_wad(I256::from_raw(pow));

    (ratio * (e_pow - c) / (e_pow + c)).into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> U256 {
        U256::from_str_radix(s, 10).unwrap()
    }

    /// The values `uniswap/test/TwammHook.t.sol` asserts, which in turn track the quad-float
    /// original to two parts in 10^18.
    #[test]
    fn the_closed_form_matches_the_solidity_twin() {
        let e18 = wad();
        let l = U256::from(1_000_000u64) * e18;
        let mut price = e18;
        let expected = [
            "1001197841728773871",
            "1002331270278227884",
            "1003400248490110669",
        ];
        for (i, want) in expected.iter().enumerate() {
            let rate0 = U256::from(3) * e18 + U256::from(i) * (e18 / U256::from(10));
            price = two_sided(price, l, rate0, U256::from(5) * e18, U256::from(600));
            assert_eq!(price, d(want));
        }
    }

    #[test]
    fn a_one_sided_span_conserves_the_token_it_sells() {
        let e18 = wad();
        // The raw liquidity matching the WAD-scaled 1e24 the closed-form reference values use.
        let l = U256::from(1_000_000u64);
        // Only pool 0->1 is selling, so pool 1->0 earns nothing and pool 0->1 earns what the AMM
        // pays out for the token0 it received.
        let (next, earned0, earned1) =
            advance(e18, l, U256::from(3) * e18, U256::ZERO, U256::from(600));
        assert!(next < e18, "selling token0 must push the price down");
        assert_eq!(earned1, U256::ZERO);
        // 3 token0/s for 600s is 1800 token0 in, and near a price of 1 that buys just under 1800.
        assert!(earned0 > U256::from(1790) && earned0 < U256::from(1800));
    }

    #[test]
    fn both_pools_selling_leaves_each_of_them_paid() {
        let e18 = wad();
        let l = U256::from(1_000_000u64);
        let (next, earned0, earned1) = advance(
            e18,
            l,
            U256::from(3) * e18,
            U256::from(5) * e18,
            U256::from(600),
        );
        assert_eq!(next, d("1001197841728773871"));
        // 1800 token0 sold in, 3000 token1 sold in, at a price near 1.
        assert!(earned0 > U256::from(1800) && earned0 < U256::from(1810));
        assert!(earned1 > U256::from(2990) && earned1 < U256::from(3000));
    }

    #[test]
    fn the_price_conversion_round_trips() {
        let e18 = wad();
        let x96 = to_sqrt_x96(e18);
        assert_eq!(x96, U256::from(1u8) << 96);
        assert_eq!(from_sqrt_x96(x96), e18);
    }

    #[test]
    fn a_huge_exponent_saturates_instead_of_overflowing() {
        let huge = I256::unchecked_from(1_000i64) * wad_i();
        assert!(exp_wad(huge) > I256::ZERO);
    }
}
