use data_layer::stream::ScalarStream;
use rand::{rngs::StdRng, SeedableRng, Rng};
use rand_distr::{Distribution, Normal};
use crate::error::MechError;

/// Sample Laplace noise (same as in noise.rs)
fn sample_laplace<R: Rng>(rng: &mut R, b: f64) -> f64 {
    let u: f64 = rng.gen::<f64>() - 0.5;
    -b * u.signum() * (1.0 - 2.0 * u.abs()).ln()
}

/// DP sum (Laplace) with L1-sensitivity `Δ1`.
pub struct DpSum;

impl DpSum {
    pub fn laplace<S: ScalarStream>(
        mut src: S,
        l1_sensitivity: f64,
        epsilon: f64,
        seed: Option<u64>,
    ) -> Result<f64, MechError> {
        if epsilon <= 0.0 {
            return Err(MechError::InvalidParam("epsilon must be > 0"));
        }
        let b = l1_sensitivity / epsilon;

        let mut sum = 0.0;
        while let Some(res) = src.next_val() {
            sum += res.map_err(MechError::Upstream)?;
        }

        let mut rng = seed.map(StdRng::seed_from_u64).unwrap_or_else(StdRng::from_entropy);
        let noise = sample_laplace(&mut rng, b);
        Ok(sum + noise)
    }
}

/// DP mean with Gaussian noise (assuming per-record clipping to control L2-sensitivity).
pub struct DpMean;

impl DpMean {
    pub fn gaussian<S: ScalarStream>(
        mut src: S,
        l2_sensitivity_per_record: f64,
        epsilon: f64,
        delta: f64,
        bounded_n: usize,
        seed: Option<u64>,
    ) -> Result<f64, MechError> {
        if epsilon <= 0.0 || !(0.0..1.0).contains(&delta) || bounded_n == 0 {
            return Err(MechError::InvalidParam("invalid epsilon/delta/n"));
        }

        let mut sum = 0.0;
        let mut n = 0usize;
        while let Some(res) = src.next_val() {
            sum += res.map_err(MechError::Upstream)?;
            n += 1;
        }
        if n == 0 {
            return Err(MechError::NotEnoughData("empty stream"));
        }

        let mean = sum / (n as f64);
        let sens_mean = l2_sensitivity_per_record / (bounded_n as f64);
        let term = (1.25 / delta).ln() * 2.0;
        let sigma = sens_mean * term.sqrt() / epsilon;

        let mut rng = seed.map(StdRng::seed_from_u64).unwrap_or_else(StdRng::from_entropy);
        let dist = Normal::new(0.0, sigma).unwrap();
        let noise = dist.sample(&mut rng);
        Ok(mean + noise)
    }
}
