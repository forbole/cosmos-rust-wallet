use wasm_bindgen::prelude::*;
use crate::{
    rpc::{
        ChainClient,
        get_node_info
    },
    wallet::{
        Wallet
    }
};
use crate::rpc::NodeInfoResponse;
use crate::error::Error;
use crate::wallet::WalletJS;

/// Import a wallet from the given mnemonic
#[wasm_bindgen(js_name = "importWallet")]
pub fn import_wallet(mnemonic: &str, derivation_path: &str, hrp: &str) -> Result<JsValue, JsValue> {
 let wallet = Wallet::from_mnemonic(
     mnemonic,
     derivation_path.to_string(),
     hrp.to_string()
 )
     .map_err(| error | JsValue::from_serde(&error).unwrap())?
     .to_js_wallet();

 Ok(JsValue::from_serde(&wallet).unwrap())
}

/// Sign and send a transaction with the given wallet
pub fn sign_and_send_tx(wallet: &JsValue, msg: &JsValue, lcd_addr: &str, grpc_addr: &str) -> Result<JsValue, JsValue> {
    // TODO
}