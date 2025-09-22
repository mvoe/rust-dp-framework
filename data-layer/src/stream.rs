// src/stream.rs
use std::error::Error;

/// Trait for a stream of scalar `f64` values, returning each value or an error until the stream ends.
pub trait ScalarStream {
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn Error + Send + Sync>>>;
}
