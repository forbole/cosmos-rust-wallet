use prost::{DecodeError, EncodeError};
use thiserror::Error;

/// The various error that can be raised from [`super::client::CosmosClient`].
#[derive(Error, Debug, Eq, PartialEq)]
pub enum CosmosError {
    #[error("Encoding error: {0}")]
    Encode(String),

    #[error("Decoding  error: {0}")]
    Decode(String),

    #[error("Sign error: {0}")]
    Sign(String),

    #[error("gRPC error: {0}")]
    Grpc(String),

    #[error("LCD error: {0}")]
    Lcd(String),
}

/// The various error that can be raised from [`super::tx::TxBuilder`].
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TxBuildError {
    #[error("Encoding error: {0}")]
    Encode(String),

    #[error("Missing account information")]
    NoAccountInfo,

    #[error("Missing transaction fee")]
    NoFee,

    #[error("Sign error: {0}")]
    Sign(String),
}

impl From<EncodeError> for CosmosError {
    fn from(e: EncodeError) -> Self {
        CosmosError::Encode(e.to_string())
    }
}

impl From<DecodeError> for CosmosError {
    fn from(e: DecodeError) -> Self {
        CosmosError::Decode(e.to_string())
    }
}
