//! Provides the FFI to interact with [`MnemonicWallet`] from other programming languages.
use crate::crypto::MnemonicWallet;
use crate::WalletError;
use bip39::{Language, Mnemonic, MnemonicType};
use ffi_helpers::error_handling::error_message_utf8;
use libc::{c_char, c_uchar};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::c_uint;
use std::ptr::null_mut;

#[repr(C)]
pub struct Signature {
    len: c_uint,
    data: *mut c_uchar,
}

/// Release a string returned from rust.
#[no_mangle]
pub extern "C" fn cstring_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}

/// Generates a random mnemonic of 24 words.
/// The returned mnemonic must bee freed using the [`cstring_free`] function to avoid memory leaks.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [`error_message_utf8`] function.
#[no_mangle]
pub extern "C" fn wallet_random_mnemonic() -> *mut c_char {
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
    let phrase = mnemonic.phrase().to_owned();

    let phrase_c_str = match CString::new(phrase) {
        Ok(s) => s,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            return null_mut();
        }
    };

    phrase_c_str.into_raw()
}

/// Derive a Secp256k1 key pair from the given mnemonic_phrase and derivation_path.
/// The returned [`MnemonicWallet`] ptr must bee freed using the [`wallet_free`] function to avoid memory
/// leaks.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [`error_message_utf8`] function.
#[no_mangle]
pub extern "C" fn wallet_from_mnemonic(
    mnemonic: *const c_char,
    derivation_path: *const c_char,
) -> *mut MnemonicWallet {
    if mnemonic.is_null() {
        ffi_helpers::update_last_error(WalletError::Mnemonic("null mnemonic ptr".to_owned()));
        return null_mut();
    }
    if derivation_path.is_null() {
        ffi_helpers::update_last_error(WalletError::DerivationPath(
            "null derivation path ptr".to_owned(),
        ));
        return null_mut();
    }

    let mnemonic_c_str = unsafe { CStr::from_ptr(mnemonic).to_string_lossy() };
    let dp_c_str = unsafe { CStr::from_ptr(derivation_path).to_string_lossy() };

    let wallet = match MnemonicWallet::new(mnemonic_c_str.as_ref(), dp_c_str.as_ref()) {
        Ok(w) => w,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            return null_mut();
        }
    };

    Box::into_raw(Box::new(wallet))
}

/// Deallocate a [`MnemonicWallet`] instance.
#[no_mangle]
pub extern "C" fn wallet_free(ptr: *mut MnemonicWallet) {
    if ptr.is_null() {
        return;
    }

    Box::from(ptr);
}

/// Gets the bech32 address derived from the mnemonic and the provided human readable part.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [`error_message_utf8`] function.
#[no_mangle]
pub extern "C" fn wallet_get_bech32_address(
    ptr: *const MnemonicWallet,
    hrp: *const c_char,
) -> *mut c_char {
    null_pointer_check!(ptr);

    if hrp.is_null() {
        ffi_helpers::update_last_error(WalletError::Hrp("received null hrp".to_owned()));
        return null_mut();
    }

    let hrp_cstr = unsafe { CStr::from_ptr(hrp).to_string_lossy() };

    let address = unsafe {
        match ptr.as_ref().unwrap().get_bech32_address(hrp_cstr.as_ref()) {
            Ok(a) => a,
            Err(e) => {
                ffi_helpers::update_last_error(e);
                return null_mut();
            }
        }
    };

    let address_c_str = match CString::new(address) {
        Ok(s) => s,
        Err(e) => {
            ffi_helpers::update_last_error(e);
            return null_mut();
        }
    };

    address_c_str.into_raw()
}

/// Generates a signature of the provided data.
/// The returned [`Signature`] pointer must bee freed using the [`wallet_sign_free`] function
/// to avoid memory leaks.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [`error_message_utf8`] function.
#[no_mangle]
pub extern "C" fn wallet_sign(
    ptr: *const MnemonicWallet,
    data: *const c_uchar,
    data_len: c_uint,
) -> *mut Signature {
    null_pointer_check!(ptr);
    null_pointer_check!(data);

    let signature = unsafe {
        let data = std::slice::from_raw_parts(data, len as usize);
        match ptr.as_ref().unwrap().sign(data).as_mut() {
            Ok(s) => {
                let ptr = s.as_mut_ptr();
                let vec_len = s.len() as c_uint;
                // Prevent deallocation from rust, the array now can be reached only from the ptr variable.
                mem::forget(s);

                Signature {
                    len: vec_len,
                    data: ptr,
                }
            }
            Err(e) => {
                ffi_helpers::update_last_error(e.to_owned());
                return null_mut();
            }
        }
    };

    Box::into_raw(Box::from(signature))
}

/// Deallocate a [`Signature`] instance.
#[no_mangle]
pub extern "C" fn wallet_sign_free(ptr: *mut Signature) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let signature = ptr.as_ref().unwrap();

        drop(Vec::from_raw_parts(
            signature.data,
            signature.len as usize,
            signature.len as usize,
        ));
    };

    Box::from(ptr);
}

// Macro to export the ffi_helpers's functions used to access the error message from other programming languages.
export_error_handling_functions!();
