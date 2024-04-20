// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, Signed, I32, U128, U256};
use alloy_sol_types::{sol, SolError};
use std::convert::TryInto;
use stylus_sdk::{msg, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct LimitOrder {
        uint256 epoch_next;
        address hook;
        mapping(bytes32 => int32) tick_lower_lasts;
        mapping(bytes32 => uint256) epochs;
        mapping(uint256 => EpochInfo) epoch_infos;
        address pool_manager;
    }

    pub struct EpochInfo {
        bool filled;
        address currency0;
        address currency1;
        uint256 token0_total;
        uint256 token1_total;
        uint128 liquidity_total;
        mapping(address => uint128) liquidity;
    }

}

sol! {
    error ZeroLiquidity();
    error InRange();
    error CrossedRange();
    error Filled();
    error NotFilled();
    error NotPoolManagerToken();
}

sol_interface! {
    interface IPoolManager {
        function getSlot0(bytes32 id) external view returns (uint160 sqrtPriceX96, int24 tick, uint16 protocolFee, uint24 swapFee);
    }
}

// #[derive(SolidityError)]
pub enum HookError {
    ZeroLiquidity(ZeroLiquidity),
    InRange(InRange),
    CrossedRange(CrossedRange),
    Filled(Filled),
    NotFilled(NotFilled),
    NotPoolManagerToken(NotPoolManagerToken),
    ExternalCall(stylus_sdk::call::Error),
}

/// We will soon provide a `#[derive(SolidityError)]` to clean this up.
impl From<stylus_sdk::call::Error> for HookError {
    fn from(err: stylus_sdk::call::Error) -> Self {
        Self::ExternalCall(err)
    }
}

impl From<HookError> for Vec<u8> {
    fn from(val: HookError) -> Self {
        match val {
            HookError::ZeroLiquidity(err) => err.encode(),
            HookError::InRange(err) => err.encode(),
            HookError::CrossedRange(err) => err.encode(),
            HookError::Filled(err) => err.encode(),
            HookError::NotFilled(err) => err.encode(),
            HookError::NotPoolManagerToken(err) => err.encode(),
            HookError::ExternalCall(err) => err.into(),
        }
    }
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl LimitOrder {
    pub fn set_pool_manager(&mut self, value: Address) -> Result<()> {
        self.pool_manager.set(value);
        Ok(())
    }

    pub fn pool_manager(&self) -> Address {
        self.pool_manager.get()
    }

    pub fn tick_lower_lasts(&self, pool_id: FixedBytes<32>) -> i32 {
        self.tick_lower_lasts.get(pool_id).as_i32()
    }

    fn set_tick_lower_lasts(&mut self, pool_id: FixedBytes<32>, tick_lower: i32) -> Result<()> {
        // why I need cas between i32 and Signed<32,i> is boring
        self.tick_lower_lasts
            .replace(pool_id, Signed::<32, 1>::unchecked_from(tick_lower));
        return Ok(());
    }

    pub fn epochs(&self, pool_id: FixedBytes<32>) -> U256 {
        self.epochs.get(pool_id)
    }

    pub fn get_epoch_liquidity(&self, epoch: U256, owner: Address) -> u128 {
        let epoch_info = self.epoch_infos.get(epoch);
        epoch_info.liquidity.get(owner).to::<u128>()
    }

    pub fn get_epoch(&self, id: FixedBytes<32>) -> U256 {
        self.epochs.get(id)
    }

    fn set_epoch(&mut self, id: FixedBytes<32>, epoch: U256) -> Result<()> {
        self.epochs.replace(id, epoch);
        return Ok(());
    }

    pub fn epoch_next(&self) -> U256 {
        self.epoch_next.get()
    }

    pub fn after_initialize(
        &mut self,
        pool_id: FixedBytes<32>,
        tick: i32,
        tick_spacing: i32,
    ) -> Result<()> {
        let last = self.get_tick_lower(tick, tick_spacing);
        let converted: Signed<32, 1> = Signed::<32, 1>::unchecked_from(last);
        self.tick_lower_lasts.replace(pool_id, converted);
        return Ok(());
    }

    pub fn after_swap(
        &mut self,
        pool_id: FixedBytes<32>,
        tick: i32,
        tick_spacing: i32,
    ) -> Result<()> {
        let last = self.get_tick_lower(tick, tick_spacing);
        let converted: Signed<32, 1> = Signed::<32, 1>::unchecked_from(last);
        self.tick_lower_lasts.replace(pool_id, converted);
        return Ok(());
    }

    // fn get_crossed_ticks(
    //     &self,
    //     pool_id: FixedBytes<32>,
    //     tick_spacing: i32,
    // ) -> (i32, i32, i32) {
    //     //let tick_lower =
    // }

    fn get_tick(&self, pool_id: FixedBytes<32>) -> Result<i32, HookError> {
        //let tick_lower =
        let pool_slot = IPoolManager::new(self.pool_manager.get()).get_slot_0(self, pool_id)?;
        return Ok(pool_slot.1);
    }

    fn get_tick_lower(&self, tick: i32, tick_spacing: i32) -> i32 {
        let mut compressed = tick / tick_spacing;
        if (tick < 0 && tick % tick_spacing != 0) {
            compressed -= 1; // round towards negative infinity
        }
        return compressed * tick_spacing;
    }
}
