//! Module to easily build and sign transactions for cosmos based blockchains.
//!
//! This module provides a facility to build and sign transactions for cosmos based blockchains.

use crate::error::TxBuildError;
use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
use cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::{Single, Sum};
use cosmos_sdk_proto::cosmos::tx::v1beta1::{
    AuthInfo, Fee, ModeInfo, SignDoc, SignerInfo, Tx, TxBody,
};
use crw_wallet::crypto::MnemonicWallet;
use prost::EncodeError;
use prost_types::Any;

/// Private structure used to represents the information of the account
/// that is performing the transaction.
struct AccountInfo {
    pub sequence: u64,
    pub number: u64,
}

/// The single signer transaction builder.
pub struct TxBuilder {
    chain_id: String,
    account_info: Option<AccountInfo>,
    tx_body: TxBody,
    fee: Option<Fee>,
}

impl TxBuilder {
    /// Function to create a new `TxBuilder`.
    ///
    /// # Example
    ///
    /// This is a simple example of a cosmos Send transaction.
    ///
    ///```
    /// use crw_wallet::crypto::MnemonicWallet;
    /// use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    /// use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    /// use crw_client::tx::TxBuilder;
    ///
    /// let cosmos_derivation_path = "m/44'/118'/0'/0/0";
    /// let (wallet, mnemonic) = MnemonicWallet::random(cosmos_derivation_path).unwrap();
    ///
    /// let amount = Coin {
    ///     denom: "stake".to_string(),
    ///     amount: "10".to_string(),
    ///  };
    ///  let msg_snd = MsgSend {
    ///      // Get the bech32 address associated to the wallet
    ///      from_address: wallet.get_bech32_address("cosmos").unwrap(),
    ///      to_address: "cosmos18ek6mnlxj8sysrtvu60k5zj0re7s5n42yncner".to_string(),
    ///      amount: vec![amount],
    ///  };
    ///
    /// let tx = TxBuilder::new("testchain")
    ///              .memo("Test memo")
    ///              .account_info(1, 5)
    ///              .fee("stake", "10", 300_000)
    ///              .timeout_height(1000)
    ///              .add_message("/cosmos.bank.v1beta1.Msg/Send", msg_snd).unwrap()
    ///              .sign(&wallet).unwrap();
    ///```
    pub fn new(chain_id: &str) -> TxBuilder {
        TxBuilder {
            chain_id: chain_id.to_string(),
            account_info: Option::None,
            tx_body: TxBody {
                messages: Vec::<Any>::new(),
                memo: "".to_string(),
                timeout_height: 0,
                extension_options: Vec::<Any>::new(),
                non_critical_extension_options: Vec::<Any>::new(),
            },
            fee: Option::None,
        }
    }

    /// Sets the account information.
    pub fn account_info(mut self, sequence: u64, number: u64) -> Self {
        self.account_info = Some(AccountInfo { sequence, number });
        self
    }

    /// Append a message to the transaction messages.
    pub fn add_message<M: prost::Message>(
        self,
        msg_type: &str,
        msg: M,
    ) -> Result<Self, TxBuildError> {
        let mut serialized: Vec<u8> = Vec::new();

        prost::Message::encode(&msg, &mut serialized)?;

        Ok(self.add_message_raw(msg_type, serialized))
    }

    fn add_message_raw(mut self, msg_type: &str, binary: Vec<u8>) -> Self {
        let data = Any {
            type_url: msg_type.to_owned(),
            value: binary,
        };

        self.tx_body.messages.push(data);

        self
    }

    /// Sets the transaction memo.
    pub fn memo(mut self, memo: &str) -> Self {
        self.tx_body.memo = memo.to_string();
        self
    }

    /// Sets the transaction timout height.
    pub fn timeout_height(mut self, timeout_height: u64) -> Self {
        self.tx_body.timeout_height = timeout_height;
        self
    }

    /// Sets the transaction fee.
    pub fn fee(mut self, denom: &str, amount: &str, gas_limit: u64) -> Self {
        let coin = Coin {
            denom: denom.to_string(),
            amount: amount.to_string(),
        };

        self.fee = Some(Fee {
            amount: vec![coin],
            gas_limit,
            payer: "".to_string(),
            granter: "".to_string(),
        });

        self
    }

