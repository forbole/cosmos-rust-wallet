//! This crate provides a set of functions to save and load the preferences from the storage of the
//! following devices:
//! * windows
//! * macOS
//! * linux
//! * android
//! * ios
//! * wasm32 on browser

use std::io::{Error as StdIoError, Error};
use thiserror::Error;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
use native as sys;

#[cfg(not(target_arch = "wasm32"))]
pub use native::set_preferences_app_dir;

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "js",))]
mod wasm;
#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "js"))]
use wasm as sys;

/// Struct that represents a generic I/O error.
#[derive(Error, Debug)]
pub enum IoError {
    #[error("io error `{0}`")]
    Std(StdIoError),
    #[error("the preferences app directory was not initialized")]
    EmptyPreferencesPath,
    #[error("invalid name `{0}`")]
    InvalidName(String),
    #[error("the preferences are empty")]
    EmptyData,
    #[error("error while reading the data")]
    Read,
    #[error("error while writing the data")]
    Write,
    #[error("i/o operation not supported `{0}`")]
    Unsupported(String),
    #[error("unknown i/o error `{0}`")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, IoError>;

/// Functions to check if a key used to access the storage is valid.
///
/// * `name` - The that will be checked.
fn is_name_valid(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ['-', '_'].contains(&c))
}

/// Loads the string representation of a preferences set.
///
/// * `name` - key that uniquely identify the preferences set that will be loaded.  
/// The `name` key can contain only ascii alphanumeric characters or -, _.
///
/// # Errors
/// This function can returns one of the following errors:
/// * [IoError::Read] - if an error occurred while reading the data from the device storage
/// * [IoError::EmptyData] - if the data associated to the provided `name` is empty
/// * [IoError::Unsupported] - if the device don't supports this operation
pub fn load(name: &str) -> Result<String> {
    if is_name_valid(name) {
        sys::load(name)
    } else {
        Err(IoError::InvalidName(name.to_owned()))
    }
}

/// Saves the string representation of preferences set into the device storage.
///
/// * `name` - key that uniquely identify the preferences set that will be saved.  
/// The `name` key can contain only ascii alphanumeric characters or -, _.
/// * `data` - the preferences set that will be stored as a string.
///
/// # Errors
/// This function can returns one of the following errors:
/// * [IoError::Write] - if an error occur while writing the data into the device storage
/// * [IoError::Unsupported] - if the device don't supports this operation
pub fn save(name: &str, data: &str) -> Result<()> {
    if is_name_valid(name) {
        sys::save(name, data)
    } else {
        Err(IoError::InvalidName(name.to_owned()))
    }
}

/// Erase a preferences set stored into the device memory.
pub fn erase(name: &str) {
    if is_name_valid(name) {
        sys::erase(name)
    }
}

/// Check if exist a preferences set withe the provided name into the device storage.
pub fn exist(name: &str) -> bool {
    if !is_name_valid(name) {
        return false;
    }

    sys::exist(name)
}

impl From<StdIoError> for IoError {
    fn from(e: Error) -> Self {
        IoError::Std(e)
    }
}
