// SPDX-License-Identifier: MIT OR Apache-2.0
//! Airdrop accounting for a Uniswap v4 hook, running on Arbitrum Stylus.
//!
//! `AirdropHookProxy.sol` is the Solidity shell whose address carries the v4 permission flags; it
//! forwards every `afterSwap` callback here, so all of the hook's state and logic is Rust/WASM.
//! The Solidity replica used by the Foundry tests lives in `uniswap/test/mocks/MockStylusAirdrop.sol`
//! and the ABI is mirrored in `uniswap/src/interfaces/IAirdropHook.sol`.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;
use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256},
};

/// Airdrop is computed from the amount swapped by a user and the number of swaps they did.
#[storage]
pub struct SwapInfo {
    /// amount swapped is profitable for liquidity providers
    amount0: StorageU256,
    amount1: StorageU256,
    /// number of swaps is profitable for the network
    counter0: StorageU256,
    counter1: StorageU256,
}

#[storage]
#[entrypoint]
pub struct AirdropHook {
    total_swap_user: StorageMap<FixedBytes<32>, StorageMap<Address, SwapInfo>>,
    total_swap: StorageMap<FixedBytes<32>, SwapInfo>,
    users_count: StorageMap<FixedBytes<32>, StorageU256>,
    user_exist: StorageMap<FixedBytes<32>, StorageMap<Address, StorageBool>>,
    airdrop_token: StorageMap<FixedBytes<32>, StorageAddress>,
    claimed: StorageMap<FixedBytes<32>, StorageMap<Address, StorageBool>>,
    hook: StorageAddress,
}

sol! {
    #[derive(Debug)]
    error NotHook();
    #[derive(Debug)]
    error HookAlreadyDefined();
    #[derive(Debug)]
    error AirdropNotEnd();
    #[derive(Debug)]
    error AlreadyClaimed();
    #[derive(Debug)]
    error TokenCallFailed();
}

sol_interface! {
    interface IERC20Airdrop {
        function totalAirdrop() external view returns (uint256);

        function restAirdrop() external view returns (uint256);

        function claim(address receiver, uint256 amount) external;
    }
}

#[derive(SolidityError, Debug)]
pub enum HookError {
    NotHook(NotHook),
    HookAlreadyDefined(HookAlreadyDefined),
    AirdropNotEnd(AirdropNotEnd),
    AlreadyClaimed(AlreadyClaimed),
    TokenCallFailed(TokenCallFailed),
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// 80 % of the airdrop is distributed on swap volume, 20 % on the number of swaps.
/// Split over the two tokens, that is 40 % / 40 % / 10 % / 10 %.
const VOLUME_PERCENT: u64 = 40;
const COUNTER_PERCENT: u64 = 10;

#[public]
impl AirdropHook {
    /// The v4 hook contract allowed to write into this contract.
    pub fn hook(&self) -> Address {
        self.hook.get()
    }

    /// One-shot binding of the hook contract. Reverts once set.
    pub fn set_hook(&mut self, value: Address) -> Result<()> {
        if !self.hook.get().is_zero() {
            return Err(HookAlreadyDefined {}.into());
        }
        self.hook.set(value);
        Ok(())
    }

    pub fn get_total_swap(&self, pool_id: FixedBytes<32>) -> (U256, U256, U256, U256) {
        let data = self.total_swap.get(pool_id);
        (
            data.amount0.get(),
            data.amount1.get(),
            data.counter0.get(),
            data.counter1.get(),
        )
    }

    pub fn get_total_swap_user(
        &self,
        pool_id: FixedBytes<32>,
        user: Address,
    ) -> (U256, U256, U256, U256) {
        let pool = self.total_swap_user.get(pool_id);
        let data = pool.get(user);
        (
            data.amount0.get(),
            data.amount1.get(),
            data.counter0.get(),
            data.counter1.get(),
        )
    }

    pub fn total_users(&self, pool_id: FixedBytes<32>) -> U256 {
        self.users_count.get(pool_id)
    }

    pub fn airdrop_token(&self, pool_id: FixedBytes<32>) -> Address {
        self.airdrop_token.get(pool_id)
    }

    pub fn has_claimed(&self, pool_id: FixedBytes<32>, user: Address) -> bool {
        self.claimed.get(pool_id).get(user)
    }

