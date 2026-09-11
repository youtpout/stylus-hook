// SPDX-License-Identifier: MIT OR Apache-2.0
//! The attribute that gives a Stylus hook `BaseHook.sol`'s guarantees.
//!
//! # Why a procedural macro
//!
//! Solidity's `BaseHook` is an abstract contract: it owns the `external onlyPoolManager` entry
//! points and the hook overrides internal `_beforeSwap`, so the guard cannot be forgotten. Rust has
//! no abstract types, and its three substitutes each fail here:
//!
//! * **A trait with default bodies** cannot hold the guard: the guard needs the host, `HostAccess`
//!   carries an associated `Host` type, and a `where Self: HookGuards` bound on a trait method makes
//!   the trait non-dyn-compatible — which Stylus's `#[implements]` requires.
//! * **A declarative macro** that writes the whole `#[public] impl` works, but it has to emit all
//!   ten entry points, which measured about 30 % of a contract's size and pushed two of this
//!   repository's hooks over a code fragment. Emitting only the declared ones needs a token-munching
//!   macro, and that trips a hygiene bug in `stylus-proc`: `#[public]` binds `result` and references
//!   it re-spanned to the method's output span, so a method arriving through a `$($out:tt)*` capture
//!   fails with `cannot find value result`.
//! * **A procedural macro** has neither problem. It rewrites the methods the hook already wrote,
//!   emitting fresh tokens at one span, so `#[public]` sees ordinary hand-written code — and it adds
//!   nothing for callbacks the hook does not implement, because it only touches what is there.
//!
//! # Use
//!
//! ```ignore
//! #[guarded_hooks]
//! #[public]
//! impl IHooks for MyHook {
//!     fn before_swap(&mut self, _sender: Address, key: PoolKey, ..)
//!         -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Vec<u8>>
//!     {
//!         // the caller is the pool manager and `key` names this hook: both already checked
//!         Ok((selector::BEFORE_SWAP, ZERO_DELTA, U24::ZERO))
//!     }
//! }
//! ```
//!
//! Attributes apply outside in, so this runs first and hands `#[public]` an impl whose every method
//! opens with the guards.

use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ItemImpl, Pat};

/// Inserts `BaseHook.sol`'s two guards at the top of every method in an `impl IHooks` block.
///
/// `require_pool_manager` goes into every method. `require_valid_pool` goes into those that take a
/// `PoolKey`, which is all ten v4 callbacks — it is conditional only so the attribute stays useful
/// on an impl that grows a helper.
///
/// A method already calling a guard is left alone, so applying this to existing code is a no-op
/// rather than a double check.
#[proc_macro_attribute]
pub fn guarded_hooks(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = match syn::parse::<ItemImpl>(item.clone()) {
        Ok(parsed) => parsed,
        Err(err) => {
            let mut out: TokenStream = err.to_compile_error().into();
            out.extend(item);
            return out;
        }
    };

    for entry in &mut input.items {
        let ImplItem::Fn(method) = entry else {
            continue;
        };

        // Leave a method that already guards itself untouched.
        let body = quote!(#method).to_string();
        if body.contains("require_pool_manager") {
            continue;
        }

        let key = key_argument(method);
        let pool = key.map(|ident| {
            quote! {
                ::stylus_uniswap_v4::hooks::HookGuards::require_valid_pool(self, &#ident)?;
            }
        });
        let guards = quote! {
            ::stylus_uniswap_v4::hooks::HookGuards::require_pool_manager(self)?;
            #pool
        };
        let statements = match syn::parse2::<syn::Block>(quote!({ #guards })) {
            Ok(block) => block.stmts,
            Err(err) => return err.to_compile_error().into(),
        };
        method.block.stmts.splice(0..0, statements);
    }

    quote!(#input).into()
}

/// The name of the method's `PoolKey` argument, if it has one.
fn key_argument(method: &syn::ImplItemFn) -> Option<syn::Ident> {
    method.sig.inputs.iter().find_map(|arg| {
        let FnArg::Typed(typed) = arg else {
            return None;
        };
        let last = match &*typed.ty {
            syn::Type::Path(path) => path.path.segments.last()?.ident.to_string(),
            _ => return None,
        };
        if last != "PoolKey" {
            return None;
        }
        match &*typed.pat {
            // `_key: PoolKey` cannot be checked against, and says the hook does not want it.
            Pat::Ident(ident) if !ident.ident.to_string().starts_with('_') => {
                Some(ident.ident.clone())
            }
            _ => None,
        }
    })
}
