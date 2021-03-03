//! Wallet utility to build & sign transactions on every cosmos-sdk based network

// Includes code originally from ibc-rs:
// <https://github.com/informalsystems/ibc-rs>
// Copyright © 2020 Informal Systems Inc.
// Licensed under the Apache 2.0 license

use std::convert::TryFrom;

use bech32::{ToBase32, Variant::Bech32};
use bip39::{Language, Mnemonic, Seed};
use bitcoin::{
    network::constants::Network,
    secp256k1::Secp256k1,
    util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey},
};
use cosmos_sdk_proto::cosmos::{
    auth::v1beta1::BaseAccount,
    tx::v1beta1::{
        mode_info::{Single, Sum},
        AuthInfo, Fee, ModeInfo, SignDoc, SignerInfo, TxBody, TxRaw,
    },
};
use hdpath::StandardHDPath;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use prost_types::Any;
use ripemd160::Ripemd160;
use sha2::{Digest, Sha256};

use crate::error::Error;
use crate::msg::Msg;
use crate::rpc::ChainClient;

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
    pub fn from_mnemonic(
        mnemonic_words: &str,
        derivation_path: String,
        hrp: String,
    ) -> Result<Wallet, Error> {
        // Create mnemonic and generate seed from it
        let mnemonic =
            Mnemonic::from_phrase(mnemonic_words, Language::English)
                .map_err(|err| Error::Mnemonic(err.to_string()))?;
        let seed = Seed::new(&mnemonic, "");

        // Set hd_path for master_key generation
        let hd_path = StandardHDPath::try_from(derivation_path.as_str()).unwrap();

        let keychain = generate_keychain(hd_path, seed)?;

        let bech32_address = bech32_address_from_public_key(keychain.ext_public_key.clone(), hrp)?;

        let wallet = Wallet {
            keychain,
            bech32_address,
        };

        Ok(wallet)
    }

    pub fn sign_tx(
        &self,
        account: BaseAccount,
        chain_client: ChainClient,
        msgs: &[Msg],
        fee: Fee,
        memo: Option<String>,
        timeout_height: u64,
    ) -> Result<Vec<u8>, Error> {
        // Check if the caller passed some memo
        let memo = match memo {
            None => "".to_string(),
            Some(mem) => mem,
        };

        // Create tx body
        let tx_body = TxBody {
            messages: msgs.iter().map(|msg| msg.0.clone()).collect(),
            memo: memo,
            timeout_height: timeout_height,
            extension_options: Vec::<Any>::new(),
            non_critical_extension_options: Vec::<Any>::new(),
        };

        // Protobuf tx_body serialization
        let mut tx_body_buffer = Vec::new();
        prost::Message::encode(&tx_body, &mut tx_body_buffer)
            .map_err(|err| Error::Encoding(err.to_string()))?;

        // Protobuf public_key serialization
        let mut pk_buffer = Vec::new();
        prost::Message::encode(
            &self.keychain.ext_public_key.public_key.to_bytes(),
            &mut pk_buffer,
        )
        .map_err(|err| Error::Encoding(err.to_string()))?;

        // TODO extract a better key type (not an Any type)
        let public_key_any = Any {
            type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
            value: pk_buffer,
        };

        // Signer specifications
        let single_signer = Single { mode: 1 };
        let single_signer_specifier = Some(Sum::Single(single_signer));
        let broadcast_mode = Some(ModeInfo {
            sum: single_signer_specifier,
        });

        // Building signer's info
        let signer_info = SignerInfo {
            public_key: Some(public_key_any),
            mode_info: broadcast_mode,
            sequence: account.sequence,
        };

        let auth_info = AuthInfo {
            signer_infos: vec![signer_info],
            fee: Some(fee),
        };

        // Protobuf auth_info serialization
        let mut auth_buffer = Vec::new();
        prost::Message::encode(&auth_info, &mut auth_buffer)
            .map_err(|err| Error::Encoding(err.to_string()))?;

        let sign_doc = SignDoc {
            body_bytes: tx_body_buffer.clone(),
            auth_info_bytes: auth_buffer.clone(),
            chain_id: chain_client.node_info.id,
            account_number: account.account_number,
        };

        // Protobuf sign_doc serialization
        let mut sign_doc_buffer = Vec::new();
        prost::Message::encode(&sign_doc, &mut sign_doc_buffer)
            .map_err(|err| Error::Encoding(err.to_string()))?;

        // sign the doc buffer
        let signature: Signature = sign_bytes(self.keychain.ext_private_key, sign_doc_buffer);
        let signed = signature.as_ref().to_vec();

        // compose the raw tx
        let tx_raw = TxRaw {
            body_bytes: tx_body_buffer,
            auth_info_bytes: auth_buffer,
            signatures: vec![signed],
        };

        // Protobuf tx_raw serialization
        let mut tx_signed_bytes = Vec::new();
        prost::Message::encode(&tx_raw, &mut tx_signed_bytes)
            .map_err(|err| Error::Encoding(err.to_string()))?;

        Ok(tx_signed_bytes)
    }
}