    /// Books one swap for `user`. Only the hook may call this.
    pub fn add_after_swap(
        &mut self,
        pool_id: FixedBytes<32>,
        user: Address,
        zero_for_one: bool,
        amount_specified: U256,
    ) -> Result<()> {
        self.only_hook()?;

        // once the airdrop token is defined the airdrop accounting is closed
        if !self.airdrop_token.get(pool_id).is_zero() {
            return Ok(());
        }

        if !self.user_exist.get(pool_id).get(user) {
            self.user_exist.setter(pool_id).setter(user).set(true);
            let mut count = self.users_count.setter(pool_id);
            let old_count = count.get();
            count.set(old_count + U256::from(1));
        }

        let mut swap_pool = self.total_swap_user.setter(pool_id);
        let mut swap_user = swap_pool.setter(user);
        let mut swap_total = self.total_swap.setter(pool_id);

        if zero_for_one {
            let amount = swap_user.amount1.get();
            swap_user.amount1.set(amount + amount_specified);
            let counter = swap_user.counter1.get();
            swap_user.counter1.set(counter + U256::from(1));

            let amount = swap_total.amount1.get();
            swap_total.amount1.set(amount + amount_specified);
            let counter = swap_total.counter1.get();
            swap_total.counter1.set(counter + U256::from(1));
        } else {
            let amount = swap_user.amount0.get();
            swap_user.amount0.set(amount + amount_specified);
            let counter = swap_user.counter0.get();
            swap_user.counter0.set(counter + U256::from(1));

            let amount = swap_total.amount0.get();
            swap_total.amount0.set(amount + amount_specified);
            let counter = swap_total.counter0.get();
            swap_total.counter0.set(counter + U256::from(1));
        }

        Ok(())
    }

    /// Freezes the accounting and binds the token distributed for this pool.
    pub fn close_airdrop(&mut self, pool_id: FixedBytes<32>, token: Address) -> Result<()> {
        self.only_hook()?;
        self.airdrop_token.setter(pool_id).set(token);
        Ok(())
    }

    pub fn claim_airdrop(&mut self, pool_id: FixedBytes<32>, receiver: Address) -> Result<()> {
        self.only_hook()?;

        let token_address = self.airdrop_token.get(pool_id);
        if token_address.is_zero() {
            return Err(AirdropNotEnd {}.into());
        }

        if self.claimed.get(pool_id).get(receiver) {
            return Err(AlreadyClaimed {}.into());
        }

        // set claimed first to prevent from reentrancy try
        self.claimed.setter(pool_id).setter(receiver).set(true);

        let amount = self.compute_claim(pool_id, token_address, receiver)?;

        let token = IERC20Airdrop::new(token_address);
        let config = Call::new_mutating(self);
        token
            .claim(self.vm(), config, receiver, amount)
            .map_err(|_| HookError::TokenCallFailed(TokenCallFailed {}))
    }

    pub fn amount_to_claim(&self, pool_id: FixedBytes<32>, receiver: Address) -> Result<U256> {
        let token_address = self.airdrop_token.get(pool_id);
        if token_address.is_zero() {
            return Ok(U256::ZERO);
        }
        self.compute_claim(pool_id, token_address, receiver)
    }
}

impl AirdropHook {
    fn only_hook(&self) -> Result<()> {
        if self.vm().msg_sender() != self.hook.get() {
            return Err(NotHook {}.into());
        }
        Ok(())
    }

    fn compute_claim(
        &self,
        pool_id: FixedBytes<32>,
        token_address: Address,
        receiver: Address,
    ) -> Result<U256> {
        let token = IERC20Airdrop::new(token_address);
        let airdrop_amount = token
            .total_airdrop(self.vm(), Call::new())
            .map_err(|_| HookError::TokenCallFailed(TokenCallFailed {}))?;

        let pool = self.total_swap_user.get(pool_id);
        let swap_user = pool.get(receiver);
        let swap_total = self.total_swap.get(pool_id);

        let amount_vol0 = Self::share(
            airdrop_amount,
            swap_user.amount0.get(),
            swap_total.amount0.get(),
            VOLUME_PERCENT,
        );
        let amount_vol1 = Self::share(
            airdrop_amount,
            swap_user.amount1.get(),
            swap_total.amount1.get(),
            VOLUME_PERCENT,
        );
        let amount_count0 = Self::share(
            airdrop_amount,
            swap_user.counter0.get(),
            swap_total.counter0.get(),
            COUNTER_PERCENT,
        );
        let amount_count1 = Self::share(
            airdrop_amount,
            swap_user.counter1.get(),
            swap_total.counter1.get(),
            COUNTER_PERCENT,
        );

        Ok(amount_vol0 + amount_vol1 + amount_count0 + amount_count1)
    }

