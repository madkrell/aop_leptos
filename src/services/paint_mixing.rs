//! Paint mixing service for finding optimal paint combinations
//!
//! Uses Kubelka-Munk theory for physically accurate subtractive color mixing.

use ndarray::{Array1, Array2};
use rayon::prelude::*;

use crate::models::{ColorError, MixingResult};
use crate::services::lhtss::LHTSS;
use crate::services::optimization::{kubelka_munk_mix, optimize_weights};

/// Paint mixing service that finds optimal paint combinations for a target color
pub struct PaintMixingService {
    t_matrix: Array2<f64>,
}

impl PaintMixingService {
    /// Create a new paint mixing service with the T-matrix for color conversion
    pub fn new(t_matrix: Array2<f64>) -> Self {
        Self { t_matrix }
    }

    /// Calculate target reflectance from RGB color using LHTSS algorithm
    pub fn calculate_target_reflectance(&self, rgb: [u8; 3]) -> Result<Array1<f64>, String> {
        let lhtss = LHTSS::new(self.t_matrix.clone());
        lhtss.compute_reflectance_target(rgb)
    }

    /// Find optimal paint combinations for a target color
    pub fn find_combinations(
        &self,
        target_reflectance: &Array1<f64>,
        paint_data: &[(String, Array1<f64>, String)],
        mix_choice: &str,
    ) -> Result<Vec<MixingResult>, ColorError> {
        let results = match mix_choice.to_lowercase().as_str() {
            "black + white + 2 colours" => {
                self.find_black_white_n_colors(target_reflectance, paint_data, 2)?
            }
            "black + white + 3 colours" => {
                self.find_black_white_n_colors(target_reflectance, paint_data, 3)?
            }
            "all available colours" => {
                self.find_all_available_colors(target_reflectance, paint_data)?
            }
            "neutral greys" => self.find_neutral_greys(target_reflectance, paint_data)?,
            "no black" => self.find_no_black(target_reflectance, paint_data)?,
            _ => return Err(ColorError::OptimizationError("Invalid mix choice".into())),
        };

        // Sort by error and take top 5
        let mut sorted = results;
        sorted.sort_by(|a, b| {
            a.error
                .partial_cmp(&b.error)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(sorted.into_iter().take(5).collect())
    }

    /// Find combinations using white + black + N other colors
    fn find_black_white_n_colors(
        &self,
        target: &Array1<f64>,
        paint_data: &[(String, Array1<f64>, String)],
        n_extra: usize,
    ) -> Result<Vec<MixingResult>, ColorError> {
        // Find white and black
        let white = paint_data
            .iter()
            .find(|(name, _, _)| name.to_lowercase().trim() == "titanium white")
            .ok_or_else(|| ColorError::MissingColor("Titanium White".into()))?
            .clone();
        let black = paint_data
            .iter()
            .find(|(name, _, _)| name.to_lowercase().trim() == "ivory black")
            .ok_or_else(|| ColorError::MissingColor("Ivory Black".into()))?
            .clone();

        // Get other colors (excluding white, black, and warm white)
        let other_paints: Vec<_> = paint_data
            .iter()
            .filter(|(name, _, _)| {
                let name_lower = name.to_lowercase();
                name_lower.trim() != "titanium white"
                    && name_lower.trim() != "ivory black"
                    && name_lower.trim() != "warm white"
            })
            .cloned()
            .collect();

        // Generate combinations based on n_extra
        let combinations: Vec<Vec<(String, Array1<f64>, String)>> = if n_extra == 2 {
            // 2 extra colors
            let mut combos = Vec::new();
            for (i, paint2) in other_paints.iter().enumerate() {
                for paint3 in other_paints.iter().skip(i + 1) {
                    combos.push(vec![
                        white.clone(),
                        black.clone(),
                        paint2.clone(),
                        paint3.clone(),
                    ]);
                }
            }
            combos
        } else if n_extra == 3 {
            // 3 extra colors
            let mut combos = Vec::new();
            for (i, paint2) in other_paints.iter().enumerate() {
                for (j, paint3) in other_paints.iter().enumerate().skip(i + 1) {
                    for paint4 in other_paints.iter().skip(j + 1) {
                        combos.push(vec![
                            white.clone(),
                            black.clone(),
                            paint2.clone(),
                            paint3.clone(),
                            paint4.clone(),
                        ]);
                    }
                }
            }
            combos
        } else {
            Vec::new()
        };

        // Process in parallel
        let target_clone = target.clone();
        let results: Vec<MixingResult> = combinations
            .par_iter()
            .filter_map(|paints| {
                let n = paints.len();
                let initial_weights = vec![1.0 / n as f64; n];
                optimize_weights(paints, &initial_weights, &target_clone)
                    .ok()
                    .map(|weights| self.create_result(paints, weights, &target_clone))
            })
            .collect();

        Ok(results)
    }

    /// Find combinations using all available colors
    fn find_all_available_colors(
        &self,
        target: &Array1<f64>,
        paint_data: &[(String, Array1<f64>, String)],
    ) -> Result<Vec<MixingResult>, ColorError> {
        let mut all_combinations: Vec<Vec<(String, Array1<f64>, String)>> = Vec::new();

        // Try 3, 4, and 5 paint combinations
        for n_paints in 3..=5 {
            for i in 0..paint_data.len().saturating_sub(n_paints - 1) {
                let combo: Vec<_> = paint_data[i..i + n_paints].to_vec();
                all_combinations.push(combo);
            }
        }

        // Process in parallel
        let target_clone = target.clone();
        let results: Vec<MixingResult> = all_combinations
            .par_iter()
            .filter_map(|combo| {
                let initial_weights = vec![1.0 / combo.len() as f64; combo.len()];
                optimize_weights(combo, &initial_weights, &target_clone)
                    .ok()
                    .map(|weights| self.create_result(combo, weights, &target_clone))
            })
            .collect();

        Ok(results)
    }

    /// Find combinations using neutral greys
    fn find_neutral_greys(
        &self,
        target: &Array1<f64>,
        paint_data: &[(String, Array1<f64>, String)],
    ) -> Result<Vec<MixingResult>, ColorError> {
        let grey_paints: Vec<_> = paint_data
            .iter()
            .filter(|(name, _, _)| {
                name.to_lowercase().contains("grey") || name.to_lowercase().contains("gray")
            })
            .cloned()
            .collect();

        if grey_paints.is_empty() {
            return Ok(Vec::new());
        }

        let other_paints: Vec<_> = paint_data
            .iter()
            .filter(|(name, _, _)| {
                let name_lower = name.to_lowercase();
                !name_lower.contains("grey")
                    && !name_lower.contains("gray")
                    && name_lower.trim() != "titanium white"
                    && name_lower.trim() != "ivory black"
                    && name_lower.trim() != "warm white"
            })
            .cloned()
            .collect();

        // Generate combinations
        let mut combinations: Vec<Vec<(String, Array1<f64>, String)>> = Vec::new();
        for grey in &grey_paints {
            for (i, paint2) in other_paints.iter().enumerate() {
                for paint3 in other_paints.iter().skip(i + 1) {
                    combinations.push(vec![grey.clone(), paint2.clone(), paint3.clone()]);
                }
            }
        }

        // Process in parallel
        let target_clone = target.clone();
        let results: Vec<MixingResult> = combinations
            .par_iter()
            .filter_map(|combo| {
                let initial_weights = vec![1.0 / combo.len() as f64; combo.len()];
                optimize_weights(combo, &initial_weights, &target_clone)
                    .ok()
                    .map(|weights| self.create_result(combo, weights, &target_clone))
            })
            .collect();

        Ok(results)
    }

    /// Find combinations without black
    fn find_no_black(
        &self,
        target: &Array1<f64>,
        paint_data: &[(String, Array1<f64>, String)],
    ) -> Result<Vec<MixingResult>, ColorError> {
        let available: Vec<_> = paint_data
            .iter()
            .filter(|(name, _, _)| !name.to_lowercase().contains("black"))
            .cloned()
            .collect();

        let mut all_combinations: Vec<Vec<(String, Array1<f64>, String)>> = Vec::new();

        for n_paints in 3..=4 {
            for i in 0..available.len().saturating_sub(n_paints - 1) {
                let combo: Vec<_> = available[i..i + n_paints].to_vec();
                all_combinations.push(combo);
            }
        }

        // Process in parallel
        let target_clone = target.clone();
        let results: Vec<MixingResult> = all_combinations
            .par_iter()
            .filter_map(|combo| {
                let initial_weights = vec![1.0 / combo.len() as f64; combo.len()];
                optimize_weights(combo, &initial_weights, &target_clone)
                    .ok()
                    .map(|weights| self.create_result(combo, weights, &target_clone))
            })
            .collect();

        Ok(results)
    }

    fn create_result(
        &self,
        paints: &[(String, Array1<f64>, String)],
        weights: Vec<f64>,
        target: &Array1<f64>,
    ) -> MixingResult {
        // Calculate mixed reflectance using Kubelka-Munk
        let reflectance_data: Vec<Array1<f64>> = paints.iter().map(|(_, r, _)| r.clone()).collect();
        let mixed = kubelka_munk_mix(&reflectance_data, &weights);

        // Calculate Delta E error using LHTSS color space conversion
        let lhtss = LHTSS::new(self.t_matrix.clone());
        let mixed_xyz = lhtss.reflectance_to_xyz(&mixed);
        let target_xyz = lhtss.reflectance_to_xyz(target);
        let mixed_lab = lhtss.xyz_to_lab(&mixed_xyz);
        let target_lab = lhtss.xyz_to_lab(&target_xyz);
        let error = lhtss.delta_e(&mixed_lab, &target_lab);

        MixingResult {
            paints: paints.iter().map(|(name, _, _)| name.clone()).collect(),
            weights,
            error,
            hex_colors: paints.iter().map(|(_, _, hex)| hex.clone()).collect(),
        }
    }
}

/// Get default T-matrix for D65 illuminant, 10-degree observer
pub fn get_default_t_matrix() -> Array2<f64> {
    // Standard CIE 1964 10-degree observer color matching functions
    // scaled for D65 illuminant, 36 wavelengths from 380nm to 730nm (10nm steps)
    let x_bar = [
        0.000160, 0.002362, 0.019110, 0.084736, 0.204492, 0.314679, 0.383734, 0.370702, 0.302273,
        0.195618, 0.080507, 0.016172, 0.003816, 0.037465, 0.117749, 0.236491, 0.376772, 0.529826,
        0.705224, 0.878655, 1.014160, 1.118520, 1.123990, 1.030480, 0.856297, 0.647467, 0.431567,
        0.268329, 0.152568, 0.081261, 0.040851, 0.019941, 0.009577, 0.004539, 0.002175, 0.001060,
    ];
    let y_bar = [
        0.000017, 0.000253, 0.002004, 0.008756, 0.021391, 0.038676, 0.062077, 0.089456, 0.128201,
        0.185190, 0.253589, 0.339133, 0.460777, 0.606741, 0.761757, 0.875211, 0.961988, 0.991761,
        0.997340, 0.955552, 0.868934, 0.777405, 0.658341, 0.527963, 0.398057, 0.283493, 0.179828,
        0.107633, 0.060281, 0.031800, 0.015905, 0.007749, 0.003718, 0.001762, 0.000846, 0.000415,
    ];
    let z_bar = [
        0.000705, 0.010482, 0.086011, 0.389366, 0.972542, 1.553480, 1.967280, 1.994800, 1.745370,
        1.317560, 0.772125, 0.415254, 0.218502, 0.112044, 0.060709, 0.030451, 0.013676, 0.003988,
        0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000,
        0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000, 0.000000,
    ];

    let mut t_matrix = Array2::zeros((3, 36));
    for i in 0..36 {
        t_matrix[[0, i]] = x_bar[i];
        t_matrix[[1, i]] = y_bar[i];
        t_matrix[[2, i]] = z_bar[i];
    }
    t_matrix
}
