// SPDX-License-Identifier: MIT
//! Uniswap v4-core's swap math in Rust: [`full_math`], [`sqrt_price_math`], [`swap_math`] and
//! [`tick_math`], ported function for function and tested against v4-core's own vectors.
//!
//! Where v4-core fuzzes, this sweeps — the tick round trip is checked at all 1,774,543 ticks, which
//! found a failing input in v4-core's own `TickMath` test. See FEEDBACK.md.

#![cfg_attr(not(test), no_std)]

pub mod full_math;
pub mod sqrt_price_math;
pub mod swap_math;
pub mod tick_math;
pub mod unsafe_math;

use alloy_primitives::{uint, U256};

/// `FixedPoint96.RESOLUTION`.
pub const RESOLUTION: usize = 96;

/// `FixedPoint96.Q96`.
pub const Q96: U256 = uint!(0x1000000000000000000000000_U256);
