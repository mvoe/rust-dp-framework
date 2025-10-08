/// Laplace scale `b` for L1 sensitivity `Δ1` and epsilon `ε`.
/// b = Δ1 / ε
pub fn laplace_b(l1_sensitivity: f64, epsilon: f64) -> f64 {
    assert!(epsilon > 0.0 && l1_sensitivity >= 0.0);
    l1_sensitivity / epsilon
}

/// (Approximate) Gaussian sigma for L2 sensitivity `Δ2`, epsilon `ε`, and delta `δ`.
/// A common conservative bound:
///   σ = Δ2 * sqrt(2 ln(1.25/δ)) / ε
pub fn gaussian_sigma(l2_sensitivity: f64, epsilon: f64, delta: f64) -> f64 {
    assert!(epsilon > 0.0 && delta > 0.0 && delta < 1.0 && l2_sensitivity >= 0.0);
    let term = (1.25 / delta).ln() * 2.0;
    l2_sensitivity * term.sqrt() / epsilon
}
