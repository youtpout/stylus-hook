// SPDX-License-Identifier: MIT OR Apache-2.0
//! A Uniswap v4 hook that prices swaps on a StableSwap curve — written entirely in Rust.
//!
//! Every other hook measured in this repository turned out to be dominated by storage, and porting
//! one to Stylus was worth a few percent at best. This one is built the other way round: it is the
//! kind of hook the benchmarks say Stylus is *for*. Pricing a swap means solving the StableSwap
//! invariant `D` and then the output reserve `y`, both by Newton's method — about twelve iterations
//! of full-range 256-bit arithmetic per swap, which is the regime where Rust runs 2.8× to 4.5×
//! cheaper than Solidity.
//!
//! `uniswap/src/StableSwapHook.sol` is the identical curve in Solidity, and
//! `bench-stableswap.bash` swaps through both.
//!
//! The invariant is implemented from Curve's published StableSwap formula. No implementation was
//! copied.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::{
    abi::Bytes,
    prelude::*,
    storage::{StorageAddress, StorageU256},
};
use stylus_uniswap_v4::{
    hooks::{selector, HookConfig, HookGuards, IHooks},
    types::{BeforeSwapDelta, PoolKey, SwapParams, U24},
    Permissions, ZERO_DELTA,
};

/// Amplification coefficient times n^n, for n = 2. Higher means a flatter curve.
const DEFAULT_ANN: u64 = 100;

#[storage]
#[entrypoint]
pub struct StableSwapHook {
    pool_manager: StorageAddress,
    reserve0: StorageU256,
    reserve1: StorageU256,
    ann: StorageU256,
    last_quote: StorageU256,
}

impl HookConfig for StableSwapHook {
    fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    fn permissions(&self) -> Permissions {
        Permissions::none().with_before_swap()
    }
}

#[public]
#[implements(IHooks)]
impl StableSwapHook {
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        self.pool_manager.set(pool_manager);
        self.ann.set(U256::from(DEFAULT_ANN));
        self.reserve0
            .set(U256::from(1000u64) * U256::from(10u64).pow(U256::from(18)));
        self.reserve1
            .set(U256::from(600u64) * U256::from(10u64).pow(U256::from(18)));
        HookGuards::validate_hook_address(self)
    }

    pub fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    pub fn reserves(&self) -> (U256, U256) {
        (self.reserve0.get(), self.reserve1.get())
    }

    pub fn last_quote(&self) -> U256 {
        self.last_quote.get()
    }

    /// The StableSwap invariant for two assets, solved by Newton's method.
    ///
    /// With imbalanced reserves this takes about four iterations, each one a chain of full-range
    /// multiplications and divisions.
    pub fn get_d(&self, x0: U256, x1: U256, ann: U256) -> U256 {
        let one = U256::from(1);
        let two = U256::from(2);
        let three = U256::from(3);
        let s = x0 + x1;
        if s.is_zero() {
            return U256::ZERO;
        }
        let mut d = s;
        for _ in 0..255u16 {
            let dp = ((d * d) / (x0 * two)) * d / (x1 * two);
            let prev = d;
            d = (ann * s + two * dp) * d / ((ann - one) * d + three * dp);
            let gap = if d > prev { d - prev } else { prev - d };
            if gap <= one {
                break;
            }
        }
        d
    }

    /// The reserve of the output token once `amount_in` has been added to `x0`.
    /// About eight more Newton iterations on top of [`StableSwapHook::get_d`].
    pub fn get_y(&self, amount_in: U256, x0: U256, x1: U256, ann: U256) -> U256 {
        let one = U256::from(1);
        let two = U256::from(2);
        let d = self.get_d(x0, x1, ann);
        let x = x0 + amount_in;
        let c = ((d * d) / (x * two)) * d / (ann * two);
        let b = x + d / ann;
        let mut y = d;
        for _ in 0..255u16 {
            let prev = y;
            y = (y * y + c) / (two * y + b - d);
            let gap = if y > prev { y - prev } else { prev - y };
            if gap <= one {
                break;
            }
        }
        y
    }

    /// `n` chained `a * b / c` on values that never overflow 256 bits.
    ///
    /// The isolation probe for the StableSwap result. In the EVM a multiply is one `MUL` and a
    /// divide is one `DIV`, five gas each. In WASM there is no 256-bit word, so this is limb
    /// arithmetic either way. Nothing else is measured here.
    pub fn plain_mul_div(&self, n: U256) -> U256 {
        let one = U256::from(1);
        let z = one << 100;
        let y = (one << 100) - one;
        let mut a = (one << 100) + one;
        let mut i = U256::ZERO;
        while i < n {
            a = (a * y) / z + one;
            i += one;
        }
        a
    }

    /// Prices `amount_in` against the current reserves without touching them.
    pub fn quote(&self, amount_in: U256) -> U256 {
        let (x0, x1) = (self.reserve0.get(), self.reserve1.get());
        let y = self.get_y(amount_in, x0, x1, self.ann.get());
        if x1 > y {
            x1 - y
        } else {
            U256::ZERO
        }
    }
}

