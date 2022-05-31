//! Provides the FFI to interact with [`MnemonicWallet`] from other programming languages.
use crate::crypto::MnemonicWallet;
use crate::WalletError;
use bip39::{Language, Mnemonic, MnemonicType};
use libc::{c_char, c_int, c_uchar, c_uint, size_t};
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::{mem, slice};

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
/// The returned mnemonic must be freed using the [`cstring_free`] function to avoid memory leaks.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [error_message_utf8](ffi_helpers::error_handling::error_message_utf8) function.
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
/// The returned [`MnemonicWallet`] ptr must be freed using the [`wallet_free`] function to avoid memory
/// leaks.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [error_message_utf8](ffi_helpers::error_handling::error_message_utf8) function.
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
/// global variable that can be accessed using the [error_message_utf8](ffi_helpers::error_handling::error_message_utf8) function.
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

/// Gets the wallet public key.
/// This function returns the number of bytes copied into `out_buffer`.
///
/// # Errors
/// Returns -1 if the provided arguments are invalid or -2 if the public key don't fit into `out_buffer`.
#[no_mangle]
pub extern "C" fn wallet_get_public_key(
    ptr: *const MnemonicWallet,
    compressed: c_uint,
    out_buffer: *mut c_uchar,
    size: size_t,
) -> c_int {
    if ptr.is_null() || out_buffer.is_null() || size <= 0 {
        return -1;
    }

    let pub_key = unsafe {
        let key = ptr.as_ref().unwrap().get_pub_key().inner;

        if compressed != 0 {
            key.serialize().to_vec()
        } else {
            key.serialize_uncompressed().to_vec()
        }
    };

    if pub_key.len() > size {
        return -2;
    }

    let out_buf = unsafe { slice::from_raw_parts_mut(out_buffer, pub_key.len()) };

    out_buf.copy_from_slice(&pub_key);
    pub_key.len() as i32
}

