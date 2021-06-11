//! Module that provides an implementation of [Preferences] that saves the values encrypted into
//! the device storage.  
//! The data are securely stored into the device storage using the Chacha20Poly1305 algorithm.

use crate::io;
use crate::io::IoError;
use crate::preferences::{Preferences, PreferencesError, Result};
use base64::DecodeError;
use cocoon::{Cocoon, Error as CocoonErr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result::Result as StdResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncryptedPreferencesError {
    #[error("error while decrypting the data")]
    DecryptionFailed,
    // Wrapper to the preferences error.
    #[error("preferences error: `{0}`")]
    Preferences(Box<PreferencesError>),
}

#[derive(Serialize, Deserialize, Debug)]
enum Value {
    I32(i32),
    Bool(bool),
    String(String),
    Bin(Vec<u8>),
}

pub struct EncryptedPreferences {
    name: String,
    password: String,
    data: HashMap<String, Value>,
}

impl EncryptedPreferences {
    fn load_from_disk(
        password: &str,
        name: &str,
    ) -> StdResult<HashMap<String, Value>, EncryptedPreferencesError> {
        let read_result = io::load(name);

        if read_result.is_err() {
            let err = read_result.err().unwrap();
            return match err {
                IoError::EmptyData => Ok(HashMap::new()),
                IoError::InvalidName(s) => Err(EncryptedPreferencesError::from(
                    PreferencesError::InvalidName(s),
                )),
                _ => Err(EncryptedPreferencesError::from(PreferencesError::IO(err))),
            };
        }
        let base64_data = read_result.unwrap();

        // Get the data as binary
        let encrypted = base64::decode(base64_data)
            .map_err(|_| EncryptedPreferencesError::from(PreferencesError::DeserializationError))?;

        // Decrypt the binary data
        let cocoon = Cocoon::new(password.as_bytes());
        let decrypted = cocoon.unwrap(&encrypted).map_err(|e| match e {
            CocoonErr::Cryptography => EncryptedPreferencesError::DecryptionFailed,
            _ => EncryptedPreferencesError::from(PreferencesError::DeserializationError),
        })?;

        // Deserialize the values
        bincode::deserialize::<HashMap<String, Value>>(&decrypted)
            .map_err(|_| EncryptedPreferencesError::from(PreferencesError::DeserializationError))
    }

    /// If already exist a preference with the provided name will be loaded otherwise will be
    /// created a new empty one.
    ///
    /// * `name` - The preferences name, can contains only ascii alphanumeric chars or -, _.
    ///
    /// # Errors
    /// This function can return the following errors:
    /// * [EncryptedPreferencesError::DecryptionFailed] if the provided password is not valid or the
    /// data is corrupted.
    /// * [PreferencesError::InvalidName] if the provided name contains non ascii alphanumeric chars
    /// * [PreferencesError::DeserializationError] if the data inside the disc is not valid.
    /// * [PreferencesError::IO] if an error occurred while reading the data from the device storage.
    ///
    pub fn new(
        password: &str,
        name: &str,
    ) -> StdResult<EncryptedPreferences, EncryptedPreferencesError> {
        Ok(EncryptedPreferences {
            name: name.to_owned(),
            password: password.to_owned(),
            data: EncryptedPreferences::load_from_disk(password, name)?,
        })
    }
}

impl Preferences for EncryptedPreferences {
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.data.get(key).and_then(|v| match v {
            Value::I32(i32) => Some(i32.to_owned()),
            _ => None,
        })
    }

    fn put_i32(&mut self, key: &str, value: i32) -> Result<()> {
        self.data.insert(key.to_owned(), Value::I32(value));
        Ok(())
    }

    fn get_str(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|v| match v {
            Value::String(string) => Some(string.to_owned()),
            _ => None,
        })
    }

    fn put_str(&mut self, key: &str, value: String) -> Result<()> {
        self.data.insert(key.to_owned(), Value::String(value));
        Ok(())
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| match v {
            Value::Bool(bool) => Some(bool.to_owned()),
            _ => None,
        })
    }

    fn put_bool(&mut self, key: &str, value: bool) -> Result<()> {
        self.data.insert(key.to_owned(), Value::Bool(value));
        Ok(())
    }

    fn get_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).and_then(|v| match v {
            Value::Bin(bin) => Some(bin.to_owned()),
            _ => None,
        })
    }

    fn put_bytes(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.data.insert(key.to_owned(), Value::Bin(value));
        Ok(())
    }

    fn clear(&mut self) {
        self.data.clear()
    }

    fn erase(&mut self) {
        self.clear();
        io::erase(&self.name);
    }

    fn save(&self) -> Result<()> {
        let serialized =
            bincode::serialize(&self.data).map_err(|_| PreferencesError::SerializationError)?;

        let storage = Cocoon::new(self.password.as_ref());
        let encrypted = storage
            .wrap(&serialized)
            .map(base64::encode)
            .map_err(|_| PreferencesError::SerializationError)?;

        io::save(&self.name, &encrypted)?;
        Ok(())
    }
}

impl From<DecodeError> for PreferencesError {
    fn from(_: DecodeError) -> Self {
        PreferencesError::DeserializationError
    }
}

impl From<PreferencesError> for EncryptedPreferencesError {
    fn from(e: PreferencesError) -> Self {
        EncryptedPreferencesError::Preferences(Box::new(e))
    }
}

#[cfg(test)]
mod test {
    use crate::encrypted::EncryptedPreferences;
    use crate::preferences::Preferences;

    #[test]
    pub fn test_creation() {
        let encrypted = EncryptedPreferences::new("password", "test");

        assert!(encrypted.is_ok());
        encrypted.unwrap().erase();
    }

    #[test]
    pub fn test_invalid_names() {
        // Check invalid names
        assert!(EncryptedPreferences::new("", "test-").is_err());
        assert!(EncryptedPreferences::new("", "test.").is_err());
        assert!(EncryptedPreferences::new("", "test\\").is_err());
        assert!(EncryptedPreferences::new("", "test//").is_err());
        assert!(EncryptedPreferences::new("", "test with spaces").is_err());
        // Test empty
        assert!(EncryptedPreferences::new("", "").is_err());
    }

    #[test]
    pub fn test_data_read_write() {
        let set_name = "rwenc";
        let password = "password";

        let test_vec: Vec<u8> = vec![12, 13, 54, 42];
        let mut preferences = EncryptedPreferences::new(password, set_name).unwrap();
        assert!(preferences.put_i32("i32", 42).is_ok());
        assert!(preferences
            .put_str(
                "str",
                "some very long string with more than 32 bytes mf".to_owned()
            )
            .is_ok());
        assert!(preferences.put_bool("bool", true).is_ok());
        assert!(preferences.put_bytes("bin", test_vec.clone()).is_ok());

        // Write data to disk
        preferences.save().unwrap();

        // Create a new one that reads from the saved preferences
        let mut preferences = EncryptedPreferences::new(password, set_name).unwrap();
        let i32_result = preferences.get_i32("i32");
        let str_result = preferences.get_str("str");
        let bool_result = preferences.get_bool("bool");
        let binary_result = preferences.get_bytes("bin");

        // Delete the file from the disk to avoid that some date remain on the disk if the
        // test fails.
        preferences.erase();
        assert_eq!(42, i32_result.unwrap());
        assert_eq!(
            "some very long string with more than 32 bytes mf",
            str_result.unwrap()
        );
        assert_eq!(true, bool_result.unwrap());
        assert_eq!(test_vec, binary_result.unwrap());
    }
}
