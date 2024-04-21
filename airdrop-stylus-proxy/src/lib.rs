// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::sol;
use stylus_sdk::{msg, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct AirdropHook {
        mapping(bytes32 => mapping(address => SwapInfo)) total_swap_user;
        mapping(bytes32 => SwapInfo) total_swap;
        mapping(bytes32 => uint256) users_count;
        mapping(bytes32 => mapping(address => bool)) user_exist;
        mapping(bytes32 => address) airdrop_token;
        mapping(bytes32 => mapping(address => bool)) claimed;
        address hook;
    }

    // Airdrop is caculated between total amount swaped by user and number of swap did
    pub struct SwapInfo{
        // amount swapped is profitable for liquidity provider
        uint256 amount0;
        uint256 amount1;
        // number swapped is profitable for network miner
        uint256 counter0;
        uint256 counter1;
    }

}

sol! {
    error NotHook();
    error HookAlreadyDefined();
    error AirdropNotEnd();
    error AlreadyClaimed();
}

sol_interface! {
    interface IERC20Airdrop {
        function totalAirdrop() external view returns (uint256);

        function restAirdrop() external returns (uint256);

        function claim(address receiver, uint256 amount) external;
    }

}

#[derive(SolidityError)]
pub enum HookError {
    NotHook(NotHook),
    HookAlreadyDefined(HookAlreadyDefined),
    AirdropNotEnd(AirdropNotEnd),
    AlreadyClaimed(AlreadyClaimed),
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl AirdropHook {
    /// Gets the number from storage.
    // pub fn get_total_swap_user(
    //     &self,
    //     pool_id: FixedBytes<32>,
    //     user: address,
    // ) -> (U256, U256, U256, U256) {
    //     self.total_swap_user.get(pool_id, user)
    // }

    pub fn get_total_swap(&self, pool_id: FixedBytes<32>) -> (U256, U256, U256, U256) {
        let data = self.total_swap.get(pool_id);
        (
            data.amount0.get(),
            data.amount1.get(),
            data.counter0.get(),
            data.counter1.get(),
        )
    }

    // pub fn get_users(&self, pool_id: FixedBytes<32>) -> Vec<Address> {
    //     self.users.get(pool_id)
    // }

    // pub fn get_airdrop_token(&self, pool_id: FixedBytes<32>) -> Address {
    //     self.airdrop_token.get(pool_id)
    // }

    pub fn set_hook(&mut self, value: Address) -> Result<()> {
        let hook: Address = self.hook.get();

        if !hook.is_zero() {
            return Err(HookError::HookAlreadyDefined(HookAlreadyDefined {}));
        }

        self.hook.set(value);
        Ok(())
    }

    pub fn add_after_swap(
        &mut self,
        pool_id: FixedBytes<32>,
        sender: Address,
        zero_for_one: bool,
        amount_specified: U256,
    ) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        // if the token for airdrop if define, aidrop calculation is finished
        if !self.airdrop_token.get(pool_id).is_zero() {
            return Ok(());
        }

        let exist: bool = self.user_exist.get(pool_id).get(sender);
        if !exist {
            self.user_exist.setter(pool_id).insert(sender, true);
            let mut count = self.users_count.setter(pool_id);
            let old_count = count.get();
            count.set(old_count + U256::from(1));
        }

        let mut swap_pool = self.total_swap_user.setter(pool_id);
        let mut swap_user = swap_pool.setter(sender);
        let mut swap_total = self.total_swap.setter(pool_id);

        // increment swap amount and total swap
        if zero_for_one {
            let amount_total1: U256 = swap_total.amount1.get();
            let counter_total1: U256 = swap_total.counter1.get();
            let amount_user1: U256 = swap_user.amount1.get();
            let counter_user1: U256 = swap_user.counter1.get();
            swap_user.amount1.set(amount_specified + amount_user1);
            swap_total.amount1.set(amount_specified + amount_total1);
            swap_user.counter1.set(counter_user1 + U256::from(1));
            swap_total.counter1.set(counter_total1 + U256::from(1));
        } else {
            let amount_total0: U256 = swap_total.amount0.get();
            let counter_total0: U256 = swap_total.counter0.get();
            let amount_user0: U256 = swap_user.amount0.get();
            let counter_user0: U256 = swap_user.counter0.get();
            swap_user.amount0.set(amount_specified + amount_user0);
            swap_total.amount0.set(amount_specified + amount_total0);
            swap_user.counter0.set(counter_user0 + U256::from(1));
            swap_total.counter0.set(counter_total0 + U256::from(1));
        }

        return Ok(());
    }

