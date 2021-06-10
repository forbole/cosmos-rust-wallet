//! Module that provides the FFI to access the preferences from other programming languages.

use crate::encrypted::EncryptedPreferences;
use crate::preferences::Preferences;
use crate::unencrypted::UnencryptedPreferences;
use ffi_helpers;
use libc::{c_char, c_uchar, c_void};
use std::ptr::null_mut;
use std::slice;

// Macro to export the ffi_helpers's functions used to access the error message from other programming languages.
export_error_handling_functions!();

/// Macro that converts a raw C string to a &str breaking the code flow if the provided string
/// is a null ptr or an invalid UTF-8 string.
/// Other then breaking the code flow this macro also update a global variable that
/// contains an error message that tells the error cause.
///
/// # Example
///
/// For example we want to create a function that computes the length of a c string
/// and returns 0 if the provided string is invalid.
///
/// ```
/// use libc::c_char;
/// pub fn c_string_length(str: *const c_char) -> usize {
///     let str = !str_or_return(str, 0);
///     str.len()
/// }
/// ```
///
///
macro_rules! check_str {
    ($ptr:expr, $rc:expr) => {{
        use std::ffi::CStr;
        if $ptr.is_null() {
            return $rc;
        }
        let c_str: &CStr = unsafe { CStr::from_ptr($ptr) };
        let str_slice = c_str.to_str();
        if let Err(e) = str_slice {
            ffi_helpers::update_last_error(e);
            return $rc;
        }
        str_slice.unwrap()
    }};
}

fn box_to_c_ptr<T: Preferences>(preferences: T) -> *mut c_void {
    let boxed: Box<dyn Preferences> = Box::new(preferences);
    Box::into_raw(Box::new(boxed)) as *mut c_void
}

unsafe fn unbox_from_c_ptr(ptr: *mut c_void) -> Box<Box<dyn Preferences>> {
    let ptr = unwrap_ptr_mut(ptr);
    Box::from_raw(ptr)
}

fn unwrap_ptr(ptr: *const c_void) -> *const Box<dyn Preferences> {
    ptr as *const Box<dyn Preferences>
}

fn unwrap_ptr_mut(ptr: *mut c_void) -> *mut Box<dyn Preferences> {
    ptr as *mut Box<dyn Preferences>
}

/// Creates a new preferences with the provided name or load an already existing preferences
/// with the provided name.
///
/// - *name* The preferences name, can contains only ascii alphanumeric chars or -, _.
///
/// Returns a valid pointer on success or nullptr if an error occurred.
#[no_mangle]
pub extern "C" fn preferences(name: *const c_char) -> *mut c_void {
    let name = check_str!(name, null_mut());

    match UnencryptedPreferences::new(name) {
        Err(e) => {
            ffi_helpers::update_last_error(e);
            return null_mut();
        }
        Ok(p) => box_to_c_ptr(p),
    }
}

/// Creates a new encrypted preferences with the provided name or load an already existing preferences
/// with the provided name.
///
/// - *name* The preferences name, can contains only ascii alphanumeric chars or -, _.
/// - *password* The password used to secure the preferences.
///
/// Returns a valid pointer on success or nullptr if an error occurred.
#[no_mangle]
pub extern "C" fn encrypted_preferences(
    name: *const c_char,
    password: *const c_char,
) -> *mut c_void {
    let name = check_str!(name, null_mut());
    let password = check_str!(password, null_mut());

    match EncryptedPreferences::new(password, name) {
        Err(e) => {
            ffi_helpers::update_last_error(e);
            return null_mut();
        }
        Ok(p) => box_to_c_ptr(p),
    }
}

/// Release all the resources owned by a preferences instance.
#[no_mangle]
pub extern "C" fn preferences_free(preferences: *mut c_void) {
    if preferences.is_null() {
        return;
    }

    // Reclaim the boxed preferences to destroy.
    unsafe { unbox_from_c_ptr(preferences) };
}

