use crate::prelude::*;
use crate::error::*;
use data_layer::stream::ScalarStream;

/// A simple stream used for testing: emits a fixed sequence of f64 values.
struct FromVec {
    it: std::vec::IntoIter<f64>,
}
impl FromVec {
    fn new(v: Vec<f64>) -> Self { Self { it: v.into_iter() } }
}
impl ScalarStream for FromVec {
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn std::error::Error + Send + Sync>>> {
        self.it.next().map(Ok)
    }
}

#[test]
fn filter_skips_negatives() {
    let src = FromVec::new(vec![-1.0, 0.0, 2.0]);
    let mut f = Filter::new(src, |v| v > 0.0);
    let mut out = vec![];
    while let Some(x) = f.next_val() { out.push(x.unwrap()); }
    assert_eq!(out, vec![2.0]);
}

#[test]
fn clip_limits_values() {
    let src = FromVec::new(vec![-10.0, 5.0, 15.0]);
    let mut c = Clip::new(src, 0.0, 10.0);
    let out: Vec<_> = std::iter::from_fn(|| c.next_val()).map(|r| r.unwrap()).collect();
    assert_eq!(out, vec![0.0, 5.0, 10.0]);
}

#[test]
fn scale_applies_affine_transform() {
    let src = FromVec::new(vec![1.0, 2.0, 3.0]);
    let mut s = Scale::new(src, 2.0, 1.0);
    let out: Vec<_> = std::iter::from_fn(|| s.next_val()).map(|r| r.unwrap()).collect();
    assert_eq!(out, vec![3.0, 5.0, 7.0]);
}

#[test]
fn moving_average_smooths_values() {
    let src = FromVec::new(vec![1.0, 2.0, 3.0, 4.0]);
    let mut ma = MovingAverage::new(src, 2);
    let out: Vec<_> = std::iter::from_fn(|| ma.next_val()).map(|r| r.unwrap()).collect();
    // expected running average with window=2 → [1, 1.5, 2.5, 3.5]
    let expect = vec![1.0, 1.5, 2.5, 3.5];
    for (a, b) in out.iter().zip(expect.iter()) {
        assert!((a - b).abs() < 1e-9, "expected {b}, got {a}");
    }
}

#[test]
fn zscore_standardizes_values() {
    let src = FromVec::new(vec![1.0, 2.0, 3.0]);
    let zs = ZScore::run(src).unwrap();
    let mean = zs.iter().sum::<f64>() / zs.len() as f64;
    let var = zs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / zs.len() as f64;
    assert!((mean.abs()) < 1e-9);
    assert!((var - 1.0).abs() < 1e-9);
}