#[public]
impl IHooks for StableSwapHook {
    /// Prices the swap on the StableSwap curve and moves the reserves along it.
    ///
    /// The delta returned is zero: this hook measures what the curve costs, it does not take custody
    /// of the swap. A production version would return the priced delta here and balance its books
    /// with `PoolManagerCalls`.
    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;

        let specified = params.amountSpecified;
        let amount_in = if specified.is_negative() {
            U256::from_limbs((-specified).into_raw().into_limbs())
        } else {
            U256::from_limbs(specified.into_raw().into_limbs())
        };

        let (x0, x1) = (self.reserve0.get(), self.reserve1.get());
        let ann = self.ann.get();
        let y = self.get_y(amount_in, x0, x1, ann);

        self.reserve0.set(x0 + amount_in);
        self.reserve1.set(y);
        self.last_quote
            .set(if x1 > y { x1 - y } else { U256::ZERO });

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

    fn deployed(vm: &TestVM) -> StableSwapHook {
        vm.set_contract_address(HOOK);
        let mut c = StableSwapHook::from(vm);
        c.constructor(Address::new([0x4e; 20])).unwrap();
        c
    }

    fn e18(n: u64) -> U256 {
        U256::from(n) * U256::from(10u64).pow(U256::from(18))
    }

    #[test]
    fn curve_matches_the_solidity_twin() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        let d = |s: &str| U256::from_str_radix(s, 10).unwrap();

        // the same reference values `uniswap/test/StableSwapHook.t.sol` asserts
        assert_eq!(
            c.get_y(e18(1), e18(1000), e18(600), U256::from(100)),
            d("599011076353954654953")
        );
        assert_eq!(
            c.get_d(e18(1000), e18(600), U256::from(100)),
            d("1598956273533045081014")
        );
    }

    #[test]
    fn a_stable_curve_barely_slips() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        // one token in, almost one token out, which is the whole point of a stable curve
        let out = c.quote(e18(1));
        assert!(
            out > e18(1) * U256::from(98) / U256::from(100),
            "out was {out}"
        );
        assert!(out < e18(1));
    }

    #[test]
    fn only_the_pool_manager_may_move_the_reserves() {
        let vm = TestVM::default();
        let mut c = deployed(&vm);
        vm.set_sender(Address::new([0x99; 20]));
        let key = PoolKey {
            currency0: Address::new([0xc0; 20]),
            currency1: Address::new([0xc1; 20]),
            fee: U24::from(3000u32),
            tickSpacing: stylus_uniswap_v4::types::I24::try_from(60i32).unwrap(),
            hooks: HOOK,
        };
        let params = SwapParams {
            zeroForOne: true,
            amountSpecified: alloy_primitives::I256::try_from(-1_000_000i64).unwrap(),
            sqrtPriceLimitX96: stylus_uniswap_v4::types::U160::from(1u64),
        };
        assert!(c
            .before_swap(Address::ZERO, key, params, Vec::new().into())
            .is_err());
    }
}
