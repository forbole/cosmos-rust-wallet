//! Module that provides a wrapper to expose a [Preferences] to a js application.

extern crate bindgen as wasm_bindgen;

use crate::encrypted::{EncryptedPreferences, EncryptedPreferencesError};
use crate::preferences;
use crate::preferences::{Preferences, PreferencesError};
use crate::unencrypted::UnencryptedPreferences;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Preferences)]
pub struct PreferencesWrapper {
    container: Box<dyn Preferences>,
}

#[wasm_bindgen(js_class = Preferences)]
impl PreferencesWrapper {
    #[wasm_bindgen(js_name = "getI32")]
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.container.get_i32(key)
    }

    #[wasm_bindgen(js_name = "putI32")]
    pub fn put_i32(&mut self, key: &str, value: i32) -> Result<(), JsValue> {
        Ok(self.container.put_i32(key, value)?)
    }

    #[wasm_bindgen(js_name = "getStr")]
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.container.get_str(key)
    }

    #[wasm_bindgen(js_name = "putStr")]
    pub fn put_str(&mut self, key: &str, value: &str) -> Result<(), JsValue> {
        Ok(self.container.put_str(key, value.to_owned())?)
    }

    #[wasm_bindgen(js_name = "getBool")]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.container.get_bool(key)
    }

    #[wasm_bindgen(js_name = "putBool")]
    pub fn put_bool(&mut self, key: &str, value: bool) -> Result<(), JsValue> {
        Ok(self.container.put_bool(key, value)?)
    }

    #[wasm_bindgen(js_name = "getBytes")]
    pub fn get_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.container.get_bytes(key)
    }

    #[wasm_bindgen(js_name = "putBytes")]
    pub fn put_bytes(&mut self, key: &str, value: Vec<u8>) -> Result<(), JsValue> {
        Ok(self.container.put_bytes(key, value)?)
    }

    pub fn clear(&mut self) {
        self.container.clear()
    }

    pub fn erase(&mut self) {
        self.container.erase();
    }

    pub fn save(&self) -> Result<(), JsValue> {
        Ok(self.container.save()?)
    }
}

#[wasm_bindgen]
pub fn exist(name: &str) -> bool {
    preferences::exist(name)
}

#[wasm_bindgen]
pub fn delete(name: &str) {
    preferences::delete(name);
}

#[wasm_bindgen(js_name = "preferences")]
pub fn preferences(name: &str) -> Result<PreferencesWrapper, JsValue> {
    UnencryptedPreferences::new(name)
        .map(|container| PreferencesWrapper {
            container: Box::new(container),
        })
        .map_err(JsValue::from)
}

#[wasm_bindgen(js_name = "encryptedPreferences")]
pub fn encrypted_preferences(password: &str, name: &str) -> Result<PreferencesWrapper, JsValue> {
    EncryptedPreferences::new(password, name)
        .map(|container| PreferencesWrapper {
            container: Box::new(container),
        })
        .map_err(JsValue::from)
}

impl From<PreferencesError> for JsValue {
    fn from(e: PreferencesError) -> Self {
        JsValue::from(e.to_string())
    }
}

impl From<EncryptedPreferencesError> for JsValue {
    fn from(e: EncryptedPreferencesError) -> Self {
        JsValue::from(e.to_string())
    }
}
