//! Error types used throughout the preprocessing layer.

use thiserror::Error;

/// High-level errors for preprocessing operations.
#[derive(Debug, Error)]
pub enum PrepError {
    /// Error bubbled up from an upstream [`ScalarStream`].
    #[error("upstream stream error: {0}")]
    Upstream(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Operation required more data than available.
    #[error("not enough data: {0}")]
    NotEnoughData(&'static str),

    /// Invalid argument such as zero window size or negative bound.
    #[error("invalid parameter: {0}")]
    InvalidParam(&'static str),
}
