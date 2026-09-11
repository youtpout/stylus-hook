// SPDX-License-Identifier: MIT OR Apache-2.0
//! The pm-AMM's arithmetic in pure Rust, to be priced against the Solidity it was ported from.
//!
//! # Why this workload
//!
//! Every hook this repository has benchmarked so far loses, because Stylus trades a lower marginal
//! cost of computation for a fixed cost per call, and a hook that counts swaps or reads a vault
//! never does enough arithmetic to earn that back. TWAMM looked like the exception until the version
//! actually deployed turned out to use plain integer `mulDiv`.
//!
//! The pm-AMM is a better bet for one reason: its arithmetic cannot be removed. With `z = (y-x)/L`,
//! Paradigm's invariant is
//!
//! ```text
//! f(y)  = (y - x)·Phi(z) + L·phi(z) - y
//! f'(y) = Phi(z) - 1
//! ```
//!
//! which is transcendental in `y`. There is no closed form, so a solve is mandatory, and every
//! iteration of it needs the Gaussian CDF and PDF. The derivative collapsing to `Phi(z) - 1` is a
//! gift — the two `z·phi(z)` terms cancel — but it still costs a full Gaussian evaluation per step.
//!
//! # What is being compared
//!
//! [`gaussian`] is a bit-exact port of `primitivefinance/solstat`, the library the one existing
//! pm-AMM hook uses, down to Solmate's `expWad` underneath it. `uniswap/src/PmAmmMath.sol` calls
//! that same Solidity library and runs the same Newton solve. So the two sides compute identical
//! numbers by identical steps, and what the benchmark measures is the language.
//!
//! The existing hook solves by 100-step bisection, which is 200 Gaussian evaluations per swap. That
//! is not what is measured here: benchmarking against a lazy implementation measures nothing, and
//! this repository has already made that mistake once. Newton is the honest floor.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

pub mod gaussian;
pub mod montgomery;

use alloc::vec::Vec;

use alloy_primitives::{I256, U256};
use alloy_sol_types::{sol, SolError};
use stylus_sdk::{prelude::*, storage::StorageU8};

use gaussian::{cdf, erfc, exp_wad, pdf, ONE};

sol! {
    /// Newton cannot step from a point where the invariant has no slope.
    #[derive(Debug)]
    error ZeroDerivative();
    /// Liquidity of zero has no curve.
    #[derive(Debug)]
    error ZeroLiquidity();
}

#[storage]
#[entrypoint]
pub struct PmAmmMath {
    _pad: StorageU8,
}

#[public]
impl PmAmmMath {
    /// A no-op, to subtract the cost of being called at all.
    pub fn baseline(&self, _x: I256) -> I256 {
        I256::ZERO
    }

    /// `n` iterations of `a * b / c`, wrapping, with the operand magnitude left to the caller.
    ///
    /// The EVM charges 5 gas for `MUL` and 5 for `DIV` whatever the operands are. A `U256` here is
    /// four 64-bit limbs and `ruint` short-circuits on the zero ones, so the same expression should
    /// cost far more when the operands are wide. This is the dial to test that with.
    pub fn mul_div_loop(&self, n: U256, a: U256, b: U256, c: U256) -> U256 {
        let mut acc = U256::ZERO;
        let mut i = U256::ZERO;
        while i < n {
            acc = acc.wrapping_add(a.wrapping_mul(b) / c);
            i += U256::from(1);
        }
        acc
    }

    /// `n` modular multiplications in a ~254-bit prime field — BN254's scalar field, the one
    /// Poseidon and every BN254 SNARK verifier work in.
    ///
    /// The EVM has `MULMOD` as one opcode at 8 gas. WASM has nothing of the sort, so this is
    /// `ruint`'s `mul_mod`: a 512-bit product followed by a reduction. This is the dial that says
    /// whether porting field arithmetic to Stylus is worth anything.
    pub fn mulmod_loop(&self, n: U256, a: U256, b: U256, m: U256) -> U256 {
        let mut acc = U256::ZERO;
        let mut i = U256::ZERO;
        while i < n {
            acc = acc.wrapping_add(a).mul_mod(b, m);
            i += U256::from(1);
        }
        acc
    }

