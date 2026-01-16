//! LHTSS (Least Hue Trace Spectral Source) algorithm for color mixing
//!
//! Converts sRGB colors to spectral reflectance curves for accurate paint mixing.

use nalgebra::{DMatrix, DVector};
use ndarray::{s, Array1, Array2, Order};

/// LHTSS algorithm implementation for spectral reflectance computation
#[derive(Clone)]
pub struct LHTSS {
    t_matrix: Array2<f64>,
}

impl LHTSS {
    /// Create a new LHTSS instance with the given T-matrix (3x36 wavelength-to-XYZ transform)
    pub fn new(t_matrix: Array2<f64>) -> Self {
        assert_eq!(
            t_matrix.shape(),
            [3, 36],
            "T-matrix must be 3x36, got {:?}",
            t_matrix.shape()
        );
        Self { t_matrix }
    }

    /// Compute target reflectance curve from sRGB color
    pub fn compute_reflectance_target(&self, srgb: [u8; 3]) -> Result<Array1<f64>, String> {
        // Special cases
        if srgb.iter().all(|&x| x == 0) {
            return Ok(Array1::from_elem(31, 0.0001)); // Black
        }
        if srgb.iter().all(|&x| x == 255) {
            return Ok(Array1::from_elem(31, 1.0)); // White
        }

        let rgb = self.srgb_to_linear(srgb);

        // Initialize optimization variables
        let mut z = Array1::zeros(36);
        let mut lambda = Array1::zeros(3);
        let d = self.create_difference_matrix();
        let max_iter = 500; // Increased from 100 for better convergence
        let ftol = 1e-6; // Slightly relaxed tolerance

        let mut best_z = z.clone();
        let mut best_error = f64::MAX;

        for _iter in 0..max_iter {
            let d0 = (&z.mapv(|x: f64| x.tanh()) + 1.0) / 2.0;
            let d1 = Array2::from_diag(&z.mapv(|x: f64| (1.0 / x.cosh()).powf(2.0) / 2.0));
            let d2 = Array2::from_diag(&z.mapv(|x: f64| -(1.0 / x.cosh()).powf(2.0) * x.tanh()));

            let f1 = &d.dot(&z) + &d1.dot(&self.t_matrix.t()).dot(&lambda);
            let t_d0 = self.t_matrix.dot(&d0);
            let f2 = &t_d0 - &rgb;

            let mut f = Vec::with_capacity(39);
            f.extend(f1.iter());
            f.extend(f2.iter());
            let f = Array1::from_vec(f);

            // Track best solution so far
            let error: f64 = f.iter().map(|x| x * x).sum();
            if error < best_error {
                best_error = error;
                best_z = z.clone();
            }

            let j = self.create_jacobian(&z, &d, &lambda, &d1, &d2)?;
            let neg_f = f.mapv(|x: f64| -x);
            let delta = self.solve_linear_system(&j, &neg_f)?;

            z = z + Array1::from_vec(delta.slice(s![..36]).to_vec());
            lambda = lambda + Array1::from_vec(delta.slice(s![36..]).to_vec());

            if f.iter().all(|&x| x.abs() < ftol) {
                let full_range = (z.mapv(|x: f64| x.tanh()) + 1.0) / 2.0;
                // Return wavelengths 400nm to 700nm (31 values from index 2 to 32)
                let reduced_range = full_range.slice(s![2..33]).to_owned();
                return Ok(reduced_range);
            }
        }

        // If we didn't converge within tolerance, use the best solution found
        // This handles difficult colors that don't fully converge but get close
        if best_error < 1.0 {
            let full_range = (best_z.mapv(|x: f64| x.tanh()) + 1.0) / 2.0;
            let reduced_range = full_range.slice(s![2..33]).to_owned();
            return Ok(reduced_range);
        }

        Err(format!(
            "LHTSS did not converge for RGB({},{},{}), best error: {:.6}",
            srgb[0], srgb[1], srgb[2], best_error
        ))
    }

    /// Mix multiple reflectance curves using weighted geometric mean
    pub fn mix_reflectance(&self, reflectance_data: &[Array1<f64>], weights: &[f64]) -> Array1<f64> {
        let n = reflectance_data[0].len();
        let mut mixed = Array1::zeros(n);
        let sum_weights: f64 = weights.iter().sum();

        for i in 0..n {
            let mut product = 1.0;
            for (j, &weight) in weights.iter().enumerate() {
                product *= reflectance_data[j][i].powf(weight);
            }
            mixed[i] = product.powf(1.0 / sum_weights);
        }
        mixed
    }

