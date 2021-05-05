#[cfg(feature = "ffi")]
#[macro_use]
extern crate ffi_helpers;

pub mod crypto;
mod error;
pub use crate::error::WalletError;

#[cfg(feature = "wasm-bindgen")]
pub mod wasm32_bindgen;

#[cfg(feature = "ffi")]
pub mod ffi;
