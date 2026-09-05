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

    /// The v4 singleton, as far as a hook needs it.
    ///
    /// Declared in the same `sol!` block as the structs above so the generated calls use exactly
    /// those types. `PoolManager` in [`crate::pool_manager`] wraps these in something callable.
    interface IPoolManager {
        function unlock(bytes calldata data) external returns (bytes memory);

        function initialize(PoolKey memory key, uint160 sqrtPriceX96) external returns (int24 tick);

        function modifyLiquidity(
            PoolKey memory key,
            ModifyLiquidityParams memory params,
            bytes calldata hookData
        ) external returns (int256 callerDelta, int256 feesAccrued);

        function swap(PoolKey memory key, SwapParams memory params, bytes calldata hookData)
            external
            returns (int256 swapDelta);

        function donate(PoolKey memory key, uint256 amount0, uint256 amount1, bytes calldata hookData)
            external
            returns (int256 delta);

        function sync(address currency) external;

        function take(address currency, address to, uint256 amount) external;

        function settle() external payable returns (uint256 paid);

        function settleFor(address recipient) external payable returns (uint256 paid);

        function clear(address currency, uint256 amount) external;

        function mint(address to, uint256 id, uint256 amount) external;

        function burn(address from, uint256 id, uint256 amount) external;

        function updateDynamicLPFee(PoolKey memory key, uint24 newDynamicLPFee) external;

        function extsload(bytes32 slot) external view returns (bytes32 value);

        function exttload(bytes32 slot) external view returns (bytes32 value);
    }
}

impl PoolKey {
    /// `PoolId` is `keccak256(abi.encode(poolKey))`, matching `PoolIdLibrary.toId`.
    pub fn to_id(&self) -> FixedBytes<32> {
        use alloy_sol_types::SolValue;
        alloy_primitives::keccak256(self.abi_encode())
    }
}

/// Two `int128` amounts packed into an `int256`, as `BalanceDelta` in v4-core: `amount0` in the
/// upper 128 bits, `amount1` in the lower ones. Negative means the hook owes the pool manager.
pub type BalanceDelta = I256;

/// The same packing, as `BeforeSwapDelta` in v4-core, but the two halves mean something else: the
/// delta in the swap's *specified* currency above, in the *unspecified* one below.
pub type BeforeSwapDelta = I256;

/// A delta of zero, as `BalanceDeltaLibrary.ZERO_DELTA`.
pub const ZERO_DELTA: I256 = I256::ZERO;

fn pack(upper: i128, lower: i128) -> I256 {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(&upper.to_be_bytes());
    bytes[16..].copy_from_slice(&lower.to_be_bytes());
    I256::from_be_bytes(bytes)
}

fn upper(delta: I256) -> i128 {
    let bytes = delta.to_be_bytes::<32>();
    let mut half = [0u8; 16];
    half.copy_from_slice(&bytes[..16]);
    i128::from_be_bytes(half)
}

fn lower(delta: I256) -> i128 {
    let bytes = delta.to_be_bytes::<32>();
    let mut half = [0u8; 16];
    half.copy_from_slice(&bytes[16..]);
    i128::from_be_bytes(half)
}

/// Packs two amounts, as `toBalanceDelta` in v4-core.
pub fn to_balance_delta(amount0: i128, amount1: i128) -> BalanceDelta {
    pack(amount0, amount1)
}

/// `BalanceDeltaLibrary.amount0`.
pub fn amount0(delta: BalanceDelta) -> i128 {
    upper(delta)
}

/// `BalanceDeltaLibrary.amount1`.
pub fn amount1(delta: BalanceDelta) -> i128 {
    lower(delta)
}

/// Packs the two halves of a `beforeSwap` return, as `toBeforeSwapDelta` in v4-core.
pub fn to_before_swap_delta(specified: i128, unspecified: i128) -> BeforeSwapDelta {
    pack(specified, unspecified)
}

/// `BeforeSwapDeltaLibrary.getSpecifiedDelta`.
pub fn specified_delta(delta: BeforeSwapDelta) -> i128 {
    upper(delta)
}

/// `BeforeSwapDeltaLibrary.getUnspecifiedDelta`.
pub fn unspecified_delta(delta: BeforeSwapDelta) -> i128 {
    lower(delta)
}