    /// Generate the signed transaction using the provided wallet.
    ///
    /// The transaction will be signed following the `SIGN_MODE_DIRECT` specification.
    /// See [Cosmos adr-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
    /// for more details.
    ///
    /// # Errors
    /// Returns an ['Err`] if one of the following cases:
    /// * If an error occur during the transaction serialization to protobuf
    /// * If an error occur during the transaction signature.
    pub fn sign(self, wallet: &MnemonicWallet) -> Result<Tx, TxBuildError> {
        if self.account_info.is_none() {
            return Result::Err(TxBuildError::NoAccountInfo);
        }

        if self.fee.is_none() {
            return Result::Err(TxBuildError::NoFee);
        }

        // Protobuf tx_body serialization
        let mut tx_body_buffer = Vec::new();
        prost::Message::encode(&self.tx_body, &mut tx_body_buffer)?;

        let mut serialized_key: Vec<u8> = Vec::new();
        prost::Message::encode(&wallet.get_pub_key().to_bytes(), &mut serialized_key)?;

        // TODO extract a better key type (not an Any type)
        let public_key_any = Any {
            type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
            value: serialized_key,
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
            sequence: self.account_info.as_ref().unwrap().sequence,
        };

        let auth_info = AuthInfo {
            signer_infos: vec![signer_info],
            fee: Some(self.fee.as_ref().unwrap().clone()),
        };

        // Protobuf auth_info serialization
        let mut auth_buffer = Vec::new();
        prost::Message::encode(&auth_info, &mut auth_buffer)?;
        let sign_doc = SignDoc {
            body_bytes: tx_body_buffer,
            auth_info_bytes: auth_buffer,
            chain_id: self.chain_id,
            account_number: self.account_info.as_ref().unwrap().number,
        };

        // Protobuf sign_doc serialization
        let mut sign_doc_buffer = Vec::new();
        prost::Message::encode(&sign_doc, &mut sign_doc_buffer)?;

        // sign the doc buffer
        let signature = wallet
            .sign(&sign_doc_buffer)
            .map_err(|err| TxBuildError::Sign(err.to_string()))?;

        // compose the raw tx
        Result::Ok(Tx {
            body: Some(self.tx_body),
            auth_info: Some(auth_info),
            signatures: vec![signature],
        })
    }
}

impl From<EncodeError> for TxBuildError {
    fn from(e: EncodeError) -> Self {
        TxBuildError::Encode(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::tx::{TxBuildError, TxBuilder};
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmos_sdk_proto::cosmos::base::v1beta1::Coin;
    use crw_wallet::crypto::MnemonicWallet;

    static TEST_MNEMONIC: &str = "elephant luggage finger obscure nest smooth flag clay recycle unfair capital category organ bicycle gallery sight canyon hotel dutch skull today pink scale aisle";
    static DESMOS_DERIVATION_PATH: &str = "m/44'/852'/0'/0/0";

    #[test]
    fn test_missing_fee() {
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, DESMOS_DERIVATION_PATH).unwrap();

        let amount = Coin {
            denom: "stake".to_string(),
            amount: "10".to_string(),
        };
        let msg_snd = MsgSend {
            from_address: wallet.get_bech32_address("desmos").unwrap(),
            to_address: "desmos18ek6mnlxj8sysrtvu60k5zj0re7s5n42yncner".to_string(),
            amount: vec![amount],
        };

        let tx_builder = TxBuilder::new("testchain")
            .memo("Test memo")
            .account_info(0, 0)
            .timeout_height(0);

        let sign_result = tx_builder
            .add_message("/cosmos.bank.v1beta1.Msg/Send", msg_snd)
            .unwrap()
            .sign(&wallet);

        assert!(sign_result.is_err());
        assert_eq!(TxBuildError::NoFee, sign_result.err().unwrap());
    }

    #[test]
    fn test_missing_account() {
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, DESMOS_DERIVATION_PATH).unwrap();

        let amount = Coin {
            denom: "stake".to_string(),
            amount: "10".to_string(),
        };
        let msg_snd = MsgSend {
            from_address: wallet.get_bech32_address("desmos").unwrap(),
            to_address: "desmos18ek6mnlxj8sysrtvu60k5zj0re7s5n42yncner".to_string(),
            amount: vec![amount],
        };

        let sign_result = TxBuilder::new("testchain")
            .memo("Test memo")
            .fee("stake", "10", 300_000)
            .timeout_height(0)
            .add_message("/cosmos.bank.v1beta1.Msg/Send", msg_snd)
            .unwrap()
            .sign(&wallet);

        assert!(sign_result.is_err());
        assert_eq!(TxBuildError::NoAccountInfo, sign_result.err().unwrap());
    }

    #[test]
    fn test_sign() {
        let wallet = MnemonicWallet::new(TEST_MNEMONIC, DESMOS_DERIVATION_PATH).unwrap();

        let amount = Coin {
            denom: "stake".to_string(),
            amount: "10".to_string(),
        };
        let msg_snd = MsgSend {
            from_address: wallet.get_bech32_address("desmos").unwrap(),
            to_address: "desmos18ek6mnlxj8sysrtvu60k5zj0re7s5n42yncner".to_string(),
            amount: vec![amount],
        };

        let tx = TxBuilder::new("testchain")
            .memo("Test memo")
            .account_info(1, 5)
            .fee("stake", "10", 300_000)
            .timeout_height(1000)
            .add_message("/cosmos.bank.v1beta1.Msg/Send", msg_snd)
            .unwrap()
            .sign(&wallet)
            .unwrap();

        let tx_body = tx.body.unwrap();
        let auth_info = tx.auth_info.unwrap();

        assert_eq!("Test memo", &tx_body.memo);
        assert_eq!(1000, tx_body.timeout_height);

        // Should be 1 since TxBuilder support only single sign.
        assert_eq!(1, tx.signatures.len());
        assert_eq!(1, auth_info.signer_infos.len());
        // Check that the sequence is the same passed to account_info
        assert_eq!(1, auth_info.signer_infos[0].sequence);
    }
}
