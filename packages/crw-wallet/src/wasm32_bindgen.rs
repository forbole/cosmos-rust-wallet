//! Implementation for WASM via via wasm-bindgen

use crate::crypto::MnemonicWallet;
extern crate bindgen as wasm_bindgen;
use bip39::{Language, Mnemonic, MnemonicType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = MnemonicWallet)]
pub struct JsMnemonicWallet {
    wallet: MnemonicWallet,
}

#[wasm_bindgen(js_class = MnemonicWallet)]
impl JsMnemonicWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(mnemonic: &str, derivation_path: &str) -> Result<JsMnemonicWallet, JsValue> {
        return Ok(JsMnemonicWallet {
            wallet: MnemonicWallet::new(mnemonic, derivation_path)
                .map_err(|e| JsValue::from(e.to_string()))?,
        });
    }

    #[wasm_bindgen(js_name = setDerivationPath)]
    pub fn set_derivation_path(&mut self, new_derivation_path: &str) -> Result<(), JsValue> {
        self.wallet
            .set_derivation_path(new_derivation_path)
            .map_err(|e| JsValue::from(e.to_string()))?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getBech32Address)]
    pub fn get_bech32_address(&self, hrp: &str) -> Result<String, JsValue> {
        Ok(self
            .wallet
            .get_bech32_address(hrp)
            .map_err(|e| JsValue::from(e.to_string()))?)
    }

    pub fn sign(&self, data: Vec<u8>) -> Result<Vec<u8>, JsValue> {
        Ok(self
            .wallet
            .sign(&data)
            .map_err(|e| JsValue::from(e.to_string()))?)
    }
}

#[wasm_bindgen(js_name = randomMnemonic)]
pub fn random_mnemonic() -> String {
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
    return mnemonic.phrase().to_string();
}
