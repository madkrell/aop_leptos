//! Paint weight optimization for color mixing using Kubelka-Munk theory
//!
//! Uses the K/S (absorption/scattering) ratio for physically accurate subtractive mixing.

use ndarray::Array1;

use crate::models::ColorError;

/// Convert reflectance R to Kubelka-Munk K/S ratio
/// Formula: K/S = (1 - R)² / (2R)
#[inline]
fn reflectance_to_ks(r: f64) -> f64 {
    // Clamp reflectance to avoid division by zero and negative values
    let r = r.max(0.001).min(0.999);
    (1.0 - r).powi(2) / (2.0 * r)
}

/// Convert Kubelka-Munk K/S ratio back to reflectance R
/// Formula: R = 1 + K/S - √(K/S² + 2·K/S)
#[inline]
fn ks_to_reflectance(ks: f64) -> f64 {
    // Handle edge cases
    if ks <= 0.0 {
        return 1.0; // Pure white (no absorption)
    }
    let r = 1.0 + ks - (ks * ks + 2.0 * ks).sqrt();
    r.max(0.0).min(1.0)
}

/// Mix reflectance curves using Kubelka-Munk theory
/// This is the physically correct way to mix subtractive colors (paints)
pub fn kubelka_munk_mix(reflectance_data: &[Array1<f64>], weights: &[f64]) -> Array1<f64> {
    let n = reflectance_data[0].len();
    let mut mixed = Array1::zeros(n);
    let sum_weights: f64 = weights.iter().sum();

    if sum_weights <= 0.0 {
        return mixed;
    }

    // Normalize weights
    let normalized_weights: Vec<f64> = weights.iter().map(|w| w / sum_weights).collect();

    for i in 0..n {
        // Convert each paint's reflectance at this wavelength to K/S
        // Then mix the K/S values (weighted average - pigments are additive in K/S space)
        let mut ks_sum = 0.0;
        for (j, &weight) in normalized_weights.iter().enumerate() {
            let r = reflectance_data[j][i];
            let ks = reflectance_to_ks(r);
            ks_sum += ks * weight;
        }

        // Convert mixed K/S back to reflectance
        mixed[i] = ks_to_reflectance(ks_sum);
    }
    mixed
}

/// Optimize paint weights to minimize error between mixed reflectance and target
/// Uses Kubelka-Munk theory for physically accurate paint mixing
pub fn optimize_weights(
    selected_paints: &[(String, Array1<f64>, String)],
    initial_weights: &[f64],
    target_reflectance: &Array1<f64>,
) -> Result<Vec<f64>, ColorError> {
    let n = initial_weights.len();
    let mut weights = initial_weights.to_vec();

    let max_iterations = 1000;
    let tolerance = 1e-8;
    let mut alpha = 0.5; // Start with smaller step size for K-M optimization

    let mut best_weights = weights.clone();
    let mut best_error = f64::MAX;

    // Extract reflectance arrays
    let reflectances: Vec<&Array1<f64>> = selected_paints
        .iter()
        .map(|(_, r, _)| r)
        .collect();

    for iteration in 0..max_iterations {
        // Normalize weights
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            for w in weights.iter_mut() {
                *w /= sum;
            }
        }

        // Calculate mixed reflectance using Kubelka-Munk
        let reflectance_data: Vec<Array1<f64>> = reflectances.iter().map(|r| (*r).clone()).collect();
        let mixed = kubelka_munk_mix(&reflectance_data, &weights);

        // Calculate error (mean squared error in reflectance space)
        let diff: Array1<f64> = target_reflectance - &mixed;
        let current_error = diff.mapv(|x| x * x).mean().unwrap_or(f64::MAX);

        // Track best solution
        if current_error < best_error {
            best_error = current_error;
            best_weights = weights.clone();
        }

        if current_error < tolerance {
            break;
        }

        // Adaptive learning rate - slow down as we get closer
        if iteration > 0 && iteration % 100 == 0 {
            alpha *= 0.9;
        }

        // Calculate gradients using finite differences
        let mut gradients = Vec::with_capacity(n);
        let delta = 0.001;

        for i in 0..n {
            let mut test_weights = weights.clone();
            test_weights[i] += delta;

            let sum: f64 = test_weights.iter().sum();
            for w in test_weights.iter_mut() {
                *w /= sum;
            }

            let test_mixed = kubelka_munk_mix(&reflectance_data, &test_weights);
            let test_diff: Array1<f64> = target_reflectance - &test_mixed;
            let test_error = test_diff.mapv(|x| x * x).mean().unwrap_or(f64::MAX);

            gradients.push((test_error - current_error) / delta);
        }

        // Update weights using gradient descent
        for i in 0..n {
            weights[i] -= alpha * gradients[i];
            weights[i] = weights[i].max(0.0).min(1.0);
        }
    }

    // Use best weights found
    let sum: f64 = best_weights.iter().sum();
    if sum > 0.0 {
        for w in best_weights.iter_mut() {
            *w /= sum;
        }
    }

    Ok(best_weights)
}

/// Compute mean squared error between mixed and target reflectance
pub fn compute_error(mixed_reflectance: &Array1<f64>, target_reflectance: &Array1<f64>) -> f64 {
    let diff = target_reflectance - mixed_reflectance;
    diff.mapv(|x| x * x).mean().unwrap_or(f64::MAX)
}

/// Compute weighted geometric mean of reflectance curves (alternative mixing method)
pub fn weighted_geometric_mean(reflectance_data: &[Array1<f64>], weights: &[f64]) -> Array1<f64> {
    let n = reflectance_data[0].len();
    let mut mixed = Array1::zeros(n);
    let sum_weights: f64 = weights.iter().sum();

    for i in 0..n {
        let mut product = 1.0;
        for (j, &weight) in weights.iter().enumerate() {
            // Clamp to avoid log(0) issues
            let r = reflectance_data[j][i].max(0.001);
            product *= r.powf(weight);
        }
        mixed[i] = product.powf(1.0 / sum_weights);
    }
    mixed
}
