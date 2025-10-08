use data_layer::stream::ScalarStream;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use std::error::Error;

/// Sample a Laplace(0, b) random value using the inverse CDF method.
fn sample_laplace<R: Rng>(rng: &mut R, b: f64) -> f64 {
    let u: f64 = rng.gen::<f64>() - 0.5;
    -b * u.signum() * (1.0 - 2.0 * u.abs()).ln()
}

/// Adds Laplace(0, b) noise to each value.
pub struct LaplaceNoise<S> {
    src: S,
    b: f64,
    rng: StdRng,
}

impl<S> LaplaceNoise<S> {
    pub fn new(src: S, b: f64, seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
        Self { src, b, rng }
    }
}

impl<S: ScalarStream> ScalarStream for LaplaceNoise<S> {
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn Error + Send + Sync>>> {
        let Some(r) = self.src.next_val() else { return None; };
        match r {
            Ok(v) => {
                let noise = sample_laplace(&mut self.rng, self.b);
                Some(Ok(v + noise))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// Adds Gaussian(0, σ) noise to each value.
pub struct GaussianNoise<S> {
    src: S,
    sigma: f64,
    rng: StdRng,
}

impl<S> GaussianNoise<S> {
    pub fn new(src: S, sigma: f64, seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
        Self { src, sigma, rng }
    }
}

impl<S: ScalarStream> ScalarStream for GaussianNoise<S> {
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn Error + Send + Sync>>> {
        let Some(r) = self.src.next_val() else { return None; };
        match r {
            Ok(v) => {
                let dist = Normal::new(0.0, self.sigma).unwrap();
                let noise = dist.sample(&mut self.rng);
                Some(Ok(v + noise))
            }
            Err(e) => Some(Err(e)),
        }
    }
}
