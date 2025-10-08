//! Streaming adapters that implement [`ScalarStream`](data_layer::stream::ScalarStream).
//!
//! Each adapter wraps an existing stream and transforms or filters the values
//! on the fly. All adapters are composable.

pub mod map;
pub mod filter;
pub mod clip;
pub mod scale;
pub mod zscore;
pub mod moving_avg;
