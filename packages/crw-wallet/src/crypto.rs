//! Utility to create an in memory Secp256k1 wallet from a BIP-32 mnemonic.
//!
//! This module contains functions to generate a Secp256k1 key pair from a BIP-32 mnemonic and
//! sign a generic [`Vec<u8>`] payload.

use crate::WalletError;
use bech32::{ToBase32, Variant::Bech32};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
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

/// Represents a Secp256k1 key pair.
#[derive(Clone)]
struct Keychain {
    pub ext_public_key: ExtendedPubKey,
    pub ext_private_key: ExtendedPrivKey,
}

/// Facility used to manage a Secp256k1 key pair and generate signatures.
#[derive(Clone)]
pub struct MnemonicWallet {
    mnemonic: Mnemonic,
    derivation_path: String,
    keychain: Keychain,
}

impl MnemonicWallet {
    /// Derive a Secp256k1 key pair from the given `mnemonic_phrase` and `derivation_path`.
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the provided `mnemonic_phrase` or `derivation_path` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use crw_wallet::crypto::MnemonicWallet;
    ///
    /// let cosmos_dp = "m/44'/118'/0'/0/0";
    /// let mnemonic = "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel";
    ///
    /// let wallet = MnemonicWallet::new(mnemonic, cosmos_dp).unwrap();
    /// ```
    pub fn new(
        mnemonic_phrase: &str,
        derivation_path: &str,
    ) -> Result<MnemonicWallet, WalletError> {
        // Create mnemonic and generate seed from it
        let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English)
            .map_err(|err| WalletError::Mnemonic(err.to_string()))?;

        let seed = Seed::new(&mnemonic, "");

        // Set hd_path for master_key generation
        let hd_path = StandardHDPath::try_from(derivation_path)
            .map_err(|_| WalletError::DerivationPath(derivation_path.to_string()))?;

        let keychain = MnemonicWallet::generate_keychain(hd_path, seed)?;

        Ok(MnemonicWallet {
            mnemonic,
            keychain,
            derivation_path: derivation_path.to_owned(),
        })
    }

    /// Generates a random mnemonic phrase and derive a Secp256k1 key pair from it.
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the provided `derivation_path` is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use crw_wallet::crypto::MnemonicWallet;
    ///
    /// let cosmos_dp = "m/44'/118'/0'/0/0";
    ///
    /// let (wallet, mnemonic) = MnemonicWallet::random(cosmos_dp).unwrap();
    /// ```
    pub fn random(derivation_path: &str) -> Result<(MnemonicWallet, String), WalletError> {
        let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
        let phrase = mnemonic.phrase().to_owned();

        Ok((MnemonicWallet::new(&phrase, derivation_path)?, phrase))
    }

    /// Changes the derivation path used used to derive the key pair from the mnemonic.
    /// This function force the regenerations of the wallet internal keypair.
    ///
    /// # Errors
    ///
    /// Returns an [`Err`] if the provided `derivation_path` is invalid.
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

    /// Utility function to generate the Secp256k1 keypair.
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

    /// Gets the public key derived from the mnemonic.
    pub fn get_pub_key(&self) -> PublicKey {
        self.keychain.ext_public_key.public_key
    }

    /// Gets the bech32 address derived from the mnemonic and the provided
    /// human readable part.
    ///
    /// # Errors
    /// Returns an an [`Err`] in one of this cases:
    /// * If the hrp contains both uppercase and lowercase characters.
    /// * If the hrp contains any non-ASCII characters (outside 33..=126).
    /// * If the hrp is outside 1..83 characters long.
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

    /// Returns the signature of the provided data.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, WalletError> {
        if data.is_empty() {
            return Result::Ok(Vec::new());
        }
        //  Get the sign key from the private key
        let sign_key =
            SigningKey::from_bytes(&self.keychain.ext_private_key.private_key.to_bytes()).unwrap();

        // Sign the data provided data
        let signature: Signature = sign_key
            .try_sign(data)
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

        assert!(match result.err().unwrap() {
            WalletError::Mnemonic(_) => true,
            _ => false,
        });
    }

    #[test]
    fn initialize_with_invalid_derivation_path() {
        let result = MnemonicWallet::new(TEST_MNEMONIC, "");

        assert!(result.is_err());

        assert!(match result.err().unwrap() {
            WalletError::DerivationPath(_) => true,
            _ => false,
        });
    }

    #[test]
    fn initialize_random_wallet() {
        let result = MnemonicWallet::random(DESMOS_DERIVATION_PATH);

        assert!(result.is_ok());
        let (_, mnemonic) = result.unwrap();

        assert!(!mnemonic.is_empty());
    }

    #[test]
    fn initialize_random_wallet_with_invalid_dp() {
        let result = MnemonicWallet::random("");

        assert!(result.is_err());
        let wallet_error = result.err().unwrap();

        assert!(match wallet_error {
            WalletError::DerivationPath(_) => true,
            _ => false,
        });
    }

    #[test]
    fn desmos_bech32_address() {
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, DESMOS_DERIVATION_PATH).unwrap();

        let address = wallet.get_bech32_address("desmos");

        assert!(address.is_ok());
        assert_eq!(
            address.unwrap(),
            "desmos1k8u92hx3k33a5vgppkyzq6m4frxx7ewnlkyjrh"
        );
    }

    #[test]
    fn cosmos_bech32_address() {
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, COSMOS_DERIVATION_PATH).unwrap();

        let address = wallet.get_bech32_address("cosmos");

        assert!(address.is_ok());
        assert_eq!(
            address.unwrap(),
            "cosmos1dzczdka6wpzwvmawpps7tf8047gkft0e5cupun"
        );
    }

    #[test]
    fn empty_sign() {
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, COSMOS_DERIVATION_PATH).unwrap();

        let empty: Vec<u8> = Vec::new();
        let result = wallet.sign(&empty);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn sign() {
        let ref_hex_signature = "ce0558eb2f0847d4e58b29ca45f0a2a8764395b52c829888fa017aaf5b8b2e695e47aac9fe1cf77a66a1ba872d8a7e5302d31874b686973c0a5c196cca707667";
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, DESMOS_DERIVATION_PATH).unwrap();

        let data = "some simple data".as_bytes();
        let result = wallet.sign(data).unwrap();

        let sign_hex = hex::encode(result);

        assert_eq!(ref_hex_signature, sign_hex);
    }
}
