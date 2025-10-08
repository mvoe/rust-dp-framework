//! Computes standardized z-scores from an entire stream.

use data_layer::stream::ScalarStream;
use crate::error::PrepError;

/// Collects all values from a stream and returns a `Vec<f64>`
/// containing z-scores `(v − mean) / std`.
pub struct ZScore;

impl ZScore {
    pub fn run<S: ScalarStream>(mut src: S) -> Result<Vec<f64>, PrepError> {
        let mut vals = Vec::new();
        while let Some(res) = src.next_val() {
            vals.push(res.map_err(PrepError::Upstream)?);
        }
        if vals.is_empty() {
            return Err(PrepError::NotEnoughData("empty stream"));
        }

        let n = vals.len() as f64;
        let mean = vals.iter().copied().sum::<f64>() / n;
        let var  = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n.max(1.0);
        let std  = var.sqrt();

        if std == 0.0 {
            return Ok(vals.into_iter().map(|_| 0.0).collect());
        }

        Ok(vals.into_iter().map(|v| (v - mean) / std).collect())
    }
}
