//! Wallet utility to build & sign transactions on every cosmos-sdk based network

// Includes code originally from ibc-rs:
// <https://github.com/informalsystems/ibc-rs>
// Copyright © 2020 Informal Systems Inc.
// Licensed under the Apache 2.0 license

use bech32::{ToBase32, Variant::Bech32};
use bip39::{Language, Mnemonic, Seed, MnemonicType};
use bitcoin::{
    network::constants::Network,
    secp256k1::Secp256k1,
    util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey},
    PublicKey,
};
use hdpath::StandardHDPath;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use ripemd160::Ripemd160;
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use crate::WalletError;

/// Keychain contains a pair of Secp256k1 keys.
#[derive(Clone)]
pub struct Keychain {
    pub ext_public_key: ExtendedPubKey,
    pub ext_private_key: ExtendedPrivKey,
}

/// MnemonicWallet is a facility used to sign transactions with a BIP-32 mnemonic.
#[derive(Clone)]
pub struct MnemonicWallet {
    mnemonic: Mnemonic,
    derivation_path: String,
    keychain: Keychain,
}

impl MnemonicWallet {
    /// Derive a Wallet from the given mnemonic_words and derivation path.
    pub fn new(mnemonic_words: &str, derivation_path: &str) -> Result<MnemonicWallet, WalletError> {
        // Create mnemonic and generate seed from it
        let mnemonic = Mnemonic::from_phrase(mnemonic_words, Language::English)
            .map_err(|err| WalletError::Mnemonic(err.to_string()))?;

        let seed = Seed::new(&mnemonic, "");

        // Set hd_path for master_key generation
        let hd_path = StandardHDPath::try_from(derivation_path)
            .map_err(|_| WalletError::DerivationPath(derivation_path.to_string()))?;

        let keychain = MnemonicWallet::generate_keychain(hd_path, seed)?;

        Ok(MnemonicWallet {
            mnemonic,
            keychain,
            derivation_path: derivation_path.to_owned()
        })
    }

    /// Generates a wallet from a random mnemonic.
    pub fn random(derivation_path: &str) -> Result<(MnemonicWallet, String), WalletError> {
        let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
        let phrase = mnemonic.phrase().to_owned();

        Ok((MnemonicWallet::new(&phrase, derivation_path)?, phrase))
    }

    /// Change the derivation path used used to derive the key from the mnemonic.
    pub fn set_derivation_path(&mut self, derivation_path: &str) -> Result<(), WalletError> {
        // Update only if the derivation path is different.
        if derivation_path == self.derivation_path {
            return Ok(());
        }

        let seed = Seed::new(&self.mnemonic, "");

        // Set hd_path for master_key generation
        let hd_path = StandardHDPath::try_from(derivation_path)
            .map_err(|_| WalletError::DerivationPath(derivation_path.to_string()))?;

        // Regenerate the keychain with the new derivation path
        let keychain = MnemonicWallet::generate_keychain(hd_path, seed)?;

        // Update the wallet.
        self.keychain = keychain;
        self.derivation_path = derivation_path.to_string();

        Ok(())
    }

    /// Generate a keychain of Secp256k1 keys from the given hd_path and seed.
    fn generate_keychain(hd_path: StandardHDPath, seed: Seed) -> Result<Keychain, WalletError> {
        let private_key = ExtendedPrivKey::new_master(Network::Bitcoin, seed.as_bytes())
            .and_then(|priv_key| {
                priv_key.derive_priv(&Secp256k1::new(), &DerivationPath::from(hd_path))
            })
            .map_err(|err| WalletError::PrivateKey(err.to_string()))?;

        let public_key = ExtendedPubKey::from_private(&Secp256k1::new(), &private_key);

        Ok(Keychain {
            ext_private_key: private_key,
            ext_public_key: public_key,
        })
    }

    /// Get the public key derived from the mnemonic.
    pub fn get_pub_key(&self) -> PublicKey {
        self.keychain.ext_public_key.public_key.clone()
    }

