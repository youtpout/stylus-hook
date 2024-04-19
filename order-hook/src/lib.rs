// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use alloy_primitives::{Address, FixedBytes, Signed, I32, U128, U256};
use alloy_sol_types::sol;
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

#[derive(SolidityError)]
pub enum HookError {
    ZeroLiquidity(ZeroLiquidity),
    InRange(InRange),
    CrossedRange(CrossedRange),
    Filled(Filled),
    NotFilled(NotFilled),
    NotPoolManagerToken(NotPoolManagerToken),
}

/// Simplifies the result type for the contract's methods.
type Result<T, E = HookError> = core::result::Result<T, E>;

/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl LimitOrder {
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

    fn get_tick_lower(&self, tick: i32, tick_spacing: i32) -> i32 {
        let mut compressed = tick / tick_spacing;
        if (tick < 0 && tick % tick_spacing != 0) {
            compressed -= 1; // round towards negative infinity
        }
        return compressed * tick_spacing;
    }
}