    /// The same, in the 64-bit Goldilocks field `2^64 - 2^32 + 1` that Plonky2, Plonky3 and Risc0
    /// verify over.
    ///
    /// A `u64` multiply is one WASM instruction and the product fits a `u128`, so the whole field
    /// operation is native. Solidity cannot reach a 64-bit multiply at all.
    pub fn goldilocks_loop(&self, n: U256, a: U256, b: U256) -> U256 {
        const P: u64 = 0xFFFF_FFFF_0000_0001;
        let mut acc = a.as_limbs()[0];
        let bb = b.as_limbs()[0];
        let mut i = U256::ZERO;
        while i < n {
            let x = ((acc as u128 + bb as u128) % P as u128) as u64;
            acc = ((x as u128 * bb as u128) % P as u128) as u64;
            i += U256::from(1);
        }
        U256::from(acc)
    }

    /// `n` Montgomery multiplications in BN254's scalar field — how a cryptography library actually
    /// multiplies, with no division anywhere.
    ///
    /// Compared against Solidity's `MULMOD` opcode, this is the measurement that decides whether any
    /// 256-bit prime field is worth porting: Poseidon over BN254, P-256 signature verification, a
    /// SNARK verifier's field layer.
    pub fn montmul_loop(&self, n: U256, a: U256, b: U256) -> U256 {
        let aa = a.into_limbs();
        let bb = b.into_limbs();
        let mut acc = aa;
        let mut i = U256::ZERO;
        while i < n {
            acc = montgomery::mont_mul(&acc, &bb);
            i += U256::from(1);
        }
        U256::from_limbs(acc)
    }

    // --- the primitives, priced one at a time ---------------------------------------------------

    pub fn exp_wad(&self, x: I256) -> I256 {
        exp_wad(x)
    }

    pub fn erfc(&self, x: I256) -> I256 {
        erfc(x)
    }

    pub fn cdf(&self, x: I256) -> I256 {
        cdf(x)
    }

    pub fn pdf(&self, x: I256) -> I256 {
        pdf(x)
    }

    /// The invariant and its slope at `y`, which is what one Newton step costs.
    ///
    /// Named `residual` to match the Solidity, which cannot call it `invariant`: forge claims any
    /// such function as an invariant test and then rejects it for taking parameters.
    pub fn residual(&self, l: I256, x: I256, y: I256) -> (I256, I256) {
        let z = (y - x).wrapping_mul(ONE) / l;
        let c = cdf(z);
        let p = pdf(z);
        let f = (y - x).wrapping_mul(c) / ONE + l.wrapping_mul(p) / ONE - y;
        (f, c - ONE)
    }

    /// Solves the pm-AMM invariant for `y` by Newton, with a fixed iteration count so the gas is
    /// not input-dependent.
    pub fn solve_newton(&self, l: I256, x: I256, y0: I256, iters: U256) -> Result<I256, Vec<u8>> {
        if l.is_zero() {
            return Err(ZeroLiquidity {}.abi_encode());
        }
        let mut y = y0;
        let mut i = U256::ZERO;
        while i < iters {
            let (f, df) = self.residual(l, x, y);
            if df.is_zero() {
                return Err(ZeroDerivative {}.abi_encode());
            }
            y -= f.wrapping_mul(ONE) / df;
            i += U256::from(1);
        }
        Ok(y)
    }

    /// `n` Newton iterations from a fixed starting point. The dial the benchmark sweeps, and the
    /// same function `PmAmmMath.sol` exposes.
    pub fn work(&self, n: U256) -> Result<I256, Vec<u8>> {
        let e18 = ONE;
        let l = I256::unchecked_from(100i64).wrapping_mul(e18);
        let x = I256::unchecked_from(40i64).wrapping_mul(e18);
        let y0 = I256::unchecked_from(60i64).wrapping_mul(e18);
        self.solve_newton(l, x, y0, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    fn deployed(vm: &TestVM) -> PmAmmMath {
        PmAmmMath::from(vm)
    }

    /// The solve has to land on an actual root, or the gas figure is meaningless. `PmAmmMath.sol`
    /// asserts the same thing about the same inputs.
    #[test]
    fn newton_converges_to_a_root_of_the_invariant() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        let y = c.work(U256::from(8)).unwrap();
        let (f, _) = c.residual(
            I256::unchecked_from(100i64) * ONE,
            I256::unchecked_from(40i64) * ONE,
            y,
        );
        assert!(
            f.abs() < I256::unchecked_from(1_000_000i64),
            "residual {f} too large"
        );
    }

    /// And it has to converge to the same root the Solidity does.
    #[test]
    fn the_root_matches_the_solidity() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        assert_eq!(
            c.work(U256::from(8)).unwrap(),
            "39788634305350540988".parse::<I256>().unwrap()
        );
    }

    #[test]
    fn zero_liquidity_is_refused() {
        let vm = TestVM::default();
        let c = deployed(&vm);
        assert!(c.solve_newton(I256::ZERO, ONE, ONE, U256::from(1)).is_err());
    }
}
