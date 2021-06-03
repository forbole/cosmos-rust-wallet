//! Module that provides an implementation of [Preferences] that saves the value into the device
//! storage.

use crate::io;
use crate::preferences::{Preferences, PreferencesError, Result};
use serde_json::{Map, Value};

pub struct UnencryptedPreferences {
    name: String,
    data: Map<String, Value>,
}

impl UnencryptedPreferences {
    /// Functions to check if the preference set name is valid.
    /// A name to be valid must contains only ascii alphanumeric chars.
    ///
    /// * `name` - The that will be checked.
    fn is_name_valid(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric())
    }

    /// Loads the json preferences from the device storage.
    ///
    /// * `name` - The preferences set name.
    ///
    /// # Errors
    /// This function returns [PreferencesError::DeserializationError] if the data loaded
    /// from the disk is not valid.
    fn load_from_disk(name: &str) -> Result<Map<String, Value>> {
        let disk_data = io::load(name);

        if disk_data.is_err() {
            return Ok(Map::new());
        }

        Ok(serde_json::from_str(&disk_data.unwrap())
            .map_err(|_| PreferencesError::DeserializationError)?)
    }

    /// Writes the data as json to the device storage.
    ///
    /// * `name` - The preference set name.
    /// * `data` - The data that will be wrote to the device storage.
    ///
    /// # Errors
    /// This function returns the following errors
    /// * [PreferencesError::IO] - if an error occurs while writing the data to the device storage
    /// * [PreferencesError::SerializationError] - if an error occurs while serializing the data.
    fn write_to_disk(name: &str, data: &Map<String, Value>) -> Result<()> {
        let json =
            serde_json::to_string(&data).map_err(|_| PreferencesError::SerializationError)?;
        io::save(name, &json)?;

        Ok(())
    }

    /// Creates a new [UnencryptedPreferences] with the provided name.
    /// If already exist a preference with the provided name will be loaded otherwise will be
    /// created a new empty one.
    ///
    /// * `name` - The preferences name, can contains only ascii alphanumeric chars.
    ///
    /// # Errors
    /// This function returns [PreferencesError::InvalidName] if the provided name contains
    /// non ascii alphanumeric chars or [PreferencesError::DeserializationError] if the data
    /// associated with the provided name are invalid.
    ///
    pub fn new(name: &str) -> Result<UnencryptedPreferences> {
        if !UnencryptedPreferences::is_name_valid(name) {
            return Err(PreferencesError::InvalidName(name.to_owned()));
        }

        let data = UnencryptedPreferences::load_from_disk(name)?;

        Ok(UnencryptedPreferences {
            name: name.to_owned(),
            data,
        })
    }
}

impl Preferences for UnencryptedPreferences {
    fn get_i32(&self, key: &str) -> Option<i32> {
        self.data.get(key).and_then(|v| {
            v.as_i64().and_then(|i| {
                if i >= (i32::MIN as i64) && i <= (i32::MAX as i64) {
                    Some(i as i32)
                } else {
                    None
                }
            })
        })
    }

    fn put_i32(&mut self, key: &str, value: i32) -> Result<()> {
        self.data.insert(key.to_owned(), Value::from(value));
        Ok(())
    }

    fn get_str(&self, key: &str) -> Option<String> {
        self.data
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_owned()))
    }

    fn put_str(&mut self, key: &str, value: String) -> Result<()> {
        self.data.insert(key.to_owned(), Value::from(value));
        Ok(())
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(|v| v.as_bool())
    }

    fn put_bool(&mut self, key: &str, value: bool) -> Result<()> {
        self.data.insert(key.to_owned(), Value::from(value));
        Ok(())
    }

    fn get_binary(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).and_then(|v| v.as_array()).map(|v| {
            let mut vector: Vec<u8> = Vec::with_capacity(v.len());
            for value in v {
                if value.is_u64() {
                    vector.push(value.as_u64().unwrap() as u8)
                }
            }
            vector
        })
    }

    fn put_binary(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.data.insert(key.to_owned(), Value::from(value));
        Ok(())
    }

    fn clear(&mut self) {
        self.data.clear()
    }

    fn erase(&mut self) {
        self.clear();
        io::erase(&self.name)
    }

    fn save(&self) -> Result<()> {
        return UnencryptedPreferences::write_to_disk(&self.name, &self.data);
    }
}

#[cfg(test)]
mod tests {
    use crate::preferences::Preferences;
    use crate::unencrypted::UnencryptedPreferences;

    #[test]
    pub fn test_preferences_save() {
        let mut test_preferences = UnencryptedPreferences::new("save").unwrap();
        test_preferences
            .put_str("data", "some simple data".to_owned())
            .unwrap();

        test_preferences.save().unwrap();

        assert_eq!(
            "some simple data",
            test_preferences.get_str("data").unwrap()
        );
        test_preferences.erase();
    }

    #[test]
    pub fn test_invalid_names() {
        // Check invalid names
        assert!(UnencryptedPreferences::new("test-").is_err());
        assert!(UnencryptedPreferences::new("test.").is_err());
        assert!(UnencryptedPreferences::new("test\\").is_err());
        assert!(UnencryptedPreferences::new("test//").is_err());
        assert!(UnencryptedPreferences::new("test with spaces").is_err());
        // Test empty
        assert!(UnencryptedPreferences::new("").is_err());

        // Only ascii alphanumeric are allowed
        let p = UnencryptedPreferences::new("TeSt42");
        assert!(p.is_ok());
        p.unwrap().erase();
    }

    #[test]
    pub fn test_data_read_write() {
        let set_name = "rw";
        let test_vec: Vec<u8> = vec![12, 13, 54, 42];
        let mut preferences = UnencryptedPreferences::new(set_name).unwrap();
        assert!(preferences.put_i32("i32", 42).is_ok());
        assert!(preferences.put_str("str", "str".to_owned()).is_ok());
        assert!(preferences.put_bool("bool", true).is_ok());
        assert!(preferences.put_binary("bin", test_vec.clone()).is_ok());

        // Write data to disk
        preferences.save().unwrap();

        // Create a new one that reads from the saved preferences
        let mut preferences = UnencryptedPreferences::new(set_name).unwrap();
        let i32_result = preferences.get_i32("i32");
        let str_result = preferences.get_str("str");
        let bool_result = preferences.get_bool("bool");
        let binary_result = preferences.get_binary("bin");

        // Delete the file from the disk to avoid that some date remain on the disk if the
        // test fails.
        preferences.erase();
        assert_eq!(42, i32_result.unwrap());
        assert_eq!("str", str_result.unwrap());
        assert_eq!(true, bool_result.unwrap());
        assert_eq!(test_vec, binary_result.unwrap());
    }
}