    /// Convert sRGB to linear RGB
    pub fn srgb_to_linear(&self, srgb: [u8; 3]) -> Array1<f64> {
        Array1::from_vec(
            srgb.iter()
                .map(|&x| {
                    let x = x as f64 / 255.0;
                    if x <= 0.04045 {
                        x / 12.92
                    } else {
                        ((x + 0.055) / 1.055).powf(2.4)
                    }
                })
                .collect(),
        )
    }

    /// Convert reflectance to XYZ color space
    pub fn reflectance_to_xyz(&self, reflectance: &Array1<f64>) -> [f64; 3] {
        // Pad reflectance to 36 values if needed
        let r = if reflectance.len() == 31 {
            let mut full = Array1::zeros(36);
            full.slice_mut(s![2..33]).assign(reflectance);
            full[0] = reflectance[0];
            full[1] = reflectance[0];
            full[33] = reflectance[30];
            full[34] = reflectance[30];
            full[35] = reflectance[30];
            full
        } else {
            reflectance.clone()
        };

        let xyz = self.t_matrix.dot(&r);
        [xyz[0], xyz[1], xyz[2]]
    }

    /// Convert XYZ to Lab color space
    pub fn xyz_to_lab(&self, xyz: &[f64; 3]) -> [f64; 3] {
        let xn = 95.047;
        let yn = 100.0;
        let zn = 108.883;

        let f = |t: f64| {
            if t > 0.008856 {
                t.powf(1.0 / 3.0)
            } else {
                7.787 * t + 16.0 / 116.0
            }
        };

        let fx = f(xyz[0] / xn);
        let fy = f(xyz[1] / yn);
        let fz = f(xyz[2] / zn);

        [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
    }

    /// Calculate Delta E (color difference) between two Lab colors
    pub fn delta_e(&self, lab1: &[f64; 3], lab2: &[f64; 3]) -> f64 {
        let dl = lab2[0] - lab1[0];
        let da = lab2[1] - lab1[1];
        let db = lab2[2] - lab1[2];
        (dl * dl + da * da + db * db).sqrt()
    }

    fn create_difference_matrix(&self) -> Array2<f64> {
        let mut d = Array2::zeros((36, 36));
        for i in 0..36 {
            d[[i, i]] = 4.0;
            if i > 0 {
                d[[i, i - 1]] = -2.0;
            }
            if i < 35 {
                d[[i, i + 1]] = -2.0;
            }
        }
        d[[0, 0]] = 2.0;
        d[[35, 35]] = 2.0;
        d
    }

    fn create_jacobian(
        &self,
        z: &Array1<f64>,
        d: &Array2<f64>,
        lambda: &Array1<f64>,
        d1: &Array2<f64>,
        d2: &Array2<f64>,
    ) -> Result<Array2<f64>, String> {
        let n = z.len();
        let mut j = Array2::zeros((n + 3, n + 3));

        let temp = d2.dot(&self.t_matrix.t());
        let lambda_reshaped = lambda.clone().into_shape_with_order(((3, 1), Order::RowMajor)).map_err(|e| e.to_string())?;
        let d2_t_lambda = temp
            .dot(&lambda_reshaped)
            .into_shape_with_order((36, Order::RowMajor))
            .map_err(|e| e.to_string())?;
        let top_left = d + &Array2::from_diag(&d2_t_lambda);
        j.slice_mut(s![..n, ..n]).assign(&top_left);
        j.slice_mut(s![..n, n..]).assign(&d1.dot(&self.t_matrix.t()));
        j.slice_mut(s![n.., ..n]).assign(&self.t_matrix.dot(d1));

        Ok(j)
    }

    fn solve_linear_system(&self, jacobian: &Array2<f64>, f: &Array1<f64>) -> Result<Array1<f64>, String> {
        let n = jacobian.nrows();
        let m = jacobian.ncols();
        let j_mat = DMatrix::from_iterator(n, m, jacobian.iter().cloned());
        let f_vec = DVector::from_iterator(n, f.iter().cloned());

        if let Some(solution) = j_mat.clone().lu().solve(&f_vec) {
            return Ok(Array1::from_vec(solution.data.into()));
        }

        // Fallback to SVD
        let svd = j_mat.svd(true, true);
        let u = svd.u.ok_or("SVD failed to compute U")?;
        let s = svd.singular_values;
        let vt = svd.v_t.ok_or("SVD failed to compute V^T")?;

        let s_inv = DVector::from_iterator(
            s.len(),
            s.iter().map(|&x| if x.abs() > 1e-10 { 1.0 / x } else { 0.0 }),
        );

        let solution = vt.transpose() * (s_inv.component_mul(&(u.transpose() * f_vec)));
        Ok(Array1::from_vec(solution.data.into()))
    }
}
