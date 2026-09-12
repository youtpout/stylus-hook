// SPDX-License-Identifier: MIT
//! Predicting and mining the address a `StylusDeployer` CREATE2 deployment lands on.
//!
//! v4 reads a hook's permissions out of the low 14 bits of its address, so a hook must be deployed
//! to a mined one. `cargo stylus deploy` routes through StylusDeployer, which uses CREATE2 on a
//! non-zero salt, so no Solidity deployer is needed.

use alloy_primitives::{address, keccak256, Address, B256, U256};

/// The canonical `StylusDeployer`, which is also `cargo stylus deploy`'s default.
pub const STYLUS_DEPLOYER: Address = address!("cEcba2F1DC234f70Dd89F2041029807F8D03A990");

/// `bytes4(keccak256("stylus_constructor()"))` — the selector `StylusDeployer` calls the freshly
/// deployed contract with, followed by the ABI-encoded constructor arguments.
pub const STYLUS_CONSTRUCTOR_SELECTOR: [u8; 4] = [0x55, 0x85, 0x25, 0x8d];

/// The init data `StylusDeployer` will call the contract with.
///
/// `None` means the contract has no constructor and the deployer will not call it at all. A
/// constructor that takes no arguments is *not* the same thing: it still gets called, with the bare
/// selector, and that changes the salt.
pub fn init_data(encoded_constructor_args: Option<&[u8]>) -> Vec<u8> {
    match encoded_constructor_args {
        None => Vec::new(),
        Some(args) => {
            let mut data = Vec::with_capacity(4 + args.len());
            data.extend_from_slice(&STYLUS_CONSTRUCTOR_SELECTOR);
            data.extend_from_slice(args);
            data
        }
    }
}

/// `StylusDeployer.initSalt` — the deployer hashes the caller's salt together with the init data so
/// that the same address always implies the same initialisation.
pub fn init_salt(user_salt: B256, init_data: &[u8]) -> B256 {
    let mut buf = Vec::with_capacity(32 + init_data.len());
    buf.extend_from_slice(user_salt.as_slice());
    buf.extend_from_slice(init_data);
    keccak256(buf)
}

/// The address `StylusDeployer` will CREATE2 the contract to for a given user salt.
pub fn predict_address(
    deployer: Address,
    user_salt: B256,
    init_data: &[u8],
    init_code_hash: B256,
) -> Address {
    let salt = init_salt(user_salt, init_data);
    deployer.create2(salt, init_code_hash)
}

/// Searches for a salt whose *plain* CREATE2 address carries exactly `flags` in its low 14 bits.
///
/// For an ordinary factory rather than `StylusDeployer`: the salt is used verbatim and constructor
/// arguments are already appended to `init_code`. This is what a Solidity hook needs.
pub fn mine_create2(
    deployer: Address,
    init_code_hash: B256,
    flags: u32,
    max_attempts: u64,
) -> Option<MinedSalt> {
    for attempt in 0..max_attempts {
        let salt = B256::from(U256::from(attempt));
        let address = deployer.create2(salt, init_code_hash);
        if stylus_uniswap_v4::permissions::address_flags(address) == flags {
            return Some(MinedSalt {
                salt,
                address,
                attempts: attempt + 1,
            });
        }
    }
    None
}

/// What [`mine`] found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinedSalt {
    pub salt: B256,
    pub address: Address,
    pub attempts: u64,
}

/// Searches for a salt whose deployment address carries exactly `flags` in its low 14 bits.
///
/// Roughly `2^14` attempts on average, so this returns in well under a second.
pub fn mine(
    deployer: Address,
    init_data: &[u8],
    init_code_hash: B256,
    flags: u32,
    max_attempts: u64,
) -> Option<MinedSalt> {
    for attempt in 0..max_attempts {
        // salt 0 makes StylusDeployer fall back to CREATE, which is not deterministic
        let salt = B256::from(U256::from(attempt + 1));
        let address = predict_address(deployer, salt, init_data, init_code_hash);
        if stylus_uniswap_v4::permissions::address_flags(address) == flags {
            return Some(MinedSalt {
                salt,
                address,
                attempts: attempt + 1,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;
    use stylus_uniswap_v4::{permissions::address_flags, Permissions};

    #[test]
    fn constructor_selector_matches_the_deployer() {
        // sol! { function stylus_constructor(); } in stylus-tools
        assert_eq!(
            &keccak256(b"stylus_constructor()")[..4],
            STYLUS_CONSTRUCTOR_SELECTOR
        );
    }

    #[test]
    fn init_salt_hashes_the_salt_with_the_init_data() {
        // StylusDeployer.initSalt is keccak256(abi.encodePacked(salt, initData))
        let salt = B256::with_last_byte(7);
        let data = hex!("deadbeef");
        let mut expected = Vec::new();
        expected.extend_from_slice(salt.as_slice());
        expected.extend_from_slice(&data);
        assert_eq!(init_salt(salt, &data), keccak256(expected));
    }

    #[test]
    fn no_constructor_means_no_init_data() {
        assert!(init_data(None).is_empty());
    }

    #[test]
    fn a_constructor_taking_nothing_still_gets_a_selector() {
        // the distinction that matters: no constructor is not the same as a constructor with no
        // arguments, and getting it wrong mines a salt for the wrong address
        assert_eq!(init_data(Some(&[])), STYLUS_CONSTRUCTOR_SELECTOR.to_vec());
        assert_ne!(init_data(Some(&[])), init_data(None));
    }

    #[test]
    fn init_data_is_the_selector_then_the_arguments() {
        let args = [0u8; 32];
        let data = init_data(Some(&args));
        assert_eq!(data.len(), 36);
        assert_eq!(&data[..4], STYLUS_CONSTRUCTOR_SELECTOR);
    }

    #[test]
    fn mines_an_address_carrying_the_requested_flags() {
        let flags = Permissions::none()
            .with_before_swap()
            .with_after_swap()
            .with_before_add_liquidity()
            .with_before_remove_liquidity()
            .flags();
        let init_code_hash = keccak256(b"a stylus contract's init code");
        let data = init_data(Some(&[0x11u8; 32]));

        let found = mine(STYLUS_DEPLOYER, &data, init_code_hash, flags, 1_000_000)
            .expect("a salt should exist within a million tries");

        assert_eq!(address_flags(found.address), flags);
        // the reported address is reproducible from the reported salt
        assert_eq!(
            predict_address(STYLUS_DEPLOYER, found.salt, &data, init_code_hash),
            found.address
        );
    }

    #[test]
    fn different_init_data_gives_a_different_address() {
        let init_code_hash = keccak256(b"code");
        let salt = B256::with_last_byte(1);
        let a = predict_address(
            STYLUS_DEPLOYER,
            salt,
            &init_data(Some(&[0x11u8; 32])),
            init_code_hash,
        );
        let b = predict_address(
            STYLUS_DEPLOYER,
            salt,
            &init_data(Some(&[0x22u8; 32])),
            init_code_hash,
        );
        assert_ne!(a, b);
    }
}
