// Magical Bitcoin Library
// Written in 2020 by
//     Alekos Filini <alekos.filini@gmail.com>
//
// Copyright (c) 2020 Magical Bitcoin
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Address validation callbacks
//!
//! The typical usage of those callbacks is for displaying the newly-generated address on a
//! hardware wallet, so that the user can cross-check its correctness.
//!
//! More generally speaking though, these callbacks can also be used to "do something" every time
//! an address is generated, without necessarily checking or validating it.
//!
//! An address validator can be attached to a [`Wallet`](super::Wallet) by using the
//! [`Wallet::add_address_validator`](super::Wallet::add_address_validator) method, and
//! whenever a new address is generated (either explicitly by the user with
//! [`Wallet::get_new_address`](super::Wallet::get_new_address) or internally to create a change
//! address) all the attached validators will be polled, in sequence. All of them must complete
//! successfully to continue.
//!
//! ## Example
//!
//! ```
//! # use std::sync::Arc;
//! # use bitcoin::*;
//! # use magical_bitcoin_wallet::address_validator::*;
//! # use magical_bitcoin_wallet::database::*;
//! # use magical_bitcoin_wallet::*;
//! struct PrintAddressAndContinue;
//!
//! impl AddressValidator for PrintAddressAndContinue {
//!     fn validate(
//!         &self,
//!         script_type: ScriptType,
//!         hd_keypaths: &HDKeyPaths,
//!         script: &Script
//!     ) -> Result<(), AddressValidatorError> {
//!         let address = Address::from_script(script, Network::Testnet)
//!                           .as_ref()
//!                           .map(Address::to_string)
//!                           .unwrap_or(script.to_string());
//!         println!("New address of type {:?}: {}", script_type, address);
//!         println!("HD keypaths: {:#?}", hd_keypaths);
//!
//!         Ok(())
//!     }
//! }
//!
//! let descriptor = "wpkh(tpubD6NzVbkrYhZ4Xferm7Pz4VnjdcDPFyjVu5K4iZXQ4pVN8Cks4pHVowTBXBKRhX64pkRyJZJN5xAKj4UDNnLPb5p2sSKXhewoYx5GbTdUFWq/*)";
//! let mut wallet: OfflineWallet<_> = Wallet::new_offline(descriptor, None, Network::Testnet, MemoryDatabase::default())?;
//! wallet.add_address_validator(Arc::new(Box::new(PrintAddressAndContinue)));
//!
//! let address = wallet.get_new_address()?;
//! println!("Address: {}", address);
//! # Ok::<(), magical_bitcoin_wallet::Error>(())
//! ```

use std::fmt;

use bitcoin::Script;

use crate::descriptor::HDKeyPaths;
use crate::types::ScriptType;

/// Errors that can be returned to fail the validation of an address
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressValidatorError {
    UserRejected,
    ConnectionError,
    TimeoutError,
    InvalidScript,
    Message(String),
}

impl fmt::Display for AddressValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AddressValidatorError {}

/// Trait to build address validators
///
/// All the address validators attached to a wallet with [`Wallet::add_address_validator`](super::Wallet::add_address_validator) will be polled
/// every time an address (external or internal) is generated by the wallet. Errors returned in the
/// validator will be propagated up to the original caller that triggered the address generation.
///
/// For a usage example see [this module](crate::address_validator)'s documentation.
pub trait AddressValidator {
    /// Validate or inspect an address
    fn validate(
        &self,
        script_type: ScriptType,
        hd_keypaths: &HDKeyPaths,
        script: &Script,
    ) -> Result<(), AddressValidatorError>;
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use crate::wallet::test::{get_funded_wallet, get_test_wpkh};
    use crate::wallet::TxBuilder;

    struct TestValidator;
    impl AddressValidator for TestValidator {
        fn validate(
            &self,
            _script_type: ScriptType,
            _hd_keypaths: &HDKeyPaths,
            _script: &bitcoin::Script,
        ) -> Result<(), AddressValidatorError> {
            Err(AddressValidatorError::InvalidScript)
        }
    }

    #[test]
    #[should_panic(expected = "InvalidScript")]
    fn test_address_validator_external() {
        let (mut wallet, _, _) = get_funded_wallet(get_test_wpkh());
        wallet.add_address_validator(Arc::new(Box::new(TestValidator)));

        wallet.get_new_address().unwrap();
    }

    #[test]
    #[should_panic(expected = "InvalidScript")]
    fn test_address_validator_internal() {
        let (mut wallet, descriptors, _) = get_funded_wallet(get_test_wpkh());
        wallet.add_address_validator(Arc::new(Box::new(TestValidator)));

        let addr = testutils!(@external descriptors, 10);
        wallet
            .create_tx(TxBuilder::with_recipients(vec![(addr, 25_000)]))
            .unwrap();
    }
}
