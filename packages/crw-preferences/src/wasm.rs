//! Module that provides a wrapper to expose a [Preferences] to a js application.

extern crate bindgen as wasm_bindgen;

use crate::preferences::{Preferences, PreferencesError};
use crate::unencrypted::UnencryptedPreferences;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = Preferences)]
pub struct PreferencesWrapper {
    container: Box<dyn Preferences>,
}

#[wasm_bindgen(js_class = Preferences)]
impl PreferencesWrapper {
    pub fn get_i32(&self, key: &str) -> Option<i32> {
        self.container.get_i32(key)
    }

    pub fn put_i32(&mut self, key: &str, value: i32) -> Result<(), JsValue> {
        Ok(self.container.put_i32(key, value)?)
    }

    pub fn get_str(&self, key: &str) -> Option<String> {
        self.container.get_str(key)
    }

    pub fn put_str(&mut self, key: &str, value: &str) -> Result<(), JsValue> {
        Ok(self.container.put_str(key, value.to_owned())?)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.container.get_bool(key)
    }

    pub fn put_bool(&mut self, key: &str, value: bool) -> Result<(), JsValue> {
        Ok(self.container.put_bool(key, value)?)
    }

    fn get_binary(&self, key: &str) -> Option<Vec<u8>> {
        self.container.get_binary(key)
    }

    fn put_binary(&mut self, key: &str, value: &[u8]) -> Result<(), JsValue> {
        Ok(self.container.insert(key.to_owned(), Value::from(value))?)
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

#[wasm_bindgen(js_name = "preferences")]
pub fn preferences(name: &str) -> Result<PreferencesWrapper, JsValue> {
    UnencryptedPreferences::new(name)
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
