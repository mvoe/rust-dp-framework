//! Filters a stream by a boolean predicate.

use data_layer::stream::ScalarStream;

/// Passes through only values satisfying a predicate function.
pub struct Filter<S, P> {
    src: S,
    pred: P,
}

impl<S, P> Filter<S, P> {
    pub fn new(src: S, pred: P) -> Self { Self { src, pred } }
}

impl<S, P> ScalarStream for Filter<S, P>
where
    S: ScalarStream,
    P: Fn(f64) -> bool + Send + Sync + 'static,
{
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        loop {
            let Some(res) = self.src.next_val() else { return None; };
            match res {
                Ok(v) if (self.pred)(v) => return Some(Ok(v)),
                Ok(_) => continue,
                Err(e) => return Some(Err(e)),
            }
        }
    }
}
