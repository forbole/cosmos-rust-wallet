//! Module that provides the functions to read write from a device disk.  
//! This module supports the following devices:
//! * windows
//! * macOS
//! * linux.

use crate::io::{IoError, Result};
use once_cell::sync::Lazy;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Mutex;

/// Global variable that contains the directory name where will be stored the configurations files.
static PREFERENCES_APP_DIR: Lazy<Mutex<String>> = Lazy::new(|| {
    let str = option_env!("CARGO_BIN_NAME").unwrap_or("").to_owned();
    Mutex::new(str)
});

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
fn get_config_file(name: &str, create: bool) -> Result<PathBuf> {
    cfg_if! {
        if #[cfg(test)] {
            // In test mode just use the current working directory.
            let mut config_dir = PathBuf::new();
        }
        else if #[cfg(target_os = "android")] {
            // On android we can't obtain the path at runtime so the app dir must be an
            // absolute path to the directory where will be stored the configurations.
            let dir = PREFERENCES_APP_DIR.lock().unwrap();
            if dir.is_empty() {
                return Err(IoError::EmptyPreferencesPath);
            }
            let mut config_dir = PathBuf::from(dir.as_str());
        }
        else {
            // The application name is resolved as compile from the cargo project name.
            let dir = PREFERENCES_APP_DIR.lock().unwrap();
            if dir.is_empty() {
                return Err(IoError::EmptyPreferencesPath);
            }
            let mut config_dir = dirs::config_dir().unwrap();
            // Append the binary name to the default config dir
            config_dir.push(dir.as_str());
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

/// Sets the application directory where will be stored the configurations.
/// On windows, macOS and linux `dir` should be only the name of the directory that will
/// be create inside the current user app configurations directory.
/// On Android instead since is not possible to obtain the `appData` directory at runtime
/// `dir` must be an absolute path to a directory where the application can read and write.
pub fn set_preferences_app_dir(dir: &str) {
    let mut str = PREFERENCES_APP_DIR.lock().unwrap();
    str.clear();
    str.push_str(dir);
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
    let config_file = get_config_file(name, true)?;
    let content = fs::read_to_string(config_file)?;

    if content.is_empty() {
        Err(IoError::EmptyData)
    } else {
        Ok(content)
    }
}

/// Saves `data` into the configuration file with the provided `name`.
///
/// * `name` - Name of the configuration file where will be stored the data.
/// * `data` - The string that will be stored inside the file.
///
/// # Errors
/// This function returns [IoError::Write] if can't write to the file with the provided `name`.
pub fn save(name: &str, data: &str) -> Result<()> {
    let config_file = get_config_file(name, true)?;
    fs::write(config_file, data)?;
    Ok(())
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

/// Check if exist a file with the provided `name` into the device storage.
pub fn exist(name: &str) -> bool {
    let path = get_config_file(name, false);

    if let Ok(p) = path {
        p.exists()
    } else {
        false
    }
}