    /// Get the bech32 address derived from the mnemonic and the provided
    /// human readable part.
    pub fn get_bech32_address(&self, hrp: &str) -> Result<String, WalletError> {
        let mut hasher = Sha256::new();
        let pub_key_bytes = self.get_pub_key().to_bytes();
        hasher.update(pub_key_bytes);

        // Read hash digest over the public key bytes & consume hasher
        let pk_hash = hasher.finalize();

        // Insert the hash result in the ripdem hash function
        let mut rip_hasher = Ripemd160::new();
        rip_hasher.update(pk_hash);
        let rip_result = rip_hasher.finalize();

        let address_bytes = rip_result.to_vec();

        let bech32_address = bech32::encode(hrp, address_bytes.to_base32(), Bech32)
            .map_err(|err| WalletError::Hrp(err.to_string()))?;

        Ok(bech32_address)
    }

    /// Sign the provided data using the public key derived from the mnemonic.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, WalletError> {
        if data.is_empty() {
            return Result::Ok(Vec::new());
        }
        //  Get the sign key from the private key
        let sign_key = SigningKey::from_bytes(&self.keychain.ext_private_key.private_key.to_bytes()).unwrap();

        // Sign the data provided data
        let signature: Signature = sign_key.try_sign(data)
            .map_err(|err| WalletError::Sign(err.to_string()))?;

        Ok(signature.as_ref().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    static DESMOS_DERIVATION_PATH: &str = "m/44'/852'/0'/0/0";
    static COSMOS_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
    static TEST_MNEMONIC: &str = "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel";

    #[test]
    fn initialization_with_valid_mnemonic_and_derivation_path() {
        let result = MnemonicWallet::new(TEST_MNEMONIC, DESMOS_DERIVATION_PATH);

        assert!(result.is_ok())
    }

    #[test]
    fn initialize_with_invalid_mnemonic() {
        let result = MnemonicWallet::new("an invalid mnemonic", DESMOS_DERIVATION_PATH);

        assert!(result.is_err());
    }

    #[test]
    fn initialize_with_invalid_derivation_path() {
        let result = MnemonicWallet::new(TEST_MNEMONIC, "");

        assert!(result.is_err());
    }

    #[test]
    fn desmos_bech32_address() {
        let wallet = MnemonicWallet::new(
            TEST_MNEMONIC,
            DESMOS_DERIVATION_PATH,
        ).unwrap();

        let address = wallet.get_bech32_address("desmos");

        assert!(address.is_ok());
        assert_eq!(address.unwrap(), "desmos1k8u92hx3k33a5vgppkyzq6m4frxx7ewnlkyjrh");
    }

    #[test]
    fn cosmos_bech32_address() {
        let wallet = MnemonicWallet::new(
            TEST_MNEMONIC,
            COSMOS_DERIVATION_PATH,
        ).unwrap();

        let address = wallet.get_bech32_address("cosmos");

        assert!(address.is_ok());
        assert_eq!(address.unwrap(), "cosmos1dzczdka6wpzwvmawpps7tf8047gkft0e5cupun");
    }

    #[test]
    fn empty_sign() {
        let wallet = MnemonicWallet::new(
            TEST_MNEMONIC,
            COSMOS_DERIVATION_PATH,
        ).unwrap();

        let empty: Vec<u8> = Vec::new();
        let result = wallet.sign(&empty);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn sign() {
        let signature = "304502205289d5bdcbc97ed3c660fafb6aa2ee593db9b5d2aa84d6cbfcdd49c68c1ded9b022100d317b7ef515dec0b378de0c19a5ed91b00fa150251eb72fe47bf2c9eb78e8846";
        let wallet = MnemonicWallet::new(
            TEST_MNEMONIC,
            COSMOS_DERIVATION_PATH,
        ).unwrap();

        let data = "some simple data".as_bytes();
        let result = wallet.sign(data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), hex::decode(signature).unwrap());
    }

}