/// LP fee encoding, mirroring `LPFeeLibrary` in v4-core.
pub mod lp_fee {
    use alloy_primitives::aliases::U24;

    /// A pool initialised with this fee lets its hook set the fee at runtime.
    pub const DYNAMIC_FEE_FLAG: u32 = 0x800000;
    /// Set on the `uint24` a `beforeSwap` returns to override the LP fee for that swap only.
    pub const OVERRIDE_FEE_FLAG: u32 = 0x400000;
    /// 100 %, in hundredths of a basis point.
    pub const MAX_LP_FEE: u32 = 1_000_000;

    /// Whether the pool was initialised as a dynamic-fee pool.
    pub fn is_dynamic(fee: U24) -> bool {
        fee.to::<u32>() == DYNAMIC_FEE_FLAG
    }

    /// Whether this `uint24` carries the override flag.
    pub fn is_override(fee: U24) -> bool {
        fee.to::<u32>() & OVERRIDE_FEE_FLAG != 0
    }

    /// Tags a fee so that returning it from `beforeSwap` overrides the pool's LP fee for that swap.
    ///
    /// Returns `None` above [`MAX_LP_FEE`], which v4 rejects.
    pub fn with_override(fee: u32) -> Option<U24> {
        if fee > MAX_LP_FEE {
            return None;
        }
        Some(U24::from(fee | OVERRIDE_FEE_FLAG))
    }

    /// Strips the override flag, leaving the fee itself.
    pub fn without_override(fee: U24) -> U24 {
        U24::from(fee.to::<u32>() & !OVERRIDE_FEE_FLAG)
    }
}

/// Re-exported so hook crates do not need to depend on `alloy_primitives` directly.
pub use alloy_primitives::aliases::{I24, U160, U24};

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    /// The same literals `uniswap/test/DeltaEncoding.t.sol` asserts against v4-core itself. If the
    /// encoding ever changes, both tests fail.
    #[test]
    fn balance_delta_packing_matches_v4_core() {
        assert_eq!(
            to_balance_delta(-5, 7).to_be_bytes::<32>(),
            hex!("fffffffffffffffffffffffffffffffb00000000000000000000000000000007")
        );
        assert_eq!(
            to_balance_delta(1_000_000_000_000_000_000, -2_000_000_000_000_000_000)
                .to_be_bytes::<32>(),
            hex!("00000000000000000de0b6b3a7640000ffffffffffffffffe43e9298b1380000")
        );
        assert_eq!(
            to_before_swap_delta(-11, 13).to_be_bytes::<32>(),
            hex!("fffffffffffffffffffffffffffffff50000000000000000000000000000000d")
        );
    }

    #[test]
    fn deltas_round_trip_through_both_halves() {
        assert_eq!(amount0(to_balance_delta(-5, 7)), -5);
        assert_eq!(amount1(to_balance_delta(-5, 7)), 7);
        assert_eq!(amount0(to_balance_delta(7, -5)), 7);
        assert_eq!(amount1(to_balance_delta(7, -5)), -5);

        let extremes = to_balance_delta(i128::MIN, i128::MAX);
        assert_eq!(amount0(extremes), i128::MIN);
        assert_eq!(amount1(extremes), i128::MAX);

        let before = to_before_swap_delta(-11, 13);
        assert_eq!(specified_delta(before), -11);
        assert_eq!(unspecified_delta(before), 13);
    }

    #[test]
    fn lp_fee_flags_match_v4_core() {
        use alloy_primitives::aliases::U24;
        assert_eq!(lp_fee::DYNAMIC_FEE_FLAG, 0x800000);
        assert_eq!(lp_fee::OVERRIDE_FEE_FLAG, 0x400000);
        assert_eq!(lp_fee::MAX_LP_FEE, 1_000_000);

        let overridden = lp_fee::with_override(3000).unwrap();
        assert!(lp_fee::is_override(overridden));
        assert_eq!(lp_fee::without_override(overridden), U24::from(3000u32));

        assert!(lp_fee::is_dynamic(U24::from(lp_fee::DYNAMIC_FEE_FLAG)));
        assert!(!lp_fee::is_dynamic(U24::from(3000u32)));
        // v4 rejects anything above 100 %
        assert!(lp_fee::with_override(lp_fee::MAX_LP_FEE + 1).is_none());
    }
}
