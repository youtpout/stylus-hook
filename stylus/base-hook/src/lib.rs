// SPDX-License-Identifier: MIT OR Apache-2.0
//! Uniswap v4 hooks in pure Rust, for Arbitrum Stylus.
//!
//! This crate is the Stylus counterpart of `BaseHook.sol`: the v4 value types, the permission flags
//! v4 reads out of a hook's address, and the ten `IHooks` callbacks with the same
//! "revert unless implemented" semantics — with no Solidity in the dependency graph.
//!
//! ```ignore
//! #[storage]
//! #[entrypoint]
//! pub struct MyHook { pool_manager: StorageAddress }
//!
//! impl HookConfig for MyHook {
//!     fn pool_manager(&self) -> Address { self.pool_manager.get() }
//!     fn permissions() -> Permissions { Permissions::none().with_after_swap() }
//! }
//!
//! #[public]
//! #[implements(IHooks)]
//! impl MyHook {}
//!
//! #[public]
//! impl IHooks for MyHook {
//!     fn after_swap(&mut self, ..) -> Result<(FixedBytes<4>, i128), Vec<u8>> {
//!         self.require_pool_manager()?;
//!         Ok((selector::AFTER_SWAP, 0))
//!     }
//! }
//! ```

#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

extern crate alloc;

pub mod hooks;
pub mod permissions;
pub mod pool_manager;
pub mod types;

pub use hooks::{selector, HookConfig, HookGuards, IHooks, NO_DELTA};
pub use permissions::Permissions;
pub use pool_manager::PoolManagerCalls;
pub use types::{
    delta_amount0, delta_amount1, to_balance_delta, BalanceDelta, BeforeSwapDelta, Currency,
    ModifyLiquidityParams, PoolKey, SwapParams, ZERO_DELTA,
};
