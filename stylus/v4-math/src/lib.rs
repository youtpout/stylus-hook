// SPDX-License-Identifier: MIT OR Apache-2.0
//! Uniswap v4-core's swap math, in Rust.
//!
//! The arithmetic a hook needs to price or replay a swap: [`full_math`] for `a * b / c` over a
//! 512-bit product, [`sqrt_price_math`] for moving a `sqrt(price)` and the token deltas that go with
//! it, [`swap_math`] for one step of a swap, and [`tick_math`] for ticks to prices and back.
//! Ported function for function from v4-core, and tested against v4-core's own vectors — every unit
//! test in `test/libraries/{FullMath,UnsafeMath,SqrtPriceMath,SwapMath,TickMath}.t.sol` appears
//! here with the same inputs and the same expected values.
//!
//! Where v4-core relies on forge's fuzzer, the port sweeps the domain instead: the tick round trip
//! is checked at all 1,774,543 ticks it is defined at, not sampled. That found one thing — v4-core's
//! `test_fuzz_getTickAtSqrtPrice_getSqrtPriceAtTick_relation` bounds its tick at `MAX_TICK - 1`, and
//! at exactly that value the test asks for the tick of `MAX_SQRT_PRICE`, which the library rejects
//! by design. The Solidity test fails there too; forge has never guessed the input.
//!
//! It exists because this is the arithmetic that clears the crossover: `mulDiv` runs 3.4× cheaper in
//! Rust and `sqrt` 5.6×, and replaying `Pool.swap` — which OpenZeppelin's `AntiSandwichHook` and
//! Uniswap's own `alf/SwapSimulator` both do — costs 21,460 gas of it per swap. See BENCHMARK.md.

#![cfg_attr(not(test), no_std)]

pub mod full_math;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick_math;
pub mod unsafe_math;

use alloy_primitives::U256;

/// `FixedPoint96.RESOLUTION`.
pub const RESOLUTION: usize = 96;

/// `FixedPoint96.Q96`.
pub fn q96() -> U256 {
    U256::from(1u8) << RESOLUTION
}
