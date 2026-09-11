# Writing a hook

`stylus-uniswap-v4` is `BaseHook.sol` for Stylus: the v4 value types, the permission flags v4 reads
out of a hook's address, and the ten `IHooks` callbacks — with no Solidity anywhere.

[`native-counter`](../native-counter/src/lib.rs) is the smallest complete example. Four steps.

## 1. Storage and the entry point

```rust
#[storage]
#[entrypoint]
pub struct Counter {
    before_swap_count: StorageMap<FixedBytes<32>, StorageU256>,
}
```

The pool manager is not a field — [`build.rs`](../native-counter/build.rs) bakes it in as a `const`:

```rust
include!(concat!(env!("OUT_DIR"), "/pool_manager.rs"));   // gives you POOL_MANAGER
```

## 2. Declare the pool manager and the callbacks

`permissions()` has to match the callbacks you implement, because v4 reads them out of the low 14
bits of the hook's address and will never call one the address does not advertise.

```rust
impl HookConfig for Counter {
    fn pool_manager(&self) -> Address {
        POOL_MANAGER
    }

    fn permissions(&self) -> Permissions {
        Permissions::none().with_before_swap().with_after_swap()
    }
}
```

## 3. Check both addresses in the constructor

`validate_hook_address` fails unless the deployed address carries exactly the declared flags — which
is why a plain `cargo stylus deploy` cannot deploy a hook, and [step 5](#5-deploy-at-a-mined-address)
mines one first.

It also takes the pool manager, only to compare it against the constant — so a wrong `$POOL_MANAGER`
fails the deployment instead of producing a hook that rejects every call.

```rust
#[public]
#[implements(IHooks)]
impl Counter {
    #[constructor]
    pub fn constructor(&mut self, pool_manager: Address) -> Result<(), Vec<u8>> {
        if pool_manager != POOL_MANAGER {
            return Err(PoolManagerMismatch { baked: POOL_MANAGER, given: pool_manager }.abi_encode());
        }
        HookGuards::validate_hook_address(self)
    }
}
```

## 4. Implement the callbacks

Write only the ones `permissions()` declares; the rest revert with `HookNotImplemented` on their own.
Each returns its own selector, which v4 checks.

`#[guarded_hooks]` goes above `#[public]`. It inserts the caller and pool-key checks into every
method below it, so you never write them — the Rust counterpart of Solidity's
`external onlyPoolManager`.

```rust
#[guarded_hooks]
#[public]
impl IHooks for Counter {
    fn before_swap(
        &mut self,
        _sender: Address,
        key: PoolKey,
        _params: SwapParams,
        _hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>> {
        // the caller is the pool manager and `key` names this hook: both already checked
        let mut slot = self.before_swap_count.setter(key.to_id());
        let count = slot.get();
        slot.set(count + U256::from(1));
        Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
    }
}
```

> Put `#[guarded_hooks]` **only** on the `impl IHooks` block. A hook's own entry points — an order
> book, a claim — must not require the pool manager, and the attribute would lock them out.

## 5. Deploy at a mined address

```bash
export POOL_MANAGER=0xYourPoolManager        # baked in at build time, so set it first
cargo stylus get-initcode --contract stylus-native-counter | tail -1 > initcode.hex
cargo run -p stylus-hook-miner -- \
  --initcode-file initcode.hex \
  --permissions before-swap,after-swap \
  --constructor-signature 'constructor(address pool_manager)' \
  --constructor-args $POOL_MANAGER
# then deploy with the salt it prints
cargo stylus deploy --contract stylus-native-counter --deployer-salt 0x… \
  --constructor-args $POOL_MANAGER
```

A contract too large for one code fragment cannot use `get-initcode`; see `deploy_fragmented_hook` in
[`bench-lib.bash`](../../bench-lib.bash) for the way round it.

## Calling back into the pool manager

A hook that returns a non-zero delta has to settle its own books before the lock closes.
[`PoolManagerCalls`](src/pool_manager.rs) wraps `unlock`, `swap`, `take`, `settle`, `sync`, `extsload`
and the rest; [`native-twamm`](../native-twamm/src/lib.rs) uses them for real.

## Build settings that matter

Two, both in this repository's `Cargo.toml` and `Stylus.toml` files, and both worth copying:

- `opt-level = 3` — the default `"s"` compiles for size and costs roughly 2× the gas.
- `build.rs` for the pool manager, as above.
- `--llvm-memory-copy-fill-lowering` in the wasm-opt flags — without it, `opt-level` 2 or 3 emits a
  section ArbOS refuses to activate, and the failure only appears at deployment.
