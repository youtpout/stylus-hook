// SPDX-License-Identifier: MIT OR Apache-2.0
//! A Uniswap v4 hook, in pure Rust, that does nothing but arithmetic — with a dial for how much.
//!
//! Stylus buys a lower marginal cost of computation at the price of a fixed cost per call: loading
//! the WASM program, and crossing from the EVM into the WASM runtime. Below some amount of work the
//! fixed cost wins and the same hook is cheaper in Solidity. This hook and its Solidity twin,
//! `uniswap/src/ComputeHook.sol`, run the identical loop so `bench-compute.bash` can sweep the dial
//! and find where the lines cross.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::U64, Address, FixedBytes, U256};
use stylus_sdk::{
    abi::Bytes,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256, StorageU64, StorageU8},
};
use stylus_uniswap_v4::{
    hooks::{selector, HookConfig, HookGuards, IHooks},
    types::{BeforeSwapDelta, PoolKey, SwapParams, U24},
    Permissions, ZERO_DELTA,
};

#[storage]
#[entrypoint]
pub struct ComputeHook {
    pool_manager: StorageAddress,
    rounds: StorageU256,
    last_result: StorageU64,
    /// 0 xorshift64, 1 `mulDiv`, 2 storage writes, 3 fixed-point `rpow`.
    mode: StorageU8,
    /// Written by mode 2. A map rather than a vector because that is what hooks use, so each access
    /// includes hashing the key as well as the store itself.
    slots: StorageMap<U256, StorageU256>,
}

/// 256-bit `mulDiv` needs a 512-bit intermediate, which WASM has to build out of limbs.
type U512 = alloy_primitives::Uint<512, 8>;

/// Full-precision `(a * b) / d`, matching `FullMath.mulDiv` in v4-core.
fn mul_div(a: U256, b: U256, d: U256) -> U256 {
    let wide = U512::from(a) * U512::from(b);
    let quotient = wide / U512::from(d);
    U256::from_limbs([
        quotient.as_limbs()[0],
        quotient.as_limbs()[1],
        quotient.as_limbs()[2],
        quotient.as_limbs()[3],
    ])
}

impl HookConfig for ComputeHook {
    fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    fn permissions(&self) -> Permissions {
        Permissions::none().with_before_swap()
    }
}

#[public]
#[implements(IHooks)]
impl ComputeHook {
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        self.pool_manager.set(pool_manager);
        HookGuards::validate_hook_address(self)
    }

    pub fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    pub fn rounds(&self) -> U256 {
        self.rounds.get()
    }

    pub fn set_rounds(&mut self, new_rounds: U256) {
        self.rounds.set(new_rounds);
    }

    pub fn mode(&self) -> u8 {
        self.mode.get().to::<u8>()
    }

    pub fn set_mode(&mut self, new_mode: u8) {
        self.mode.set(alloy_primitives::aliases::U8::from(new_mode));
    }

    pub fn last_result(&self) -> u64 {
        self.last_result.get().to::<u64>()
    }

    /// xorshift64, `n` times. Must agree with `ComputeHook.sol`.
    ///
    /// A `u64` is a native WASM word. The EVM has no such thing: Solidity's `uint64` is a 256-bit
    /// word masked back down after every shift, which is the asymmetry this benchmark is about.
    pub fn work(&self, n: U256) -> u64 {
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut i = U256::ZERO;
        while i < n {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            i += U256::from(1);
        }
        x
    }

    /// Writes `n` storage slots. Must agree with `ComputeHook.sol`.
    ///
    /// Storage is the one thing Stylus is not supposed to make cheaper: loads and stores are host
    /// operations priced in EVM gas whichever VM runs the contract. This mode is here to check that
    /// rather than assume it.
    pub fn work_storage(&mut self, n: U256) -> U256 {
        let mut last = U256::ZERO;
        let mut i = U256::ZERO;
        while i < n {
            last = i + U256::from(1);
            self.slots.setter(i).set(last);
            i += U256::from(1);
        }
        last
    }

    pub fn read_storage(&self, key: U256) -> U256 {
        self.slots.get(key)
    }

    /// Fixed-point exponentiation by squaring, matching solady's `FixedPointMathLib.rpow`.
    ///
    /// This is what Bunni's Liquidity Density Functions run on every swap:
    /// `LibGeometricDistribution` calls `alphaX96.rpow(length, Q96)` several times per LDF query.
    /// One `rpow` is a chain of full-precision `mulDiv`s.
    pub fn rpow(&self, x: U256, n: U256, precision: U256) -> U256 {
        let two = U256::from(2);
        let mut x = x;
        let mut n = n;
        let mut z = if !(n % two).is_zero() { x } else { precision };
        n /= two;
        while !n.is_zero() {
            x = mul_div(x, x, precision);
            if !(n % two).is_zero() {
                z = mul_div(z, x, precision);
            }
            n /= two;
        }
        z
    }

    /// `n` LDF-sized `rpow` calls. Must agree with `ComputeHook.sol`.
    pub fn work_rpow(&self, n: U256) -> U256 {
        let q96 = U256::from(1u8) << 96;
        let alpha = (q96 / U256::from(100)) * U256::from(99);
        let hundred = U256::from(100);
        let mut acc = q96;
        let mut i = U256::ZERO;
        while i < n {
            acc = self.rpow(alpha + i, hundred, q96);
            i += U256::from(1);
        }
        acc
    }

    /// `mulDiv` on 256-bit words, `n` times. Must agree with `ComputeHook.sol`.
    ///
    /// This is the atom Uniswap's own swap math is built from — `computeSwapStep`, `SqrtPriceMath`
    /// and every tick-walking simulation are mostly chains of it. A 256-bit multiply and divide is
    /// one EVM opcode each; WASM has no 256-bit word and has to do it over limbs. This is the case
    /// that decides whether porting real AMM math to Stylus buys anything.
    pub fn work_mul_div(&self, n: U256) -> U256 {
        let mut a = U256::from_str_radix(
            "9E3779B97F4A7C15C2B2AE3D27D4EB4F165667B19E3779F9165667B19E3779F9",
            16,
        )
        .unwrap();
        let b = U256::from(u128::MAX);
        let d = U256::from(1u8) << 128;
        let high_bit = U256::from(1u8) << 249;
        let mut i = U256::ZERO;
        while i < n {
            a = mul_div(a | high_bit, b, d) + i + U256::from(1);
            i += U256::from(1);
        }
        a
    }
}

