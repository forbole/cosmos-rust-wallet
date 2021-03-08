use wasm_bindgen::prelude::*;
use crate::{
    rpc::{
        ChainClient,
        get_node_info,
        NodeInfoResponse
    },
    wallet::{
        Wallet,
        WalletJS
    },
    error::Error
};
use crate::msg::Msg;


/// Import a wallet from the given mnemonic
#[wasm_bindgen(js_name = "importWallet")]
pub fn import_wallet(mnemonic: &str, derivation_path: &str, hrp: &str) -> Result<JsValue, JsValue> {
 let wallet: WalletJS = Wallet::from_mnemonic(
     mnemonic,
     derivation_path.to_string(),
     hrp.to_string()
 )
     .map_err(| error | JsValue::from_serde(&error).unwrap())?
     .into();

 Ok(JsValue::from_serde(&wallet).unwrap())
}


/// Sign and send a transaction with the given wallet
#[wasm_bindgen(js_name = "signAndSendMsg")]
pub fn sign_and_send_msg(wallet: &JsValue, msg: &JsValue, lcd_addr: &str, grpc_addr: &str) -> Result<JsValue, JsValue> {
    let walletJS : WalletJS = wallet.into_serde().unwrap();
    let msg: Msg = msg.into_serde().unwrap();

    let wallet = walletJS.into()
}