//! Applies an affine transformation to each stream element.

use data_layer::stream::ScalarStream;

/// Performs a linear scaling and offset using `v ↦ a*v + b`.
pub struct Scale<S> {
    src: S,
    a: f64,
    b: f64,
}

impl<S> Scale<S> {
    pub fn new(src: S, a: f64, b: f64) -> Self { Self { src, a, b } }
}

impl<S> ScalarStream for Scale<S>
where
    S: ScalarStream,
{
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        self.src.next_val().map(|r| r.map(|v| self.a * v + self.b))
    }
}
