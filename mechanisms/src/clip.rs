use data_layer::stream::ScalarStream;

/// Clamps each value into [lo, hi].
pub struct Clipper<S> {
    src: S,
    lo: f64,
    hi: f64,
}

impl<S> Clipper<S> {
    pub fn new(src: S, lo: f64, hi: f64) -> Self { Self { src, lo, hi } }
}

impl<S: ScalarStream> ScalarStream for Clipper<S> {
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        self.src.next_val().map(|r| r.map(|v| v.clamp(self.lo, self.hi)))
    }
}
