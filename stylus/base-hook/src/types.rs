// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Uniswap v4 value types a hook receives, mapped onto their Solidity ABI encodings.

use alloy_primitives::{Address, FixedBytes, I256};
use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

/// A v4 currency is an address, with the zero address meaning native ETH.
pub type Currency = Address;

sol! {
    /// `PoolKey` — uniquely identifies a v4 pool. ABI: `(address,address,uint24,int24,address)`.
    #[derive(Debug, AbiType)]
    struct PoolKey {
        address currency0;
        address currency1;
        uint24 fee;
        int24 tickSpacing;
        address hooks;
    }

    /// Parameter struct for `ModifyLiquidity` pool operations.
    #[derive(Debug, AbiType)]
    struct ModifyLiquidityParams {
        int24 tickLower;
        int24 tickUpper;
        int256 liquidityDelta;
        bytes32 salt;
    }

    /// Parameter struct for `Swap` pool operations.
    #[derive(Debug, AbiType)]
    struct SwapParams {
        bool zeroForOne;
        int256 amountSpecified;
        uint160 sqrtPriceLimitX96;
    }
}

impl PoolKey {
    /// `PoolId` is `keccak256(abi.encode(poolKey))`, matching `PoolIdLibrary.toId`.
    pub fn to_id(&self) -> FixedBytes<32> {
        use alloy_sol_types::SolValue;
        alloy_primitives::keccak256(self.abi_encode())
    }
}

/// Two `int128` amounts packed into an `int256`, as `BalanceDelta` in v4-core.
///
/// The upper 128 bits hold `amount0` and the lower 128 bits hold `amount1`.
pub type BalanceDelta = I256;

/// Two `int128` amounts packed into an `int256`, as `BeforeSwapDelta` in v4-core.
///
/// The upper 128 bits hold the delta in the specified currency and the lower 128 bits the delta in
/// the unspecified one.
pub type BeforeSwapDelta = I256;

/// Reads the upper `int128` out of a packed delta — `amount0` for a `BalanceDelta`.
pub fn delta_amount0(delta: I256) -> i128 {
    let bytes = delta.to_be_bytes::<32>();
    let mut hi = [0u8; 16];
    hi.copy_from_slice(&bytes[..16]);
    i128::from_be_bytes(hi)
}

/// Reads the lower `int128` out of a packed delta — `amount1` for a `BalanceDelta`.
pub fn delta_amount1(delta: I256) -> i128 {
    let bytes = delta.to_be_bytes::<32>();
    let mut lo = [0u8; 16];
    lo.copy_from_slice(&bytes[16..]);
    i128::from_be_bytes(lo)
}

/// Packs two `int128` amounts into a delta, as `toBalanceDelta` in v4-core.
pub fn to_balance_delta(amount0: i128, amount1: i128) -> I256 {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(&amount0.to_be_bytes());
    bytes[16..].copy_from_slice(&amount1.to_be_bytes());
    I256::from_be_bytes(bytes)
}

/// A delta of zero, as `BalanceDeltaLibrary.ZERO_DELTA`.
pub const ZERO_DELTA: I256 = I256::ZERO;

/// Re-exported so hook crates do not need to depend on `alloy_primitives` directly.
pub use alloy_primitives::aliases::{I24, U160, U24};
