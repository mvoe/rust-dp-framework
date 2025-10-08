use thiserror::Error;

#[derive(Debug, Error)]
pub enum MechError {
    #[error("upstream stream error: {0}")]
    Upstream(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("invalid parameter: {0}")]
    InvalidParam(&'static str),

    #[error("not enough data: {0}")]
    NotEnoughData(&'static str),
}