/// Gets an i32 from the preferences.
///
/// - *preferences* pointer to the preferences from which will be extracted the value.
/// - *key* name of the preference that will be loaded.
/// - *out* pointer where will be stored the value.
///
/// Returns 0 on success -1 if the requested value is not present into the preferences or -2
/// if one or more of the provided arguments is invalid.
#[no_mangle]
pub extern "C" fn preferences_get_i32(
    preferences: *const c_void,
    key: *const c_char,
    out: *mut i32,
) -> i32 {
    if preferences.is_null() || out.is_null() {
        return -2;
    }

    let key = check_str!(key, -2);
    let preferences = unsafe { unwrap_ptr(preferences).as_ref().unwrap() };
    match preferences.get_i32(key) {
        Some(i) => {
            unsafe { *out = i };
            0
        }
        None => -1,
    }
}

/// Puts an i32 into the preferences.
///
/// - *preferences* pointer to the preferences where will be stored the value.
/// - *key* name of the preference that will be stored.
/// - *value* the value that will be stored.
///
/// Returns 0 on success or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_put_i32(
    preferences: *mut c_void,
    key: *const c_char,
    value: i32,
) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let key = check_str!(key, -1);
    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };
    match preferences.put_i32(key, value) {
        Ok(_) => 0,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            -1
        }
    }
}

/// Gets a string from the preferences.
///
/// - *preferences* pointer to the preferences from which will be extracted the value.
/// - *key* name of the preference that will be loaded.
/// - *out_buf* pointer where will be stored the value.
/// - *buf_len* maximum number of bytes that can be used from `out_buf`
///
/// Returns the number of bytes that would have been written if `out_buf` had been sufficiently large,
/// 0 if the value is not present into the preferences or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_get_string(
    preferences: *const c_void,
    key: *const c_char,
    out_buf: *mut c_uchar,
    len: usize,
) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let key = check_str!(key, -1);
    let preferences = unsafe { unwrap_ptr(preferences).as_ref().unwrap() };
    match preferences.get_str(key) {
        Some(s) => {
            let bytes = s.as_bytes();
            // Use bytes len instead of the string since in UTF-8 strings the length can be
            // different from the number of bytes that represents the string.
            if bytes.len() <= len {
                let out_slice: &mut [u8] =
                    unsafe { slice::from_raw_parts_mut(out_buf as *mut u8, bytes.len()) };
                out_slice.copy_from_slice(s.as_bytes())
            }
            s.len() as i32
        }
        None => 0,
    }
}

/// Puts a string into the preferences.
///
/// - *preferences* pointer to the preferences from which will be extracted the value.
/// - *key* name of the preference that will be loaded.
/// - *value* the value that will be stored.
///
/// Returns 0 on success -1 on error.
#[no_mangle]
pub extern "C" fn preferences_put_string(
    preferences: *mut c_void,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let key = check_str!(key, -1);
    let value = check_str!(value, -1);
    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };

    match preferences.put_str(key, value.to_owned()) {
        Ok(_) => 0,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            -1
        }
    }
}

/// Gets a bool from the preferences.
///
/// - *preferences* pointer to the preferences from which will be extracted the value.
/// - *key* name of the preference that will be loaded.
/// - *out* pointer where will be stored the value.
///
/// Returns 0 on success -1 if the requested value is not present into the preferences or -2
/// if one or more of the provided arguments is invalid.
#[no_mangle]
pub extern "C" fn preferences_get_bool(
    preferences: *const c_void,
    key: *const c_char,
    out: *mut bool,
) -> i32 {
    if preferences.is_null() || out.is_null() {
        return -2;
    }

    let key = check_str!(key, -2);
    let preferences = unsafe { unwrap_ptr(preferences).as_ref().unwrap() };
    match preferences.get_bool(key) {
        Some(b) => {
            unsafe { *out = b };
            0
        }
        None => -1,
    }
}

