// SPDX-License-Identifier: MIT
//! Hook permission flags, mirroring `Hooks.sol` in v4-core.
//!
//! Uniswap v4 decides which callbacks to invoke by reading the low 14 bits of the hook's address,
//! so a hook must be deployed to a mined address that matches the permissions it declares.

use alloy_primitives::Address;

pub const ALL_HOOK_MASK: u32 = (1 << 14) - 1;

pub const BEFORE_INITIALIZE_FLAG: u32 = 1 << 13;
pub const AFTER_INITIALIZE_FLAG: u32 = 1 << 12;
pub const BEFORE_ADD_LIQUIDITY_FLAG: u32 = 1 << 11;
pub const AFTER_ADD_LIQUIDITY_FLAG: u32 = 1 << 10;
pub const BEFORE_REMOVE_LIQUIDITY_FLAG: u32 = 1 << 9;
pub const AFTER_REMOVE_LIQUIDITY_FLAG: u32 = 1 << 8;
pub const BEFORE_SWAP_FLAG: u32 = 1 << 7;
pub const AFTER_SWAP_FLAG: u32 = 1 << 6;
pub const BEFORE_DONATE_FLAG: u32 = 1 << 5;
pub const AFTER_DONATE_FLAG: u32 = 1 << 4;
pub const BEFORE_SWAP_RETURNS_DELTA_FLAG: u32 = 1 << 3;
pub const AFTER_SWAP_RETURNS_DELTA_FLAG: u32 = 1 << 2;
pub const AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG: u32 = 1 << 1;
pub const AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG: u32 = 1 << 0;

/// The set of callbacks a hook implements, as `Hooks.Permissions` in v4-core.
///
/// Build one with [`Permissions::none`] and the builder methods, then expose it from
/// [`crate::IHooksBase::hook_permissions`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Permissions {
    pub before_initialize: bool,
    pub after_initialize: bool,
    pub before_add_liquidity: bool,
    pub after_add_liquidity: bool,
    pub before_remove_liquidity: bool,
    pub after_remove_liquidity: bool,
    pub before_swap: bool,
    pub after_swap: bool,
    pub before_donate: bool,
    pub after_donate: bool,
    pub before_swap_return_delta: bool,
    pub after_swap_return_delta: bool,
    pub after_add_liquidity_return_delta: bool,
    pub after_remove_liquidity_return_delta: bool,
}

macro_rules! setter {
    ($name:ident, $field:ident) => {
        #[must_use]
        pub const fn $name(mut self) -> Self {
            self.$field = true;
            self
        }
    };
}

impl Permissions {
    /// A hook that implements no callbacks.
    pub const fn none() -> Self {
        Self {
            before_initialize: false,
            after_initialize: false,
            before_add_liquidity: false,
            after_add_liquidity: false,
            before_remove_liquidity: false,
            after_remove_liquidity: false,
            before_swap: false,
            after_swap: false,
            before_donate: false,
            after_donate: false,
            before_swap_return_delta: false,
            after_swap_return_delta: false,
            after_add_liquidity_return_delta: false,
            after_remove_liquidity_return_delta: false,
        }
    }

    setter!(with_before_initialize, before_initialize);
    setter!(with_after_initialize, after_initialize);
    setter!(with_before_add_liquidity, before_add_liquidity);
    setter!(with_after_add_liquidity, after_add_liquidity);
    setter!(with_before_remove_liquidity, before_remove_liquidity);
    setter!(with_after_remove_liquidity, after_remove_liquidity);
    setter!(with_before_swap, before_swap);
    setter!(with_after_swap, after_swap);
    setter!(with_before_donate, before_donate);
    setter!(with_after_donate, after_donate);
    setter!(with_before_swap_return_delta, before_swap_return_delta);
    setter!(with_after_swap_return_delta, after_swap_return_delta);
    setter!(
        with_after_add_liquidity_return_delta,
        after_add_liquidity_return_delta
    );
    setter!(
        with_after_remove_liquidity_return_delta,
        after_remove_liquidity_return_delta
    );