    fn share(amount_to_airdrop: U256, user_volume: U256, total_volume: U256, percent: u64) -> U256 {
        let num = amount_to_airdrop * user_volume * U256::from(percent);
        // 1 to prevent divide by zero
        let den = U256::from(1) + total_volume * U256::from(100);
        num / den
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use alloy_sol_types::SolValue;
    use stylus_sdk::testing::*;

    const HOOK: Address = Address::new([0x11; 20]);
    const TOKEN: Address = Address::new([0x22; 20]);
    const AIRDROP_SUPPLY: u128 = 50_000_000_000_000_000_000_000_000; // 50_000_000 ether

    fn pool_id(byte: u8) -> FixedBytes<32> {
        FixedBytes::from([byte; 32])
    }

    fn alice() -> Address {
        Address::from([0xa1; 20])
    }

    fn bob() -> Address {
        Address::from([0xb0; 20])
    }

    fn bound(vm: &TestVM) -> AirdropHook {
        let mut contract = AirdropHook::from(vm);
        contract.set_hook(HOOK).unwrap();
        vm.set_sender(HOOK);
        contract
    }

    /// `totalAirdrop()` — resolved through a static call by the generated interface.
    fn mock_total_airdrop(vm: &TestVM, supply: U256) {
        vm.mock_static_call(
            TOKEN,
            alloc::vec![0x5c, 0xe9, 0x7d, 0xbb],
            Ok(supply.abi_encode()),
        );
    }

    /// `claim(address,uint256)` — a state-mutating call, so it carries a value of zero.
    fn mock_claim(vm: &TestVM, receiver: Address, amount: U256) {
        let mut data: Vec<u8> = alloc::vec![0xaa, 0xd3, 0xec, 0x96];
        data.extend_from_slice(&(receiver, amount).abi_encode());
        vm.mock_call(TOKEN, data, U256::ZERO, Ok(Vec::new()));
    }

    #[test]
    fn accounting_splits_by_direction_and_pool() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        let pool = pool_id(1);

        contract
            .add_after_swap(pool, alice(), true, U256::from(1_000u64))
            .unwrap();
        contract
            .add_after_swap(pool, alice(), true, U256::from(2_000u64))
            .unwrap();
        contract
            .add_after_swap(pool, bob(), false, U256::from(500u64))
            .unwrap();

        assert_eq!(contract.total_users(pool), U256::from(2));

        // zeroForOne swaps are booked on the "1" side, the other way round on the "0" side
        let (amount0, amount1, counter0, counter1) = contract.get_total_swap(pool);
        assert_eq!(amount1, U256::from(3_000u64));
        assert_eq!(counter1, U256::from(2));
        assert_eq!(amount0, U256::from(500u64));
        assert_eq!(counter0, U256::from(1));

        let (_, alice_amount1, _, alice_counter1) = contract.get_total_swap_user(pool, alice());
        assert_eq!(alice_amount1, U256::from(3_000u64));
        assert_eq!(alice_counter1, U256::from(2));

        // a different pool keeps its own books
        assert_eq!(contract.total_users(pool_id(2)), U256::ZERO);
    }

    #[test]
    fn only_the_hook_can_write() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        vm.set_sender(Address::from([0x99; 20]));

        assert!(matches!(
            contract.add_after_swap(pool_id(1), alice(), true, U256::from(1u64)),
            Err(HookError::NotHook(_))
        ));
        assert!(matches!(
            contract.close_airdrop(pool_id(1), TOKEN),
            Err(HookError::NotHook(_))
        ));
    }

    #[test]
    fn hook_binds_once() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        assert!(matches!(
            contract.set_hook(Address::from([0x22; 20])),
            Err(HookError::HookAlreadyDefined(_))
        ));
    }

    #[test]
    fn closing_freezes_the_accounting() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        let pool = pool_id(1);

        contract
            .add_after_swap(pool, alice(), true, U256::from(1_000u64))
            .unwrap();
        contract.close_airdrop(pool, TOKEN).unwrap();
        contract
            .add_after_swap(pool, bob(), true, U256::from(1_000u64))
            .unwrap();

        assert_eq!(contract.total_users(pool), U256::from(1));
        assert_eq!(contract.airdrop_token(pool), TOKEN);
    }

    #[test]
    fn amount_to_claim_is_zero_before_the_airdrop_closes() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        let pool = pool_id(1);

        contract
            .add_after_swap(pool, alice(), true, U256::from(1_000u64))
            .unwrap();

        assert_eq!(contract.amount_to_claim(pool, alice()).unwrap(), U256::ZERO);
    }

    #[test]
    fn claim_pays_the_computed_share_once() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        let pool = pool_id(1);
        let supply = U256::from(AIRDROP_SUPPLY);

        contract
            .add_after_swap(
                pool,
                alice(),
                true,
                U256::from(3_000_000_000_000_000_000u64),
            )
            .unwrap();
        contract
            .add_after_swap(pool, bob(), true, U256::from(1_000_000_000_000_000_000u64))
            .unwrap();
        contract.close_airdrop(pool, TOKEN).unwrap();

        // Same numbers as `AirdropHook.sol` / `MockStylusAirdrop.sol` for the same swaps.
        let expected_alice = U256::from_str_radix("17487562189054726368121703", 10).unwrap();
        let expected_bob = U256::from_str_radix("7487562189054726368146703", 10).unwrap();

        // `TestVM` keeps a single global return-data buffer that every mock registration
        // overwrites, so the mock whose payload must be decoded is registered last.
        mock_claim(&vm, alice(), expected_alice);
        mock_total_airdrop(&vm, supply);

        assert_eq!(
            contract.amount_to_claim(pool, alice()).unwrap(),
            expected_alice
        );
        assert_eq!(contract.amount_to_claim(pool, bob()).unwrap(), expected_bob);

        contract.claim_airdrop(pool, alice()).unwrap();
        assert!(contract.has_claimed(pool, alice()));

        assert!(matches!(
            contract.claim_airdrop(pool, alice()),
            Err(HookError::AlreadyClaimed(_))
        ));
    }

    #[test]
    fn claim_reverts_before_the_airdrop_closes() {
        let vm = TestVM::default();
        let mut contract = bound(&vm);
        assert!(matches!(
            contract.claim_airdrop(pool_id(1), alice()),
            Err(HookError::AirdropNotEnd(_))
        ));
    }
}
