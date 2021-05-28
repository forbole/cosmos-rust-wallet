#[macro_use]
extern crate cfg_if;

mod cypher;
mod io;
pub mod preferences;
pub mod unencrypted;

#[cfg(feature = "wasm-bindgen")]
pub mod wasm;