    /// The address bits this permission set requires — the value a CREATE2 salt must be mined for.
    pub const fn flags(&self) -> u32 {
        let mut flags = 0u32;
        if self.before_initialize {
            flags |= BEFORE_INITIALIZE_FLAG;
        }
        if self.after_initialize {
            flags |= AFTER_INITIALIZE_FLAG;
        }
        if self.before_add_liquidity {
            flags |= BEFORE_ADD_LIQUIDITY_FLAG;
        }
        if self.after_add_liquidity {
            flags |= AFTER_ADD_LIQUIDITY_FLAG;
        }
        if self.before_remove_liquidity {
            flags |= BEFORE_REMOVE_LIQUIDITY_FLAG;
        }
        if self.after_remove_liquidity {
            flags |= AFTER_REMOVE_LIQUIDITY_FLAG;
        }
        if self.before_swap {
            flags |= BEFORE_SWAP_FLAG;
        }
        if self.after_swap {
            flags |= AFTER_SWAP_FLAG;
        }
        if self.before_donate {
            flags |= BEFORE_DONATE_FLAG;
        }
        if self.after_donate {
            flags |= AFTER_DONATE_FLAG;
        }
        if self.before_swap_return_delta {
            flags |= BEFORE_SWAP_RETURNS_DELTA_FLAG;
        }
        if self.after_swap_return_delta {
            flags |= AFTER_SWAP_RETURNS_DELTA_FLAG;
        }
        if self.after_add_liquidity_return_delta {
            flags |= AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG;
        }
        if self.after_remove_liquidity_return_delta {
            flags |= AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG;
        }
        flags
    }
}

/// The low 14 bits of `address`, which is what v4 reads to decide which callbacks to invoke.
pub fn address_flags(address: Address) -> u32 {
    let bytes = address.into_array();
    let low = u32::from_be_bytes([0, 0, bytes[18], bytes[19]]);
    low & ALL_HOOK_MASK
}

/// Whether `address` would have v4 invoke the callbacks in `permissions`, and nothing else.
///
/// This is the check `Hooks.validateHookPermissions` performs in the Solidity `BaseHook`
/// constructor.
pub fn is_valid_hook_address(address: Address, permissions: &Permissions) -> bool {
    address_flags(address) == permissions.flags()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flag layout is fixed by v4-core's `Hooks.sol`; these are the literal values from it.
    #[test]
    fn flags_match_hooks_sol() {
        assert_eq!(BEFORE_INITIALIZE_FLAG, 1 << 13);
        assert_eq!(AFTER_INITIALIZE_FLAG, 1 << 12);
        assert_eq!(BEFORE_ADD_LIQUIDITY_FLAG, 1 << 11);
        assert_eq!(AFTER_ADD_LIQUIDITY_FLAG, 1 << 10);
        assert_eq!(BEFORE_REMOVE_LIQUIDITY_FLAG, 1 << 9);
        assert_eq!(AFTER_REMOVE_LIQUIDITY_FLAG, 1 << 8);
        assert_eq!(BEFORE_SWAP_FLAG, 1 << 7);
        assert_eq!(AFTER_SWAP_FLAG, 1 << 6);
        assert_eq!(BEFORE_DONATE_FLAG, 1 << 5);
        assert_eq!(AFTER_DONATE_FLAG, 1 << 4);
        assert_eq!(BEFORE_SWAP_RETURNS_DELTA_FLAG, 1 << 3);
        assert_eq!(AFTER_SWAP_RETURNS_DELTA_FLAG, 1 << 2);
        assert_eq!(AFTER_ADD_LIQUIDITY_RETURNS_DELTA_FLAG, 1 << 1);
        assert_eq!(AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG, 1);
        assert_eq!(ALL_HOOK_MASK, 0x3fff);
    }

    #[test]
    fn counter_permissions_produce_the_expected_flags() {
        let permissions = Permissions::none()
            .with_before_swap()
            .with_after_swap()
            .with_before_add_liquidity()
            .with_before_remove_liquidity();

        // the value `Counter.sol` is mined against in the v4-template
        assert_eq!(permissions.flags(), 0x0ac0);
        assert_eq!(
            permissions.flags(),
            BEFORE_SWAP_FLAG
                | AFTER_SWAP_FLAG
                | BEFORE_ADD_LIQUIDITY_FLAG
                | BEFORE_REMOVE_LIQUIDITY_FLAG
        );
    }

    #[test]
    fn only_the_low_fourteen_bits_of_an_address_count() {
        // ...a0c0: high bits are ignored, 0x20c0 & 0x3fff == 0x20c0
        let address: Address = "0x00000000000000000000000000000000000020c0"
            .parse()
            .unwrap();
        assert_eq!(address_flags(address), 0x20c0);

        let noisy: Address = "0xdeadbeefdeadbeefdeadbeefdeadbeef000020c0"
            .parse()
            .unwrap();
        assert_eq!(address_flags(noisy), 0x20c0);
    }

    #[test]
    fn address_validation_matches_the_declared_permissions() {
        let permissions = Permissions::none().with_after_swap(); // 0x0040
        let good: Address = "0xdeadbeefdeadbeefdeadbeefdeadbeef00000040"
            .parse()
            .unwrap();
        let bad: Address = "0xdeadbeefdeadbeefdeadbeefdeadbeef00000041"
            .parse()
            .unwrap();

        assert!(is_valid_hook_address(good, &permissions));
        // 0x41 also enables afterRemoveLiquidityReturnDelta, which was not declared
        assert!(!is_valid_hook_address(bad, &permissions));
    }
}
