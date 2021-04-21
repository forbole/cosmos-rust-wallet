//! This file defines the various errors raised by the wallet
use thiserror::Error;

/// Various kinds of errors that can be raised by the wallet
#[derive(Error, Debug, Clone)]
pub enum WalletError {
    #[error("sign error: {0}")]
    Sign(String),

    #[error("Mnemonic error: {0}")]
    Mnemonic(String),

    #[error("Invalid derivation path: {0}")]
    DerivationPath(String),

    #[error("Private key error: {0}")]
    PrivateKey(String),

    #[error("Invalid human readable path {0}")]
    Hrp(String),
}
