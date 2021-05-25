use core::result;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::OpenOptions;

pub type Result<T> = result::Result<T, PreferencesError>;

#[derive(Debug)]
pub enum PreferencesError {
    InvalidName,
    NotFound,
    IO(String),
    DeserializationError,
    SerializationError,
}

pub struct Preferences {
    name: String,
    data: HashMap<String, String>,
}

impl Preferences {
    fn load_from_disk(name: &str) -> Result<HashMap<String, String>> {
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(PreferencesError::InvalidName);
        }

        // TODO: Handle read in a multiplatform environment
        let path = format!("./{}", name);
        let read_result = OpenOptions::new().read(true).open(&path);

        if read_result.is_err() {
            return Ok(HashMap::new());
        }

        let file = read_result.unwrap();
        let data: HashMap<String, String> = if let Ok(data) = serde_json::from_reader(file) {
            data
        } else {
            HashMap::new()
        };

        Ok(data)
    }

    fn write_to_disk(name: &str, data: &HashMap<String, String>) -> Result<()> {
        // TODO: Handle write in a multiplatform environment
        let path = format!("./{}", name);
        let out_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|_e| PreferencesError::IO(_e.to_string()))?;

        serde_json::to_writer(out_file, &data)
            .map_err(|_e| PreferencesError::SerializationError)?;

        Ok(())
    }

    /// Creates a new [Preferences] instance.
    ///
    /// * `name` - The preferences name, can contains only ascii alphanumeric chars.
    pub fn new(name: &str) -> Result<Preferences> {
        let data = Preferences::load_from_disk(name)?;

        Ok(Preferences {
            name: name.to_owned(),
            data,
        })
    }

    /// Retrive a value from the preferences.
    ///
    /// * `key` - The name of the preference to retrieve.
    pub fn get<'a, T>(&self, key: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if let Some(value) = self.data.get(key) {
            let result: serde_json::Result<T> = serde_json::from_str(value);
            return Ok(result.map_err(|_e| PreferencesError::DeserializationError)?);
        }

        Result::Err(PreferencesError::NotFound)
    }

    /// Store a value into the preferences.
    ///
    /// * `key` - The name of the preference that will be stored.
    pub fn put<T>(&mut self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let serialized =
            serde_json::to_string(value).map_err(|_e| PreferencesError::SerializationError)?;

        self.data.insert(key.to_owned(), serialized);

        Ok(())
    }

    /// Saves the data on the device disk
    pub fn save(&self) -> Result<()> {
        return Preferences::write_to_disk(&self.name, &self.data);
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::Preferences;
    use std::fs;

    #[test]
    pub fn test_preferences_save() {
        let preference_name = "test";

        let mut test_preferences = Preferences::new(preference_name).unwrap();
        test_preferences
            .put("data", &"some simple data".to_string())
            .unwrap();

        test_preferences.save().unwrap();

        assert_eq!(
            "some simple data",
            test_preferences.get::<String>("data").unwrap()
        );

        // Delete the preferences file
        fs::remove_file(preference_name).unwrap();
    }

    #[test]
    pub fn test_invalid_names() {
        // Check invalid names
        assert!(Preferences::new("test-").is_err());
        assert!(Preferences::new("test.").is_err());
        assert!(Preferences::new("test\\").is_err());
        assert!(Preferences::new("test//").is_err());
        assert!(Preferences::new("pippo pluto").is_err());
        assert!(Preferences::new("").is_err());
        // Only ascii alphanumeric are allowed
        assert!(Preferences::new("test").is_ok());
        assert!(Preferences::new("test1").is_ok());
    }
}
