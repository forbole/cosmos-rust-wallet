//! Crate that provides a set of utility to store preferences to the device storage.  
//!
//! The values that can be saved into the preference are:
//! * i32
//! * bool
//! * str
//! * Vec<u8>

#[macro_use]
extern crate cfg_if;

pub mod encrypted;
mod io;
pub mod preferences;
pub mod unencrypted;

#[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "js",))]
pub mod wasm;
