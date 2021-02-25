//! This file defines the various errors raised by the cosmos signer
use thiserror::Error;

/// Various kinds of errors that can be raised by the signer
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("GRPC error: {0}")]
    Grpc(String),

    #[error("encoding error: {0]")]
    Encoding(String),

    #[error("decoding error: {0}")]
    Decode(String),

    #[error("Legacy API error: {0}")]
    Lcd(String),

    #[error("Mnemonic error: {0}")]
    Mnemonic(String),

    #[error("Private key error: {0}")]
    PrivateKey(String),

    #[error("Bech 32 encoding error: {0}")]
    Bech32(String)
}