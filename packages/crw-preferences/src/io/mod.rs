//! This crate provides a set of functions to save and load strings from the storage of the following
//! devices:
//! * windows
//! * macOS
//! * linux
//! * wasm32 on browser

use thiserror::Error;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod desktop;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use desktop as sys;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
use wasm as sys;

/// Struct that represents a generic I/O error.
#[derive(Error, Debug)]
pub enum IoError {
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

/// Loads a string from the device memory.
///
/// - *name* key that uniquely identify the string that will be loaded.
///
/// # Errors
/// This function can returns one of the following errors:
/// * [IoError::Read] - if an error occurred while reading the data from the device storage
/// * [IoError::EmptyData] - if the data associated to the provided `name` is empty
/// * [IoError::Unsupported] - if the device don't supports this operation
#[inline]
pub fn load(name: &str) -> Result<String> {
    sys::load(name)
}

/// Saves a string into the device memory.
///
/// - *name* key that uniquely identify the data that will be stored.
/// - *data* The string that will be stored into the device storage.
///
/// # Errors
/// This function can returns one of the following errors:
/// * [IoError::Write] - if an error occur while writing the data into the device storage
/// * [IoError::Unsupported] - if the device don't supports this operation
#[inline]
pub fn save(name: &str, data: &str) -> Result<()> {
    sys::save(name, data)
}

/// Erase the configurations stored into the device memory.
#[inline]
pub fn erase(name: &str) {
    sys::erase(name)
}
