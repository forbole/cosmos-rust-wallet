//! This file defines the various errors raised by the signer

use anomaly::{BoxError, Context};
use thiserror::Error;

/// The error raised by the signer
pub type Error = anomaly::Error<Kind>;

/// Various kinds of errors that can be raised by the signer
#[derive(Error, Debug, Clone)]
pub enum Kind {
    #[error("GRPC error: {0}")]
    Grpc(String),

    #[error("decoding error: {0}")]
    Decode(String)
}