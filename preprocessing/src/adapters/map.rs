//! Applies a user-defined function to each value in a stream.

use data_layer::stream::ScalarStream;

/// Transforms each upstream value using a closure `f(v)`.
pub struct Map<S, F> {
    src: S,
    f: F,
}

impl<S, F> Map<S, F> {
    pub fn new(src: S, f: F) -> Self { Self { src, f } }
}

impl<S, F> ScalarStream for Map<S, F>
where
    S: ScalarStream,
    F: Fn(f64) -> f64 + Send + Sync + 'static,
{
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        self.src.next_val().map(|r| r.map(|v| (self.f)(v)))
    }
}
