//! Module that provides the generic trait to store and load preferences from the device storage.

use crate::io::IoError;
use std::result;
use thiserror::Error;

pub type Result<T> = result::Result<T, PreferencesError>;

#[derive(Error, Debug)]
pub enum PreferencesError {
    #[error("invalid preference name `{0}`")]
    InvalidName(String),
    #[error("i/o error `{0}`")]
    IO(#[from] IoError),
    #[error("error while deserializing the preferences")]
    DeserializationError,
    #[error("error while serializing the preferences")]
    SerializationError,
}

/// Trait that represents a generic preference container.
pub trait Preferences {
    /// Gets a i32 from the preferences.
    ///
    /// - *key* The name of the preference to retrieve.
    fn get_i32(&self, key: &str) -> Option<i32>;

    /// Store a i32 into the preferences.
    ///
    /// - *key* The name of the preference that will be stored.
    /// - *value* The value that will be stored into the preferences.
    fn put_i32(&mut self, key: &str, value: i32) -> Result<()>;

    /// Gets a string from the preferences.
    ///
    /// * `key` - name of the preference that will be loaded.
    fn get_str(&self, key: &str) -> Option<String>;

    /// Store a string into the preferences.
    ///
    /// - *key* The name of the preference that will be stored.
    /// - *value* The value that will be stored into the preferences.
    fn put_str(&mut self, key: &str, value: String) -> Result<()>;

    /// Gets a boolean from the preferences.
    ///
    /// * `key` - name of the preference that will be loaded.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Store a boolean into the preferences.
    ///
    /// - *key* The name of the preference that will be stored.
    /// - *value* The value that will be stored into the preferences.
    fn put_bool(&mut self, key: &str, value: bool) -> Result<()>;

    /// Gets an array of bytes from the preferences.
    ///
    /// * `key` - name of the preference that will be loaded.
    fn get_binary(&self, key: &str) -> Option<Vec<u8>>;

    /// Store an array of bytes into the preferences.
    ///
    /// - *key* The name of the preference that will be stored.
    /// - *value* The array that will be stored into the preferences.
    fn put_binary(&mut self, key: &str, value: Vec<u8>) -> Result<()>;

    /// Delete all the preferences currently loaded.
    fn clear(&mut self);

    /// Delete all the preferences currently loaded and also the one stored into the
    /// device storage.
    fn erase(&mut self);

    /// Saves the preferences on the device disk.
    fn save(&self) -> Result<()>;
}
