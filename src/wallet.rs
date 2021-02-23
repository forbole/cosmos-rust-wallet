use bitcoin_wallet::mnemonic::Mnemonic;
use crate::error::{Error};
use hdpath::{CustomHDPath, StandardHDPath};
use bitcoin::{
    network::constants::Network,
    secp256k1::Secp256k1,
    hashes::core::convert::TryFrom,
    util::bip32::{ExtendedPrivKey, ExtendedPubKey, DerivationPath}
    bech32::ToBase32
};
use crypto::{
    digest::Digest,
    ripemd160::Ripemd160,
    sha2::Sha256
};

use bech32::Variant;

/// Keychain contains a pair of Secp256k1 keys.
struct Keychain {
    pub public_key: ExtendedPubKey,
    pub private_key: ExtendedPrivKey,
}

/// Wallet is a facility used to manipulate private and public keys associated
/// to a BIP-32 mnemonic.
pub struct Wallet {
    pub keychain: Keychain,
    pub bech32_address: String,
}

impl Wallet {
    pub fn from_mnemonic(mnemonic_words: &str, derivation_path: String, hrp: String) -> Result<Wallet, Error> {
        // Create mnemonic and generate seed from it
        let mnemonic = Mnemonic::from_str(mnemonic_words)
            .map_err(| err | Error::Mnemonic(err.to_string()))?;
        let seed = mnemonic.to_seed(Some(""));

        // Set hd_path for master_key generation
        let hd_path = StandardHDPath::try_from(derivation_path.as_str())
            .unwrap();

        // Create private key from seed and given derivation path
        let private_key = ExtendedPrivKey::new_master(Network::Bitcoin, &seed.0)
            .and_then(| key | key.derive_priv(&Secp256k1::new(), &DerivationPath::from(hd_path)))
            .map_err(| err | Error::PrivateKey(err.to_string()))?;

        // Get public key from private key
        let public_key = ExtendedPubKey::from_private(&Secp256k1::new(), &private_key);

        let address_bytes = get_address_bytes(public_key);

        let bech32_address = bech32::encode(hrp.as_str(), address_bytes.to_base32(), Variant::Bech32)
            .map_err(| err | Error::Bech32(err.to_string()))?;

        let wallet = Wallet {
            keychain: Keychain { public_key, private_key },
            bech32_address
        };

        Ok(wallet)
    }
}

/// To construct a bech32 address from a public key we need 3 pieces:
    /// 1) human readable part: e.g "desmos" "cosmos" "akash"
    /// 2) witness version: it can be 0 (0x00 byte) up to 16 (0x10)
    /// 3) witness program: it depends on which key we want,
    ///    in our case we want a Pay-to-witness-public-key (P2WPK)
    ///    so the 20-byte hash160 of the compressed public key
    ///    e.g
    ///    ripemd160(sha256(compressed_pub_key))
fn get_address_bytes(pub_key: ExtendedPubKey) -> Vec<u8> {
    let mut sha256 = Sha256::new();
    sha256.input(pub_key.public_key.to_bytes().as_slice());
    let mut bytes = vec![0; sha256.output_bytes()];
    sha256.result(&mut bytes);
    let mut hash = Ripemd160::new();
    hash.input(bytes.as_slice());
    let mut addr_bytes = vec![0; hash.output_bytes()];
    hash.result(&mut addr_bytes);
    addr_bytes.to_vec()
}