#[public]
impl IHooks for ComputeHook {
    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        self.require_pool_manager()?;
        self.require_valid_pool(&key)?;
        let rounds = self.rounds.get();
        // `uint64(...)` on the Solidity side: keep the low 64 bits, do not panic on overflow
        let result = match self.mode.get().to::<u8>() {
            0 => self.work(rounds),
            1 => self.work_mul_div(rounds).as_limbs()[0],
            2 => self.work_storage(rounds).as_limbs()[0],
            _ => self.work_rpow(rounds).as_limbs()[0],
        };
        self.last_result.set(U64::from(result));
        Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::testing::*;

    /// The flags this hook declares: beforeSwap only.
    const HOOK: Address = Address::new([
        0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe,
        0xef, 0x00, 0x00, 0x00, 0x80,
    ]);

    #[test]
    fn xorshift_matches_the_solidity_twin() {
        let vm = TestVM::default();
        vm.set_contract_address(HOOK);
        let mut contract = ComputeHook::from(&vm);
        contract.constructor(Address::new([0x4e; 20])).unwrap();

        // zero rounds leaves the seed untouched
        assert_eq!(contract.work(U256::ZERO), 0x9E37_79B9_7F4A_7C15);
        // reference values, asserted against ComputeHook.sol in uniswap/test/ComputeHook.t.sol
        assert_eq!(contract.work(U256::from(1)), 0xdc1b_77ae_0bf3_4dad);
        assert_eq!(contract.work(U256::from(10)), 0x8f8e_a9d3_4942_8d8e);
        assert_eq!(contract.work(U256::from(100)), 0xab59_17a8_1f0f_b2ae);
    }

    #[test]
    fn mul_div_matches_the_solidity_twin() {
        let vm = TestVM::default();
        vm.set_contract_address(HOOK);
        let mut contract = ComputeHook::from(&vm);
        contract.constructor(Address::new([0x4e; 20])).unwrap();

        let expect = |hex: &str| U256::from_str_radix(hex, 16).unwrap();
        assert_eq!(
            contract.work_mul_div(U256::ZERO),
            expect("9E3779B97F4A7C15C2B2AE3D27D4EB4F165667B19E3779F9165667B19E3779F9")
        );
        assert_eq!(
            contract.work_mul_div(U256::from(1)),
            expect("9e3779b97f4a7c15c2b2ae3d27d4eb4e781eedf81eecfde353a3b97476628eaa")
        );
        assert_eq!(
            contract.work_mul_div(U256::from(100)),
            expect("9e3779b97f4a7c15c2b2ae3d27d4eb1148aadb3be51f0179088a57ce0f0bae90")
        );
    }

    #[test]
    fn rpow_matches_the_solidity_twin() {
        let vm = TestVM::default();
        vm.set_contract_address(HOOK);
        let mut contract = ComputeHook::from(&vm);
        contract.constructor(Address::new([0x4e; 20])).unwrap();

        let d = |s: &str| U256::from_str_radix(s, 10).unwrap();
        assert_eq!(
            contract.work_rpow(U256::ZERO),
            d("79228162514264337593543950336")
        );
        assert_eq!(
            contract.work_rpow(U256::from(1)),
            d("29000069819872093002514032964")
        );
        assert_eq!(
            contract.work_rpow(U256::from(10)),
            d("29000069819872093002514033297")
        );
    }

    #[test]
    fn only_declares_before_swap() {
        let vm = TestVM::default();
        vm.set_contract_address(HOOK);
        let mut contract = ComputeHook::from(&vm);
        contract.constructor(Address::new([0x4e; 20])).unwrap();
        assert_eq!(
            HookConfig::permissions(&contract).flags(),
            stylus_uniswap_v4::permissions::BEFORE_SWAP_FLAG
        );
    }
}