    fn close_airdrop(&mut self, pool_id: FixedBytes<32>, token: Address) -> Result<()> {
        self.airdrop_token.setter(pool_id).set(token);
        return Ok(());
    }

    pub fn claim_airdrop(&mut self, pool_id: FixedBytes<32>, receiver: Address) -> Result<()> {
        let hook: Address = self.hook.get();

        if msg::sender() != hook {
            return Err(HookError::NotHook(NotHook {}));
        }

        let token_address: Address = self.airdrop_token.get(pool_id);
        let token = IERC20Airdrop::new(token_address);
        if token.is_zero() {
            return Err(HookError::AirdropNotEnd(AirdropNotEnd {}));
        }

        // set claimed first to prevent from reentrancy try
        self.claimed.setter(pool_id).setter(receiver).set(true);

        let amount = self._amount_to_claim(pool_id, token, receiver);
        IERC20Airdrop::new(token_address).claim(&mut *self, receiver, amount?);
        return Ok(());
    }

    pub fn claim(&mut self, pool_id: FixedBytes<32>) -> Result<()> {
        let receiver: Address = msg::sender();

        let token_address: Address = self.airdrop_token.get(pool_id);
        let token = IERC20Airdrop::new(token_address);
        if token.is_zero() {
            return Err(HookError::AirdropNotEnd(AirdropNotEnd {}));
        }

        // set claimed first to prevent from reentrancy try
        self.claimed.setter(pool_id).setter(receiver).set(true);

        let amount = self._amount_to_claim(pool_id, token, receiver);
        IERC20Airdrop::new(token_address).claim(&mut *self, receiver, amount?);
        return Ok(());
    }

    pub fn amount_to_claim(&self, pool_id: FixedBytes<32>, receiver: Address) -> Result<U256> {
        let token = IERC20Airdrop::new(self.airdrop_token.get(pool_id));
        return Ok(self._amount_to_claim(pool_id, token, receiver)?);
    }

    fn _amount_to_claim(
        &self,
        pool_id: FixedBytes<32>,
        token: IERC20Airdrop,
        receiver: Address,
    ) -> Result<U256> {
        let airdrop_amount = token.total_airdrop(&*self).ok().unwrap();
        let swap_pool = self.total_swap_user.get(pool_id);
        let swap_user = swap_pool.get(receiver);
        let swap_total = self.total_swap.get(pool_id);

        // 80 % base on volume and 10% on number of swap
        // so 40 % for each token
        let amount_vol0 = self._calculate_token_airdrop(
            airdrop_amount,
            swap_user.amount0.get(),
            swap_total.amount0.get(),
            U256::from(40),
        );
        let amount_vol1 = self._calculate_token_airdrop(
            airdrop_amount,
            swap_user.amount1.get(),
            swap_total.amount1.get(),
            U256::from(40),
        );
        let amount_count0 = self._calculate_token_airdrop(
            airdrop_amount,
            swap_user.counter0.get(),
            swap_total.counter0.get(),
            U256::from(10),
        );
        let amount_count1 = self._calculate_token_airdrop(
            airdrop_amount,
            swap_user.counter1.get(),
            swap_total.counter1.get(),
            U256::from(10),
        );

        return Ok(amount_vol0 + amount_vol1 + amount_count0 + amount_count1);
    }

    fn _calculate_token_airdrop(
        &self,
        amount_to_airdrop: U256,
        user_volume: U256,
        total_volume: U256,
        percent: U256,
    ) -> U256 {
        let num = amount_to_airdrop * user_volume * percent;
        // 1 to prevent divide by zero
        let den = U256::from(1) + total_volume * U256::from(100);
        num / den
    }
}
