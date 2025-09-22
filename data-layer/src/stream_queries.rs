// src/stream_queries.rs
use std::error::Error;
use crate::stream::ScalarStream;

/// Closed interval [min, max] used for clamping values to a bounded domain.
#[derive(Clone, Copy)]
pub struct BoundedF64 { pub min: f64, pub max: f64 }
impl BoundedF64 {
    pub fn new(min: f64, max: f64) -> Self { assert!(min < max); Self { min, max } }
    #[inline] fn clamp(&self, v: f64) -> f64 { v.min(self.max).max(self.min) }
}

/// COUNT over a streaming source.
/// Note: Clamping is irrelevant here (count does not depend on value magnitude).
pub fn count_stream<S: ScalarStream>(mut s: S) -> Result<usize, Box<dyn Error + Send + Sync>> {
    let mut n = 0usize;
    while let Some(val) = s.next_val() {
        val?;
        n += 1;
    }
    Ok(n)
}

/// SUM over a stream with per-item clamping to ensure bounded influence.
/// Returns (sum, n) so callers can reuse the count.
pub fn sum_stream<S: ScalarStream>(mut s: S, dom: BoundedF64) -> Result<(f64, usize), Box<dyn Error + Send + Sync>> {
    let (mut sum, mut n) = (0.0, 0usize);
    while let Some(val) = s.next_val() {
        let v = dom.clamp(val?);
        sum += v;
        n += 1;
    }
    Ok((sum, n))
}

/// MEAN over a stream with clamping. Deterministic (no noise added here).
/// Returns (mean, n). If n == 0, mean is defined as 0.0 to avoid NaN.
pub fn mean_stream<S: ScalarStream>(mut s: S, dom: BoundedF64) -> Result<(f64, usize), Box<dyn Error + Send + Sync>> {
    let (mut sum, mut n) = (0.0, 0usize);
    while let Some(val) = s.next_val() {
        let v = dom.clamp(val?);
        sum += v;
        n += 1;
    }
    let mean = if n == 0 { 0.0 } else { sum / n as f64 };
    Ok((mean, n))
}

/// HISTOGRAM with `bins` equal-width buckets over [min, max].
/// Values are clamped, then assigned to a bucket. The rightmost boundary is inclusive.
///
/// Returns a vector of triplets: (bin_left, bin_right, count).
pub fn histogram_stream<S: ScalarStream>(
    mut s: S,
    dom: BoundedF64,
    bins: usize
) -> Result<Vec<(f64, f64, usize)>, Box<dyn Error + Send + Sync>> {
    let b = bins.max(1);
    let width = (dom.max - dom.min) / b as f64;
    let mut counts = vec![0usize; b];

    while let Some(val) = s.next_val() {
        let x = dom.clamp(val?);
        let mut k = ((x - dom.min) / width).floor() as isize;
        if k < 0 { k = 0; }
        if k as usize >= b { k = (b as isize) - 1; }
        counts[k as usize] += 1;
    }

    let mut out = Vec::with_capacity(b);
    for i in 0..b {
        let left = dom.min + i as f64 * width;
        let right = if i + 1 == b { dom.max } else { dom.min + (i + 1) as f64 * width };
        out.push((left, right, counts[i]));
    }
    Ok(out)
}


/// L1 sensitivities for the corresponding streaming queries.
/// These are used to calibrate DP mechanisms later on.

pub fn l1_sens_count() -> f64 { 1.0 }
pub fn l1_sens_sum(dom: BoundedF64) -> f64 { dom.max - dom.min }
pub fn l1_sens_mean(dom: BoundedF64, n: usize) -> f64 { if n == 0 { 0.0 } else { (dom.max - dom.min) / n as f64 } }
pub fn l1_sens_hist_count() -> f64 { 1.0 }
