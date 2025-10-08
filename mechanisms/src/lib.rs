//! Privacy mechanisms and utilities for numeric streams.

pub mod error;
pub mod calibrate;
pub mod clip;
pub mod noise;
pub mod aggregate;

/// Re-exports commonly used pieces.
pub mod prelude {
    pub use crate::error::MechError;
    pub use crate::calibrate::{laplace_b, gaussian_sigma};
    pub use crate::clip::Clipper;
    pub use crate::noise::{LaplaceNoise, GaussianNoise};
    pub use crate::aggregate::{DpMean, DpSum};
}
