//! Clamps numeric values to a fixed interval.

use data_layer::stream::ScalarStream;

/// Restricts each upstream value `v` to the range `[lo, hi]`.
pub struct Clip<S> {
    src: S,
    lo: f64,
    hi: f64,
}

impl<S> Clip<S> {
    pub fn new(src: S, lo: f64, hi: f64) -> Self { Self { src, lo, hi } }
}

impl<S> ScalarStream for Clip<S>
where
    S: ScalarStream,
{
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        self.src.next_val().map(|r| r.map(|v| v.clamp(self.lo, self.hi)))
    }
}