/// Puts a bool into the preferences.
///
/// - *preferences* pointer to the preferences where will be stored the value.
/// - *key* name of the preference that will be stored.
/// - *value* the value that will be stored.
///
/// Returns 0 on success or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_put_bool(
    preferences: *mut c_void,
    key: *const c_char,
    value: bool,
) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let key = check_str!(key, -1);
    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };
    match preferences.put_bool(key, value) {
        Ok(_) => 0,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            -1
        }
    }
}

/// Gets an array of bytes from the preferences.
///
/// - *preferences* pointer to the preferences from which will be extracted the value.
/// - *key* name of the preference that will be loaded.
/// - *out_buf* pointer where will be stored the value.
/// - *buf_len* maximum number of bytes that can be used from `out_buf`
///
/// Returns the number of bytes that would have been written if `out_buf` had been sufficiently large,
/// 0 if the value is not present into the preferences or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_get_binary(
    preferences: *const c_void,
    key: *const c_char,
    out_buf: *mut u8,
    buf_len: usize,
) -> i32 {
    if preferences.is_null() || out_buf.is_null() {
        return -1;
    }

    let key = check_str!(key, -1);
    let preferences = unsafe { unwrap_ptr(preferences).as_ref().unwrap() };

    match preferences.get_binary(key) {
        Some(v) => {
            if v.len() <= buf_len {
                // The buffer is large enough copy the vec to the dest buffer
                let dest: &mut [u8] = unsafe { slice::from_raw_parts_mut(out_buf, v.len()) };
                dest.copy_from_slice(v.as_slice());
            }
            v.len() as i32
        }
        None => 0,
    }
}

/// Puts an array of bytes into the preferences.
///
/// - *preferences* pointer to the preferences from which will be extracted the value.
/// - *key* name of the preference that will be stored.
/// - *value* array that will be stored into the preferences.
/// - *len* length of `value`.
///
/// Return 0 on on success, -1 on error.
#[no_mangle]
pub extern "C" fn preferences_put_binary(
    preferences: *mut c_void,
    key: *const c_char,
    value: *const u8,
    len: usize,
) -> i32 {
    if preferences.is_null() || value.is_null() {
        return -1;
    }

    let key = check_str!(key, -1);
    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };
    let value = unsafe { slice::from_raw_parts(value, len) };

    match preferences.put_binary(key, value.to_owned()) {
        Ok(_) => 0,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            -1
        }
    }
}

/// Delete all the preferences currently loaded from the provided preferences instance.
///
/// - *preferences* pointer to the preferences instance.
///
/// Returns 0 on success or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_clear(preferences: *mut c_void) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };
    preferences.clear();
    0
}

/// Delete all the preferences currently loaded and also the one stored into the
/// device storage from the provided preferences instance
///
/// - *preferences* pointer to the preferences instance.
///
/// Returns 0 on success or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_erase(preferences: *mut c_void) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };
    preferences.erase();
    0
}

