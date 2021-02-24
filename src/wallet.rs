use std::convert::TryFrom;
use crate::error::{Error};
use bech32::{ToBase32, Variant::Bech32};
use bitcoin::{
    network::constants::Network,
    secp256k1::Secp256k1,
    util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey},
};
use bitcoin_wallet::{mnemonic::Mnemonic, account::Seed};
use hdpath::{StandardHDPath};
use crypto::{
    digest::Digest,
    ripemd160::Ripemd160,
    sha2::Sha256
};


/// Keychain contains a pair of Secp256k1 keys.
pub struct Keychain {
    pub ext_public_key: ExtendedPubKey,
    pub ext_private_key: ExtendedPrivKey,
}

/// Wallet is a facility used to manipulate private and public keys associated
/// to a BIP-32 mnemonic.
pub struct Wallet {
    pub keychain: Keychain,
    pub bech32_address: String,
}

impl Wallet {
    /// Derive a Wallet from the given mnemonic_words, derivation path and human readable part
    pub fn from_mnemonic(mnemonic_words: &str, derivation_path: String, hrp: String) -> Result<Wallet, Error> {
        // Create mnemonic and generate seed from it
        let mnemonic = Mnemonic::from_str(mnemonic_words)
            .map_err(| err | Error::Mnemonic(err.to_string()))?;
        let seed = mnemonic.to_seed(Some(""));

        // Set hd_path for master_key generation
        let hd_path = StandardHDPath::try_from(derivation_path.as_str())
            .unwrap();

        let keychain = generate_keychain(hd_path, seed)?;

        let bech32_address = bech32_address_from_public_key(keychain.ext_public_key.clone(), hrp)?;

        let wallet = Wallet {
            keychain,
            bech32_address
        };

        Ok(wallet)
    }
}

/// generate a keychain of Secp256k1 keys from the given hd_path and seed.
fn generate_keychain(hd_path: StandardHDPath, seed: Seed) -> Result<Keychain, Error> {
    let private_key = ExtendedPrivKey::new_master(Network::Bitcoin, &seed.0)
        .and_then(|priv_key| priv_key.derive_priv(&Secp256k1::new(), &DerivationPath::from(hd_path)))
        .map_err(| err | Error::PrivateKey(err.to_string()))?;

    let public_key = ExtendedPubKey::from_private(&Secp256k1::new(), &private_key);

    Ok(Keychain{ ext_private_key: private_key, ext_public_key: public_key })
}

/// To construct a bech32 address from a public key we need 3 pieces:
    /// 1) human readable part: e.g "desmos" "cosmos" "akash"
    /// 2) witness version: it can be 0 (0x00 byte) up to 16 (0x10)
    /// 3) witness program: it depends on which key we want,
    ///    in our case we want a Pay-to-witness-public-key (P2WPK)
    ///    so the 20-byte hash160 of the compressed public key
    ///    e.g
    ///    ripemd160(sha256(compressed_pub_key))
fn bech32_address_from_public_key(pub_key: ExtendedPubKey, hrp: String) -> Result<String, Error> {
    let mut sha256_digest = Sha256::new();
    sha256_digest.input(pub_key.public_key.to_bytes().as_slice());
    let mut sha256_bytes = vec![0; sha256_digest.output_bytes()];
    sha256_digest.result(&mut sha256_bytes);

    let mut ripdem_hash = Ripemd160::new();
    ripdem_hash.input(sha256_bytes.as_slice());
    let mut address_bytes = vec![0; ripdem_hash.output_bytes()];
    ripdem_hash.result(&mut address_bytes);
    address_bytes.to_vec();

    let bech32_address = bech32::encode(hrp.as_str(), address_bytes.to_base32(), Bech32)
        .map_err(| err | Error::Bech32(err.to_string()))?;

    Ok(bech32_address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Error};

    struct TestData {
        hd_path: StandardHDPath,
        seed: Seed,
    }

    impl TestData {
        fn setup_test(derivation_path: &str, mnemonic_words: &str) -> TestData {
            let hd_path = StandardHDPath::try_from(derivation_path).unwrap();
            let mnemonic = Mnemonic::from_str(mnemonic_words).unwrap();
            let seed = mnemonic.to_seed(Some(""));
            TestData{ hd_path, seed }
        }
    }

    #[test]
    fn generate_keychain_works() {
        let test_data = TestData::setup_test(
            "m/44'/852'/0'/0/0",
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel"
        );

        let keychain = generate_keychain(test_data.hd_path, test_data.seed).unwrap();

        assert_ne!(keychain.ext_public_key.public_key.to_string().len(), 0);
        assert_eq!(keychain.ext_public_key.public_key.to_string(), "02f5bf794ef934cb419bb9113f3a94c723ec6c2881a8d99eef851fd05b61ad698d")
    }

    #[test]
    fn bech32_address_from_public_key_works() {
        let test_data = TestData::setup_test(
            "m/44'/852'/0'/0/0",
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel"
        );

        let keychain = generate_keychain(test_data.hd_path, test_data.seed).unwrap();
        let bech32_address = bech32_address_from_public_key(keychain.ext_public_key, "desmos".to_string()).unwrap();

        assert_ne!(bech32_address.len(), 0);
        assert_eq!(bech32_address, "desmos1k8u92hx3k33a5vgppkyzq6m4frxx7ewnlkyjrh")
    }

    #[test]
    fn from_mnemonic_works() {
        let wallet = Wallet::from_mnemonic(
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel",
            "m/44'/852'/0'/0/0".to_string(),
            "desmos".to_string(),
        ).unwrap();

        assert_eq!(wallet.bech32_address, "desmos1k8u92hx3k33a5vgppkyzq6m4frxx7ewnlkyjrh");
        assert_eq!(wallet.keychain.ext_public_key.public_key.to_string(), "02f5bf794ef934cb419bb9113f3a94c723ec6c2881a8d99eef851fd05b61ad698d")
    }
}