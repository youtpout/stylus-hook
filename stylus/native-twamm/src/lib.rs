// SPDX-License-Identifier: MIT OR Apache-2.0
//! A TWAMM interval, in pure Rust, as a Uniswap v4 hook.
//!
//! Uniswap shipped a TWAMM hook as a v4-periphery example and recorded what it costs: 489,927 gas
//! for one interval and about 100,000 for each one after. That is the only workload in this
//! repository's benchmarks above the crossover where a Stylus port pays for itself.
//!
//! Their cost comes from `ABDKMathQuad`, which emulates IEEE 754 binary128 in software because the
//! EVM has no floating point. Stylus has none either — a contract that so much as converts an
//! integer to an `f64` is refused at activation — so this implementation works in 1e18 fixed point,
//! and `uniswap/src/TwammHook.sol` carries the identical fixed-point form so the benchmark compares
//! two languages rather than two algorithms. Both agree with the quad-float original to two parts
//! in 10^18.
//!
//! The closed form is the published TWAMM solution (Paradigm, 2021), implemented from the formula:
//!
//! ```text
//! sqrtSellRate  = sqrt(rate0 * rate1)
//! sqrtSellRatio = sqrt(rate1 / rate0)
//! pow           = 2 * sqrtSellRate * elapsed / liquidity
//! c             = (sqrtSellRatio - sqrtPrice) / (sqrtSellRatio + sqrtPrice)
//! newSqrtPrice  = sqrtSellRatio * (e^pow - c) / (e^pow + c)
//! ```

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, I256, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{
    abi::Bytes,
    prelude::*,
    storage::{StorageU256, StorageU8},
};
use stylus_uniswap_v4::{
    hooks::{selector, HookConfig, HookGuards, IHooks},
    types::{BeforeSwapDelta, PoolKey, SwapParams, U24},
    Permissions, ZERO_DELTA,
};

include!(concat!(env!("OUT_DIR"), "/pool_manager.rs"));

sol! {
    /// The address baked in at build time is not the one this deployment was given.
    #[derive(Debug)]
    error PoolManagerMismatch(address baked, address given);
}

#[storage]
#[entrypoint]
pub struct TwammHook {
    /// How many intervals a swap advances through. The dial the benchmark sweeps.
    intervals: StorageU256,
    last_sqrt_price: StorageU256,
    _pad: StorageU8,
}

impl HookConfig for TwammHook {
    fn pool_manager(&self) -> Address {
        POOL_MANAGER
    }

    fn permissions(&self) -> Permissions {
        Permissions::none().with_before_swap()
    }
}

/// 1e18.
fn wad() -> I256 {
    I256::try_from(1_000_000_000_000_000_000i64).unwrap()
}

/// ln(2), scaled by 1e18.
fn ln2() -> I256 {
    I256::try_from(693_147_180_559_945_309i64).unwrap()
}