/// Generates a signature of the provided data.
/// The returned [`Signature`] pointer must bee freed using the [`wallet_sign_free`] function
/// to avoid memory leaks.
///
/// # Errors
/// This function returns a nullptr in case of error and store the error cause in a local thread
/// global variable that can be accessed using the [error_message_utf8](ffi_helpers::error_handling::error_message_utf8) function.
#[no_mangle]
pub extern "C" fn wallet_sign(
    ptr: *const MnemonicWallet,
    data: *const c_uchar,
    data_len: c_uint,
) -> *mut Signature {
    null_pointer_check!(ptr);
    null_pointer_check!(data);

    let signature = unsafe {
        let data = std::slice::from_raw_parts(data, data_len as usize);
        match ptr.as_ref().unwrap().sign(data) {
            Ok(s) => {
                let mut sign = s.to_owned();
                let ptr = sign.as_mut_ptr();
                let vec_len = s.len() as c_uint;
                // Prevent deallocation from rust, the array now can be reached only from the ptr variable.
                mem::forget(sign);

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

#[cfg(test)]
mod tests {
    use crate::ffi::{
        cstring_free, wallet_free, wallet_from_mnemonic, wallet_get_bech32_address,
        wallet_get_public_key, wallet_random_mnemonic, wallet_sign, wallet_sign_free,
    };
    use ffi_helpers::error_handling::error_message;
    use std::ffi::CString;
    use std::mem;
    use std::ptr::null_mut;

    static COSMOS_DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
    static TEST_MNEMONIC: &str = "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel";

    #[test]
    fn test_random_mnemonic() {
        let mnemonic = wallet_random_mnemonic();
        assert!(!mnemonic.is_null());

        let c_str = unsafe { CString::from_raw(mnemonic) };
        let string = c_str.to_string_lossy().to_string();
        let phrases: Vec<&str> = string.split(" ").collect();
        assert_eq!(24, phrases.len());
    }

    #[test]
    fn initialization_with_valid_mnemonic_and_derivation_path() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        assert!(!wallet.is_null());
        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn initialize_with_invalid_mnemonic() {
        let c_mnemonic = CString::new("invalid mnemonic").unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        assert!(wallet.is_null());

        let error_msg = error_message();
        assert!(error_msg.is_some());

        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn initialize_with_null_mnemonic() {
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(null_mut(), c_dp);

        assert!(wallet.is_null());
        let error_msg = error_message();
        assert!(error_msg.is_some());

        cstring_free(c_dp);
    }

    #[test]
    fn initialize_with_invalid_derivation_path() {
        let c_mnemonic = CString::new("invalid mnemonic").unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();

        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        assert!(wallet.is_null());

        let error_msg = error_message();
        assert!(error_msg.is_some());

        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn initialize_with_null_derivation_path() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, null_mut());

        assert!(wallet.is_null());
        let error_msg = error_message();
        assert!(error_msg.is_some());

        cstring_free(c_mnemonic);
    }

    #[test]
    fn bech32_address() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let hrp = CString::new("cosmos").unwrap().into_raw();
        let address = wallet_get_bech32_address(wallet, hrp);
        let c_address = unsafe { CString::from_raw(address) };

        assert!(!address.is_null());
        assert_eq!(
            c_address.to_string_lossy().as_ref(),
            "cosmos1dzczdka6wpzwvmawpps7tf8047gkft0e5cupun"
        );

        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn bech32_address_with_null_hrp() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let address = wallet_get_bech32_address(wallet, null_mut());
        let error_msg = error_message();
        assert!(address.is_null());
        assert!(error_msg.is_some());

        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn get_public_key() {
        let ref_public_key = "048b3f1f48e4dbc68287473da1a76d81bd827aac22622b7da5f351e2580d14b2823fe447037648f5d83b11dd2ea88e06db6c452b5376aa4c70e7a8c9c7b13cf39a";
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let mut out_buffer: [u8; 65] = [0; 65];
        let copied_bytes =
            wallet_get_public_key(wallet, 0, out_buffer.as_mut_ptr(), out_buffer.len());
        assert_eq!(65, copied_bytes);

        let pub_key_hex = hex::encode(out_buffer);
        assert_eq!(ref_public_key, pub_key_hex);

        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn get_public_key_compressed() {
        let ref_public_key = "028b3f1f48e4dbc68287473da1a76d81bd827aac22622b7da5f351e2580d14b282";
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let mut out_buffer: [u8; 33] = [0; 33];
        let copied_bytes =
            wallet_get_public_key(wallet, 1, out_buffer.as_mut_ptr(), out_buffer.len());
        assert_eq!(33, copied_bytes);

        let pub_key_hex = hex::encode(out_buffer);
        assert_eq!(ref_public_key, pub_key_hex);

        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn get_public_key_invalid_args() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);
        let mut out_buffer: [u8; 64] = [0; 64];

        let copied_bytes = wallet_get_public_key(null_mut(), 0, null_mut(), 0);
        assert_eq!(-1, copied_bytes);

        let copied_bytes = wallet_get_public_key(wallet, 0, null_mut(), 0);
        assert_eq!(-1, copied_bytes);

        let copied_bytes = wallet_get_public_key(null_mut(), 0, out_buffer.as_mut_ptr(), 0);
        assert_eq!(-1, copied_bytes);

        let copied_bytes = wallet_get_public_key(wallet, 0, out_buffer.as_mut_ptr(), 0);
        assert_eq!(-1, copied_bytes);

        let copied_bytes = wallet_get_public_key(wallet, 0, out_buffer.as_mut_ptr(), 2);
        assert_eq!(-2, copied_bytes);

        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn empty_sign() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let empty: Vec<u8> = Vec::new();
        let signature = wallet_sign(wallet, empty.as_ptr(), empty.len() as u32);

        assert!(!signature.is_null());
        assert_eq!(0, unsafe { signature.as_ref() }.unwrap().len);

        wallet_sign_free(signature);
        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn sign_null() {
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let signature = wallet_sign(wallet, null_mut(), 12);
        let error = error_message();

        assert!(signature.is_null());
        assert!(error.is_some());

        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }

    #[test]
    fn sign() {
        let ref_hex_signature = "5590171f32520497dd9ca07a3f03ef69ceff972471821902ebe31532d7f13be51021b7c8849431340fe6e91321987a90ffe5598d5e87fe4d55acf1bb90a000e9";
        let c_mnemonic = CString::new(TEST_MNEMONIC).unwrap().into_raw();
        let c_dp = CString::new(COSMOS_DERIVATION_PATH).unwrap().into_raw();
        let wallet = wallet_from_mnemonic(c_mnemonic, c_dp);

        let data = "some simple data".as_bytes();
        let signature = wallet_sign(wallet, data.as_ptr(), data.len() as u32);

        assert!(!signature.is_null());
        let sign_ref = unsafe { signature.as_ref().unwrap() };
        let signature_vec = unsafe {
            Vec::from_raw_parts(sign_ref.data, sign_ref.len as usize, sign_ref.len as usize)
        };
        let sign_hex = hex::encode(&signature_vec);
        assert_eq!(ref_hex_signature, sign_hex);

        // Forget since will be freed from wallet_sign_free
        mem::forget(signature_vec);
        wallet_sign_free(signature);
        wallet_free(wallet);
        cstring_free(c_mnemonic);
        cstring_free(c_dp);
    }
}
