// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::{vec};
use alloy_primitives::{U256, FixedBytes};
use stylus_sdk::{
    prelude::*
};

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct Counter {
        mapping(bytes32 => uint256) before_swap_count;
        mapping(bytes32 => uint256) after_swap_count;
        mapping(bytes32 => uint256) before_add_liquidity_count;
        mapping(bytes32 => uint256) before_remove_liquidity_count;
    }    
}

// copy from IHooks and BaseHooks
sol!{
    error NotPoolManager();
    error NotSelf();
    error InvalidPool();
    error LockFailure();
    error HookNotImplemented();

    function beforeInitialize(address sender, PoolKey calldata key, uint160 sqrtPriceX96, bytes calldata hookData)
        external
        returns (bytes4);

    function afterInitialize(
        address sender,
        PoolKey calldata key,
        uint160 sqrtPriceX96,
        int24 tick,
        bytes calldata hookData
    ) external returns (bytes4);

    function beforeAddLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        bytes calldata hookData
    ) external returns (bytes4);

    function afterAddLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        BalanceDelta delta,
        bytes calldata hookData
    ) external returns (bytes4);

    function beforeRemoveLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        bytes calldata hookData
    ) external returns (bytes4);

    function afterRemoveLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        BalanceDelta delta,
        bytes calldata hookData
    ) external returns (bytes4);

    function beforeSwap(
        address sender,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        bytes calldata hookData
    ) external returns (bytes4);

    function afterSwap(
        address sender,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        BalanceDelta delta,
        bytes calldata hookData
    ) external returns (bytes4);

    function beforeDonate(
        address sender,
        PoolKey calldata key,
        uint256 amount0,
        uint256 amount1,
        bytes calldata hookData
    ) external returns (bytes4);

    function afterDonate(
        address sender,
        PoolKey calldata key,
        uint256 amount0,
        uint256 amount1,
        bytes calldata hookData
    ) external returns (bytes4);
}


/// Declare that `Counter` is a contract with the following external methods.
#[external]
impl Counter {

    /// Gets the number from storage.
    pub fn before_swap_count(&self,  pool_id :FixedBytes<32>) -> U256 {
        self.before_swap_count.get(pool_id)
    }

    pub fn after_swap_count(&self, pool_id :FixedBytes<32>) -> U256 {
        self.after_swap_count.get(pool_id)
    }


    pub fn before_add_liquidity_count(&self, pool_id :FixedBytes<32>) -> U256 {
        self.before_add_liquidity_count.get(pool_id)
    }


    pub fn before_remove_liquidity_count(&self, pool_id :FixedBytes<32>) -> U256 {
        self.before_remove_liquidity_count.get(pool_id)
    }

    pub fn get_hook_permissions(&self) -> (bool,bool,bool,bool,bool,bool,bool,bool,bool,bool) {
        // doesn't support struct in return so use tuple
       let permissions =  (
         false,
         false,
         true,
         false,
         true,
         false,
         true,
         true,
         false,
         false
       );

        permissions
    }
}
