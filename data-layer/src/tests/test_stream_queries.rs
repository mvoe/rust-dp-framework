// data-layer/src/tests/test_stream_queries.rs
use approx::assert_relative_eq;

use crate::stream::ScalarStream;
use crate::stream_queries::{
    BoundedF64, count_stream, sum_stream, mean_stream, histogram_stream,
    l1_sens_count, l1_sens_sum, l1_sens_mean, l1_sens_hist_count,
};

/// Minimal in-memory stream for tests.
struct VecScalarStream {
    data: Vec<Result<f64, Box<dyn std::error::Error + Send + Sync>>>,
    idx: usize,
}
impl VecScalarStream {
    fn ok(vals: &[f64]) -> Self {
        Self {
            data: vals.iter().copied().map(|v| Ok(v) as _).collect(),
            idx: 0,
        }
    }
    fn with_error_at(vals: &[f64], err_at: usize) -> Self {
        let mut data: Vec<Result<f64, Box<dyn std::error::Error + Send + Sync>>> =
            vals.iter().copied().map(|v| Ok(v) as _).collect();
        if err_at < data.len() {
            data[err_at] = Err("parse error".into());
        }
        Self { data, idx: 0 }
    }
}
impl ScalarStream for VecScalarStream {
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        if self.idx >= self.data.len() { return None; }
        let out = Some(self.data[self.idx].as_ref().map(|v| *v).map_err(|e| e.to_string().into()));
        self.idx += 1;
        out
    }
}

#[test]
fn count_stream_basic() {
    let s = VecScalarStream::ok(&[1.0, 2.0, 3.0, 4.0]);
    let n = count_stream(s).unwrap();
    assert_eq!(n, 4);
}

#[test]
fn sum_and_mean_with_clamping() {
    let dom = BoundedF64::new(-10.0, 10.0);
    let s = VecScalarStream::ok(&[-100.0, -2.0, 0.0, 3.5, 200.0]);
    let (sum, n) = sum_stream(s, dom).unwrap();
    // Clamped sequence: [-10, -2, 0, 3.5, 10] => sum = 1.5
    assert_eq!(n, 5);
    assert_relative_eq!(sum, 1.5, epsilon = 1e-12);

    // mean_stream should match sum / n
    let s2 = VecScalarStream::ok(&[-100.0, -2.0, 0.0, 3.5, 200.0]);
    let (mean, n2) = mean_stream(s2, dom).unwrap();
    assert_eq!(n2, 5);
    assert_relative_eq!(mean, 1.5 / 5.0, epsilon = 1e-12);
}

#[test]
fn mean_empty_returns_zero() {
    let dom = BoundedF64::new(0.0, 1.0);
    let s = VecScalarStream::ok(&[]);
    let (mean, n) = mean_stream(s, dom).unwrap();
    assert_eq!(n, 0);
    assert_relative_eq!(mean, 0.0, epsilon = 1e-12);
}

#[test]
fn histogram_basic_bins_and_edges() {
    let dom = BoundedF64::new(0.0, 10.0);
    // Values include boundaries and out-of-range to test clamping and rightmost-inclusive rule
    let s = VecScalarStream::ok(&[-5.0, 0.0, 1.0, 4.9, 5.0, 9.999, 10.0, 12.0]);
    let bins = histogram_stream(s, dom, 2).unwrap();
    // Two bins: [0,5), [5,10] (we set last bin right edge inclusive to 10)
    assert_eq!(bins.len(), 2);
    let (l0, r0, c0) = bins[0];
    let (l1, r1, c1) = bins[1];
    assert_relative_eq!(l0, 0.0);
    assert_relative_eq!(r0, 5.0);
    assert_relative_eq!(l1, 5.0);
    assert_relative_eq!(r1, 10.0);

    // Expected counts:
    // After clamping: 0,0,1,4.9 in bin0; 5.0,9.999,10.0,10.0(in from 12) in bin1 => 4 & 4
    assert_eq!(c0, 4);
    assert_eq!(c1, 4);
}

#[test]
fn sensitivities_match_formulas() {
    let dom = BoundedF64::new(-10.0, 10.0);
    assert_relative_eq!(l1_sens_count(), 1.0);
    assert_relative_eq!(l1_sens_sum(dom), 20.0);

    let n = 5;
    assert_relative_eq!(l1_sens_mean(dom, n), 20.0 / n as f64, epsilon = 1e-12);
    assert_relative_eq!(l1_sens_hist_count(), 1.0);
}

#[test]
fn error_propagates() {
    let dom = BoundedF64::new(0.0, 10.0);
    let s = VecScalarStream::with_error_at(&[1.0, 2.0, 3.0], 1);
    let err = mean_stream(s, dom).unwrap_err();
    assert!(err.to_string().contains("parse error"));
}
