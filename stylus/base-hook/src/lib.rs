// SPDX-License-Identifier: MIT
//! Uniswap v4 hooks in pure Rust, for Arbitrum Stylus.
//!
//! The Stylus counterpart of `BaseHook.sol`: the v4 value types, the permission flags v4 reads out
//! of a hook's address, and the ten `IHooks` callbacks. `README.md` walks through writing one.

#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

extern crate alloc;

pub mod hooks;
pub mod permissions;
pub mod pool_manager;
pub mod types;

pub use hooks::{selector, HookConfig, HookGuards, IHooks, IUnlockCallback, NO_DELTA};
pub use permissions::Permissions;
pub use pool_manager::PoolManagerCalls;
pub use stylus_uniswap_v4_macros::guarded_hooks;
pub use types::{
    amount0, amount1, lp_fee, specified_delta, to_balance_delta, to_before_swap_delta,
    unspecified_delta, BalanceDelta, BeforeSwapDelta, Currency, ModifyLiquidityParams, PoolKey,
    SwapParams, ZERO_DELTA,
};