/// generate a keychain of Secp256k1 keys from the given hd_path and seed.
fn generate_keychain(hd_path: StandardHDPath, seed: Seed) -> Result<Keychain, Error> {
    let private_key = ExtendedPrivKey::new_master(Network::Bitcoin, seed.as_bytes())
        .and_then(|priv_key| {
            priv_key.derive_priv(&Secp256k1::new(), &DerivationPath::from(hd_path))
        })
        .map_err(|err| Error::PrivateKey(err.to_string()))?;

    let public_key = ExtendedPubKey::from_private(&Secp256k1::new(), &private_key);

    Ok(Keychain {
        ext_private_key: private_key,
        ext_public_key: public_key,
    })
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
    let mut hasher = Sha256::new();
    hasher.update(pub_key.public_key.to_bytes().as_slice());

    // Read hash digest over the public key bytes & consume hasher
    let pk_hash = hasher.finalize();

    // Insert the hash result in the ripdem hash function
    let mut rip_hasher = Ripemd160::new();
    rip_hasher.update(pk_hash);
    let rip_result = rip_hasher.finalize();

    let address_bytes = rip_result.to_vec();

    let bech32_address = bech32::encode(hrp.as_str(), address_bytes.to_base32(), Bech32)
        .map_err(|err| Error::Bech32(err.to_string()))?;

    Ok(bech32_address)
}


