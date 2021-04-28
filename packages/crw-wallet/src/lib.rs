pub mod crypto;
mod error;

#[cfg(feature = "wasm-bindgen")]
pub mod wasm32_bindgen;

pub use crate::error::WalletError;
