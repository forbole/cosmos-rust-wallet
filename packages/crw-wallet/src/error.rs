//! This file defines the various errors raised by the wallet.
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum WalletError {
    #[error("sign error: {0}")]
    Sign(String),

    #[error("mnemonic error: {0}")]
    Mnemonic(String),

    #[error("invalid derivation path: {0}")]
    DerivationPath(String),

    #[error("private key error: {0}")]
    PrivateKey(String),

    #[error("invalid human readable path {0}")]
    Hrp(String),
}
