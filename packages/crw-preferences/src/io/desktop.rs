//! Module that provides the functions to read write from a device disk.  
//! This module supports the following devices:
//! * windows
//! * macOS
//! * linux.

use crate::io::{IoError, Result};
use std::fs::File;
use std::path::PathBuf;
use std::{fs, result::Result as StdResult};

/// Gets a file with the provided name from the application configuration directory.
///
/// * `name` - the name of the file requested from the user.
/// * `create` - if true creates also the file if not exist.
///
/// The file is located inside the application config directory that depends on the target device OS.
///
/// |Platform | Value                                 | Example                                                            |
/// | ------- | ------------------------------------- | -------------------------------------------------------------------|
/// | Linux   | `$XDG_CONFIG_HOME` or `$HOME`/.config | /home/alice/.config/$CARGO_BIN_NAME/{name}                         |
/// | macOS   | `$HOME`/Library/Application Support   | /Users/Alice/Library/Application Support/$CARGO_BIN_NAME/{name}    |
/// | Windows | `{FOLDERID_RoamingAppData}`           | C:\Users\Alice\AppData\Roaming\%CARGO_BIN_NAME%\{name}             |
///
/// # Errors
///
/// This function return an [std::io::Error] if the file can't be created inside the configuration directory.
fn get_config_file(name: &str, create: bool) -> StdResult<PathBuf, std::io::Error> {
    cfg_if! {
        if #[cfg(test)] {
            // In test mode just use the current working directory.
            let mut config_dir = PathBuf::new();
        }
        else {
            // The application name is resolved as compile from the cargo project name.
            let bin_name: &'static str = env!("CARGO_BIN_NAME");

            let mut config_dir = dirs::config_dir().unwrap();
            // Append the binary name to the default config dir
            config_dir.push(bin_name);
            // Check if the directory exists, if not create it.
            if !config_dir.exists() {
                fs::create_dir_all(config_dir.as_path())?;
            }
        }
    }

    // Append the name provided from the user
    config_dir.push(name);
    // Check if the file exists, if not create an empty one.
    if create && !config_dir.exists() {
        File::create(config_dir.as_path())?;
    }
    // Returns the config file path.
    Ok(config_dir)
}

/// Loads data from the file with the provided name.
///
/// * `name` - Name of the configuration file from which will be loaded the data.
///
/// # Errors
/// This function can returns one of the following errors:
/// * [IoError::Read] if the file with the provided `name` can't be read
/// * [IoError::EmptyData] if the file is empty
pub fn load(name: &str) -> Result<String> {
    get_config_file(name, true)
        .and_then(fs::read_to_string)
        .map_err(|_| IoError::Read)
        .and_then(|data| {
            if data.is_empty() {
                Err(IoError::EmptyData)
            } else {
                Ok(data)
            }
        })
}

/// Saves `data` into the configuration file with the provided `name`.
///
/// * `name` - Name of the configuration file where will be stored the data.
/// * `data` - The string that will be stored inside the file.
///
/// # Errors
/// This function returns [IoError::Write] if can't write to the file with the provided `name`.
pub fn save(name: &str, data: &str) -> Result<()> {
    get_config_file(name, true)
        .and_then(|path| fs::write(path, data))
        .map_err(|_| IoError::Write)
}

/// Deletes the file with the provide `name` from the device storage.
pub fn erase(name: &str) {
    let path = get_config_file(name, false);
    if let Ok(path) = path {
        if path.exists() {
            // Make the compiler happy, an error here should never occur.
            let _ = fs::remove_file(path);
        }
    }
}