/// sign the given bytes with the given private key, returning a signature representation
fn sign_bytes(ext_private_key: ExtendedPrivKey, bytes_to_sign: Vec<u8>) -> Signature {
    let private_key_bytes = ext_private_key.private_key.to_bytes();
    let signing_key = SigningKey::from_bytes(private_key_bytes.as_slice()).unwrap();
    signing_key.sign(&bytes_to_sign)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::rpc::get_node_info;
    use cosmos_sdk_proto::cosmos::{
        base::v1beta1::Coin,
        bank::v1beta1::MsgSend
    };
    use k256::ecdsa::{
        VerifyingKey,
        signature::Verifier
    };

    struct TestData {
        hd_path: StandardHDPath,
        seed: Seed,
    }

    impl TestData {
        fn setup_test(derivation_path: &str, mnemonic_words: &str) -> TestData {
            let hd_path = StandardHDPath::try_from(derivation_path).unwrap();
            let mnemonic = Mnemonic::from_phrase(mnemonic_words, Language::English).unwrap();
            let seed = Seed::new(&mnemonic, "");
            TestData { hd_path, seed }
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
        assert_eq!(
            keychain.ext_public_key.public_key.to_string(),
            "02f5bf794ef934cb419bb9113f3a94c723ec6c2881a8d99eef851fd05b61ad698d"
        )
    }

    #[test]
    fn bech32_address_from_public_key_works() {
        let test_data = TestData::setup_test(
            "m/44'/852'/0'/0/0",
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel"
        );

        let keychain = generate_keychain(test_data.hd_path, test_data.seed).unwrap();
        let bech32_address =
            bech32_address_from_public_key(keychain.ext_public_key, "desmos".to_string()).unwrap();

        assert_ne!(bech32_address.len(), 0);
        assert_eq!(
            bech32_address,
            "desmos1k8u92hx3k33a5vgppkyzq6m4frxx7ewnlkyjrh"
        )
    }

    #[test]
    fn from_mnemonic_works() {
        let wallet = Wallet::from_mnemonic(
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel",
            "m/44'/852'/0'/0/0".to_string(),
            "desmos".to_string(),
        ).unwrap();

        assert_eq!(
            wallet.bech32_address,
            "desmos1k8u92hx3k33a5vgppkyzq6m4frxx7ewnlkyjrh"
        );
        assert_eq!(
            wallet.keychain.ext_public_key.public_key.to_string(),
            "02f5bf794ef934cb419bb9113f3a94c723ec6c2881a8d99eef851fd05b61ad698d"
        )
    }

    #[test]
    fn sign_bytes_works() {
        let wallet = Wallet::from_mnemonic(
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel",
            "m/44'/852'/0'/0/0".to_string(),
            "desmos".to_string(),
        ).unwrap();

        let private_key = wallet.keychain.ext_private_key.clone();
        let public_key = wallet.keychain.ext_public_key.public_key.key.clone();
        let signing_key = SigningKey::from_bytes(private_key.private_key.to_bytes().as_slice()).unwrap();
        let verify_key = VerifyingKey::from(&signing_key);

        let amount = Coin{ denom: "stake".to_string(), amount: "100000".to_string() };
        let msg = MsgSend{
            from_address: wallet.bech32_address.clone(),
            to_address: "desmos1gvd8j8w986qey68s6trc3h9zkzxest20zs5g0w".to_string(),
            amount: vec![amount]
        };

        let mut msg_bytes =  Vec::new();
        prost::Message::encode(&msg, &mut msg_bytes).unwrap();

        let signature: Signature = sign_bytes(private_key, msg_bytes.clone());

        assert!(verify_key.verify(msg_bytes.clone().as_slice(), &signature).is_ok());
    }

    #[actix_rt::test]
    async fn sign_tx_works() {
        let wallet = Wallet::from_mnemonic(
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel",
            "m/44'/852'/0'/0/0".to_string(),
            "desmos".to_string(),
        ).unwrap();

        let lcd_endpoint = "http://localhost:1317";
        let node_info = get_node_info(lcd_endpoint.to_string())
            .await
            .unwrap()
            .node_info;
        let grpc_endpoint = "http://localhost:9090";
        let chain_client = ChainClient::new(node_info, lcd_endpoint.to_string(), grpc_endpoint.to_string());

        let account = chain_client.get_account_data(wallet.bech32_address.clone())
            .await
            .unwrap();

        // Gas Fee
        let coin = Coin {
            denom: "stake".to_string(),
            amount: "5000".to_string(),
        };

        let fee = Fee {
            amount: vec![coin],
            gas_limit: 300000,
            payer: "".to_string(),
            granter: "".to_string(),
        };

        let amount = Coin{ denom: "stake".to_string(), amount: "100000".to_string() };
        let msg = MsgSend{
            from_address: wallet.bech32_address.clone(),
            to_address: "desmos1gvd8j8w986qey68s6trc3h9zkzxest20zs5g0w".to_string(),
            amount: vec![amount]
        };

        let mut msg_bytes =  Vec::new();
        prost::Message::encode(&msg, &mut msg_bytes).unwrap();

        let proto_msg = Msg::new(
            "/cosmos.bank.v1beta1.Msg/Send",
            msg_bytes
        );

        let tx_signed_bytes = wallet.sign_tx(account, chain_client, &[proto_msg], fee, None, 0)
            .unwrap();

        let tx_raw: TxRaw = prost::Message::decode(tx_signed_bytes.as_slice()).unwrap();

        assert_ne!(tx_raw.signatures[0].len(), 0)
    }
}