#[public]
#[implements(IHooks)]
impl TwammHook {
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        if pool_manager != POOL_MANAGER {
            return Err(PoolManagerMismatch {
                baked: POOL_MANAGER,
                given: pool_manager,
            }
            .abi_encode());
        }
        self.intervals.set(U256::from(1));
        HookGuards::validate_hook_address(self)
    }

    pub fn pool_manager(&self) -> Address {
        POOL_MANAGER
    }

    pub fn intervals(&self) -> U256 {
        self.intervals.get()
    }

    pub fn set_intervals(&mut self, n: U256) {
        self.intervals.set(n);
    }

    pub fn last_sqrt_price(&self) -> U256 {
        self.last_sqrt_price.get()
    }

    /// `e^x` for `x >= 0`, in 1e18 fixed point.
    ///
    /// Range-reduce to `e^x = 2^k · e^r` with `|r| <= ln2/2`, then a Taylor series for `e^r`, which
    /// converges to well under a wei of WAD in twelve terms at that magnitude.
    pub fn exp_wad(&self, x: I256) -> I256 {
        let w = wad();
        let l = ln2();
        let k = (x + l / I256::try_from(2i64).unwrap()) / l;
        let r = x - k * l;

        let mut term = w;
        let mut sum = w;
        let mut i = 1i64;
        while i <= 12 {
            term = (term * r) / (w * I256::try_from(i).unwrap());
            sum += term;
            i += 1;
        }
        sum << k.as_i64() as usize
    }

    /// Square root of a 1e18 fixed-point number, in 1e18 fixed point.
    pub fn sqrt_wad(&self, x: U256) -> U256 {
        if x.is_zero() {
            return U256::ZERO;
        }
        let v = x * U256::from(1_000_000_000_000_000_000u64);
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

    /// One TWAMM interval: the price the pool arrives at after `elapsed` seconds of two order pools
    /// selling into each other against `liquidity`.
    pub fn new_sqrt_price_fixed(
        &self,
        sqrt_price_e18: U256,
        liquidity: U256,
        rate0: U256,
        rate1: U256,
        elapsed: U256,
    ) -> U256 {
        let w = U256::from(1_000_000_000_000_000_000u64);
        // Both rates are WAD-scaled, so their product carries WAD twice and one comes back out.
        let sqrt_sell_rate = self.sqrt_wad(rate0 * rate1 / w);
        let sqrt_sell_ratio = self.sqrt_wad(rate1 * w / rate0);

        // `elapsed` is a plain count of seconds, so the WAD the quotient loses is put back.
        let pow = U256::from(2) * sqrt_sell_rate * elapsed * w / liquidity;

        let ratio = I256::from_raw(sqrt_sell_ratio);
        let price = I256::from_raw(sqrt_price_e18);
        let c = (ratio - price) * wad() / (ratio + price);
        let e_pow = self.exp_wad(I256::from_raw(pow));

        (ratio * (e_pow - c) / (e_pow + c)).into_raw()
    }

    /// `n` intervals, each feeding its price into the next. Must agree with `TwammHook.sol`.
    pub fn work_fixed(&self, n: U256) -> U256 {
        let e18 = U256::from(1_000_000_000_000_000_000u64);
        let mut price = e18;
        let mut i = U256::ZERO;
        while i < n {
            price = self.new_sqrt_price_fixed(
                price,
                U256::from(1_000_000u64) * e18,
                U256::from(3) * e18 + i * (e18 / U256::from(10)),
                U256::from(5) * e18,
                U256::from(600),
            );
            i += U256::from(1);
        }
        price
    }
}

#[public]
impl IHooks for TwammHook {
    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        let n = self.intervals.get();
        let price = self.work_fixed(n);
        self.last_sqrt_price.set(price);
        Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    /// beforeSwap only.
    const HOOK: Address = Address::new([
        0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe,
        0xef, 0x00, 0x00, 0x00, 0x80,
    ]);

    fn deployed(vm: &TestVM) -> TwammHook {
        vm.set_contract_address(HOOK);
        let mut c = TwammHook::from(vm);
        c.constructor(POOL_MANAGER).unwrap();
        c
    }

    /// The same values `uniswap/test/TwammHook.t.sol` asserts, which in turn track the quad-float
    /// original to two parts in 10^18.
    #[test]
    fn interval_matches_the_solidity_twin() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        let d = |s: &str| U256::from_str_radix(s, 10).unwrap();

        assert_eq!(c.work_fixed(U256::ZERO), d("1000000000000000000"));
        assert_eq!(c.work_fixed(U256::from(1)), d("1001197841728773871"));
        assert_eq!(c.work_fixed(U256::from(2)), d("1002331270278227884"));
        assert_eq!(c.work_fixed(U256::from(3)), d("1003400248490110669"));
    }

    #[test]
    fn the_price_walks_towards_the_sell_ratio() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        let one = c.work_fixed(U256::from(1));
        let three = c.work_fixed(U256::from(3));
        assert!(one > U256::from(1_000_000_000_000_000_000u64));
        assert!(three > one);
    }
}
