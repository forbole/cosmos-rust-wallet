//! Module that provides functions to read and write data from the browser local storage.

use crate::io::{IoError, Result};
use web_sys::{Storage, Window};

/// Gets the browser `LocalStorage` instance.
///
/// # Errors
/// This function returns [IoError::Unsupported] if the browser don't support the LocalStorage API
/// or the global window object was not found.
fn get_storage() -> Result<Storage> {
    let window = web_sys::window().ok_or(IoError::Unsupported(
        "global `window` object not found".to_owned(),
    ))?;

    Ok(Window::local_storage(&window)
        .map_err(|_| IoError::Unsupported("Local storage not supported".to_owned()))?
        .ok_or(IoError::Unsupported("Local storage is null".to_owned()))?)
}

/// Loads the a string from the browse `LocalStorage`.
///
/// * `name` - key that uniquely identify the data that will be loaded.
pub fn load(name: &str) -> Result<String> {
    let storage = get_storage()?;

    Storage::get_item(&storage, name)
        .map_err(|_| IoError::Read)?
        .ok_or(IoError::EmptyData)
        .and_then(|s| {
            if s.is_empty() {
                Err(IoError::EmptyData)
            } else {
                Ok(s)
            }
        })
}

/// Saves a string into the browser `LocalStorage`.
///
/// * `name` - key that uniquely identify the data that will be saved.
/// * `value` - the value that will be saved into the browser localStorage.
///
/// # Errors
/// This function returns [Err(IoError::Unsupported)] if the browser don't support the LocalStorage API
/// or [Err(IoError::Write)] if an error occur when writing the data to the browser local storage.
pub fn save(name: &str, value: &str) -> Result<()> {
    let storage = get_storage()?;

    Storage::set_item(&storage, name, value).map_err(|_| IoError::Write)
}

/// Deletes the data from the browser `LocalStorage`
pub fn erase(name: &str) {
    let storage = get_storage();

    if let Ok(storage) = storage {
        // Make the compiler happy, an error here should neve occur.
        let _ = Storage::set_item(&storage, name, "");
    }
}

/// Check if is present a non empty string into the browser `LocalStorage`.
pub fn exist(name: &str) -> bool {
    let storage = get_storage();

    return if storage.is_err() {
        false
    } else {
        let storage = storage.unwrap();
        let item_result = Storage::get_item(&storage, name);

        if item_result.is_err() {
            false
        } else {
            let item = item_result.unwrap().unwrap_or("".to_owned());
            !item.is_empty()
        }
    };
}
