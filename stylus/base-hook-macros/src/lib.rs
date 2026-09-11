// SPDX-License-Identifier: MIT OR Apache-2.0
//! The attribute that gives a Stylus hook `BaseHook.sol`'s guarantee.
//!
//! Solidity inherits `onlyPoolManager` from an abstract contract. Rust has no abstract types, and
//! the two obvious substitutes both fail on Stylus: a trait cannot hold the guard because that makes
//! it non-dyn-compatible, which `#[implements]` requires, and a declarative macro has to write all
//! ten entry points, which costs about 30 % of a contract's size.
//!
//! A procedural macro has neither problem. It edits the callbacks the hook already wrote, so
//! `#[public]` sees ordinary code and nothing is emitted for callbacks that do not exist.

use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ItemImpl, Pat};

/// Inserts `require_pool_manager`, and `require_valid_pool` where the method takes a `PoolKey`, at
/// the top of every method in an `impl IHooks` block.
///
/// A method that already guards itself is left alone, so this is safe to add to existing code.
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