/// Saves the preferences into the device disk.
///
/// - *preferences* pointer to the preferences instance.
///
/// Returns 0 on success or -1 on error.
#[no_mangle]
pub extern "C" fn preferences_save(preferences: *mut c_void) -> i32 {
    if preferences.is_null() {
        return -1;
    }

    let preferences = unsafe { unwrap_ptr_mut(preferences).as_mut().unwrap() };
    match preferences.save() {
        Ok(_) => 0,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::{
        encrypted_preferences, preferences, preferences_erase, preferences_free,
        preferences_get_binary, preferences_get_bool, preferences_get_i32, preferences_get_string,
        preferences_put_binary, preferences_put_bool, preferences_put_i32, preferences_put_string,
        preferences_save,
    };
    use std::ffi::CString;

    #[test]
    fn test_preferences_creation() {
        let preferences_name = CString::new("creation").unwrap();

        let raw_preferences = preferences(preferences_name.as_ptr());
        assert!(!raw_preferences.is_null());

        preferences_erase(raw_preferences);
        preferences_free(raw_preferences);
    }

    #[test]
    fn test_encrypted_preferences_creation() {
        let preferences_name = CString::new("encrypted").unwrap();
        let preferences_password = CString::new("password").unwrap();

        let raw_preferences =
            encrypted_preferences(preferences_name.as_ptr(), preferences_password.as_ptr());
        assert!(!raw_preferences.is_null());

        preferences_erase(raw_preferences);
        preferences_free(raw_preferences);
    }

    #[test]
    fn test_put_i32() {
        let preferences_name = CString::new("ffi").unwrap();
        let raw_preferences = preferences(preferences_name.as_ptr());
        assert!(!raw_preferences.is_null());

        let i32_key = CString::new("i32").unwrap();
        let insert_rc = preferences_put_i32(raw_preferences, i32_key.as_ptr(), 42);
        assert_eq!(0, insert_rc);

        let save_rc = preferences_save(raw_preferences);
        assert_eq!(0, save_rc);

        let mut read_val = 0;
        let read_rc = preferences_get_i32(raw_preferences, i32_key.as_ptr(), &mut read_val);
        assert_eq!(0, read_rc);
        assert_eq!(42, read_val);

        let erase_rc = preferences_erase(raw_preferences);
        assert_eq!(0, erase_rc);

        preferences_free(raw_preferences);
    }

    #[test]
    fn test_put_string() {
        let preferences_name = CString::new("string").unwrap();
        let raw_preferences = preferences(preferences_name.as_ptr());
        assert!(!raw_preferences.is_null());

        let str_key = CString::new("str").unwrap();
        let str_value = CString::new("value").unwrap();
        let insert_rc =
            preferences_put_string(raw_preferences, str_key.as_ptr(), str_value.as_ptr());
        assert_eq!(0, insert_rc);

        let save_rc = preferences_save(raw_preferences);
        assert_eq!(0, save_rc);

        let mut str_bytes = [0u8; 32];
        let read_rc = preferences_get_string(
            raw_preferences,
            str_key.as_ptr(),
            str_bytes.as_mut_ptr(),
            str_bytes.len(),
        );
        assert_eq!(5, read_rc);
        let str = String::from_utf8(str_bytes[0..5].to_vec()).unwrap();
        assert_eq!("value", str);

        let erase_rc = preferences_erase(raw_preferences);
        assert_eq!(0, erase_rc);

        preferences_free(raw_preferences);
    }

    #[test]
    fn test_put_bool() {
        let preferences_name = CString::new("bool").unwrap();
        let raw_preferences = preferences(preferences_name.as_ptr());
        assert!(!raw_preferences.is_null());

        let key = CString::new("bool").unwrap();
        let insert_rc = preferences_put_bool(raw_preferences, key.as_ptr(), true);
        assert_eq!(0, insert_rc);

        let save_rc = preferences_save(raw_preferences);
        assert_eq!(0, save_rc);

        let mut read_bool = false;
        let read_rc = preferences_get_bool(raw_preferences, key.as_ptr(), &mut read_bool);
        assert_eq!(0, read_rc);
        assert_eq!(true, read_bool);

        let erase_rc = preferences_erase(raw_preferences);
        assert_eq!(0, erase_rc);

        preferences_free(raw_preferences);
    }

    #[test]
    fn test_put_binary() {
        let preferences_name = CString::new("binary").unwrap();
        let raw_preferences = preferences(preferences_name.as_ptr());
        assert!(!raw_preferences.is_null());

        let key = CString::new("bin").unwrap();
        let bin = [1u8, 2, 4, 5];
        let insert_rc =
            preferences_put_binary(raw_preferences, key.as_ptr(), bin.as_ptr(), bin.len());
        assert_eq!(0, insert_rc);

        let mut read_buf = [0u8; 10];
        let read_rc = preferences_get_binary(
            raw_preferences,
            key.as_ptr(),
            read_buf.as_mut_ptr(),
            read_buf.len(),
        );
        assert_eq!(4, read_rc);
        assert_eq!(bin, read_buf[0..4]);

        let erase_rc = preferences_erase(raw_preferences);
        assert_eq!(0, erase_rc);

        preferences_free(raw_preferences);
    }
}
