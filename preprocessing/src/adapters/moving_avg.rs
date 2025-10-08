//! Computes a simple moving average over a fixed-size window.

use data_layer::stream::ScalarStream;
use std::collections::VecDeque;

/// Produces a smoothed version of the upstream stream by averaging
/// the most recent `win` values.
pub struct MovingAverage<S> {
    src: S,
    win: usize,
    buf: VecDeque<f64>,
    sum: f64,
}

impl<S> MovingAverage<S> {
    pub fn new(src: S, win: usize) -> Self {
        Self { src, win: win.max(1), buf: VecDeque::new(), sum: 0.0 }
    }
}

impl<S> ScalarStream for MovingAverage<S>
where
    S: ScalarStream,
{
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        let Some(res) = self.src.next_val() else { return None; };
        match res {
            Ok(v) => {
                self.buf.push_back(v);
                self.sum += v;
                if self.buf.len() > self.win {
                    if let Some(x) = self.buf.pop_front() { self.sum -= x; }
                }
                let avg = self.sum / (self.buf.len() as f64);
                Some(Ok(avg))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
