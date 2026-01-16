use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::MixingResult;

/// Paint brand info for the frontend
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaintBrand {
    pub id: String,
    pub name: String,
}

/// Paint color info for the frontend
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaintColorInfo {
    pub id: String,
    pub hex: String,
}

/// User paint settings
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct UserPaintSettings {
    pub mix_choice: String,
    pub brand: String,
    pub colors: Vec<String>,
}

/// Get available paint brands
#[server]
pub async fn get_paint_brands() -> Result<Vec<PaintBrand>, ServerFnError> {
    use crate::db;
    use axum::Extension;
    use leptos_axum::extract;
    use crate::state::AppState;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let brands = db::get_paint_brands(&state.db).await;

    Ok(brands
        .into_iter()
        .map(|id| {
            let name = id
                .replace('_', " ")
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            PaintBrand { id, name }
        })
        .collect())
}

/// Get paint colors for a brand
#[server]
pub async fn get_paint_colors(brand: String) -> Result<Vec<PaintColorInfo>, ServerFnError> {
    use crate::db;
    use axum::Extension;
    use leptos_axum::extract;
    use crate::state::AppState;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let colors = db::get_paint_colors(&state.db, &brand).await;

    Ok(colors
        .into_iter()
        .map(|c| PaintColorInfo {
            id: c._id,
            hex: c.d65_10deg_hex.unwrap_or_else(|| "#808080".to_string()),
        })
        .collect())
}

/// Get user's paint settings
#[server]
pub async fn get_user_paint_settings() -> Result<UserPaintSettings, ServerFnError> {
    use crate::db;
    use crate::server_fns::get_current_user;

    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    use axum::Extension;
    use leptos_axum::extract;
    use crate::state::AppState;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let settings = db::get_user_settings(&state.db, &user.id).await;

    match settings {
        Some(s) => {
            let selected: serde_json::Value = s
                .selected_colors
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(serde_json::json!({}));

            // Extract brand and colors from the JSON structure
            let (brand, colors) = if let Some(obj) = selected.as_object() {
                if let Some((brand_name, colors_val)) = obj.iter().next() {
                    let colors = colors_val
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    (brand_name.clone(), colors)
                } else {
                    (String::new(), vec![])
                }
            } else {
                (String::new(), vec![])
            };

            Ok(UserPaintSettings {
                mix_choice: s.colour_mix_choice.unwrap_or_default(),
                brand,
                colors,
            })
        }
        None => Ok(UserPaintSettings::default()),
    }
}

/// Save user's paint settings
#[server]
pub async fn save_user_paint_settings(
    mix_choice: String,
    brand: String,
    colors: Vec<String>,
) -> Result<(), ServerFnError> {
    use crate::db;
    use crate::server_fns::get_current_user;

    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    use axum::Extension;
    use leptos_axum::extract;
    use crate::state::AppState;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Store as JSON: { "brand_name": ["color1", "color2", ...] }
    let selected_colors = serde_json::json!({ brand: colors }).to_string();

    db::upsert_user_settings(&state.db, &user.id, &user.email, &mix_choice, &selected_colors)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Find optimal paint combinations for a target color
#[server]
pub async fn find_paint_mix(
    r: u8,
    g: u8,
    b: u8,
) -> Result<Vec<MixingResult>, ServerFnError> {
    use crate::db;
    use crate::server_fns::get_current_user;
    use crate::services::paint_mixing::{get_default_t_matrix, PaintMixingService};
    use ndarray::Array1;

    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    use axum::Extension;
    use leptos_axum::extract;
    use crate::state::AppState;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Get user settings
    let settings = db::get_user_settings(&state.db, &user.id)
        .await
        .ok_or_else(|| ServerFnError::new("Please configure your paint settings first"))?;

    let mix_choice = settings
        .colour_mix_choice
        .unwrap_or_else(|| "black + white + 2 colours".to_string());

    let selected: serde_json::Value = settings
        .selected_colors
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .ok_or_else(|| ServerFnError::new("Please select your paints first"))?;

    // Extract brand and colors
    let (brand, color_names): (String, Vec<String>) = selected
        .as_object()
        .and_then(|obj| obj.iter().next())
        .map(|(brand, colors)| {
            let names = colors
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            (brand.clone(), names)
        })
        .ok_or_else(|| ServerFnError::new("Invalid paint selection"))?;

    if color_names.is_empty() {
        return Err(ServerFnError::new("Please select at least some paints"));
    }

    // Get paint data
    let all_colors = db::get_paint_colors(&state.db, &brand).await;

    // Filter to selected colors and convert spectral data
    let paint_data: Vec<(String, Array1<f64>, String)> = all_colors
        .into_iter()
        .filter(|c| color_names.contains(&c._id))
        .filter_map(|c| {
            let spectral = c.spectral_curve?;
            // Decode spectral curve from bincode (Vec<u8> -> Vec<f64>)
            let curve: Vec<f64> = bincode::deserialize(&spectral).ok()?;
            let hex = c.d65_10deg_hex.unwrap_or_else(|| "#808080".to_string());
            Some((c._id, Array1::from_vec(curve), hex))
        })
        .collect();

    if paint_data.len() < 3 {
        return Err(ServerFnError::new(
            "Not enough paint data. Please select more colors.",
        ));
    }

    // Create mixing service and find combinations
    let service = PaintMixingService::new(get_default_t_matrix());

    let target = service
        .calculate_target_reflectance([r, g, b])
        .map_err(|e| ServerFnError::new(format!("Failed to compute target reflectance: {}", e)))?;

    // Verify paint data dimensions match target
    for (name, curve, _) in &paint_data {
        if curve.len() != target.len() {
            return Err(ServerFnError::new(format!(
                "Paint '{}' has {} spectral values, expected {}",
                name,
                curve.len(),
                target.len()
            )));
        }
    }

    let results = service
        .find_combinations(&target, &paint_data, &mix_choice)
        .map_err(|e| ServerFnError::new(format!("Failed to find combinations: {}", e)))?;

    Ok(results)
}

/// Test a custom paint mixture
#[server]
pub async fn test_paint_mix(
    paints: Vec<String>,
    weights: Vec<f64>,
) -> Result<String, ServerFnError> {
    use crate::db;
    use crate::server_fns::get_current_user;
    use crate::services::optimization::kubelka_munk_mix;
    use ndarray::Array1;

    let user = get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;

    use axum::Extension;
    use leptos_axum::extract;
    use crate::state::AppState;

    let Extension(state) = extract::<Extension<AppState>>()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Get user settings for brand
    let settings = db::get_user_settings(&state.db, &user.id)
        .await
        .ok_or_else(|| ServerFnError::new("Please configure your paint settings first"))?;

    let selected: serde_json::Value = settings
        .selected_colors
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .ok_or_else(|| ServerFnError::new("Please select your paints first"))?;

    let brand = selected
        .as_object()
        .and_then(|obj| obj.keys().next())
        .ok_or_else(|| ServerFnError::new("Invalid paint selection"))?;

    // Get paint data
    let all_colors = db::get_paint_colors(&state.db, brand).await;

    // For a single paint, return its database hex value directly
    if paints.len() == 1 {
        let paint_name = &paints[0];
        if let Some(color) = all_colors.iter().find(|c| &c._id == paint_name) {
            return Ok(color.d65_10deg_hex.clone().unwrap_or_else(|| "#808080".to_string()));
        }
    }

    // Get spectral data for requested paints
    let paint_reflectances: Vec<Array1<f64>> = paints
        .iter()
        .filter_map(|paint_name| {
            all_colors.iter().find(|c| &c._id == paint_name).and_then(|c| {
                let spectral = c.spectral_curve.as_ref()?;
                let curve: Vec<f64> = bincode::deserialize(spectral).ok()?;
                Some(Array1::from_vec(curve))
            })
        })
        .collect();

    if paint_reflectances.len() != paints.len() {
        return Err(ServerFnError::new("Could not find all paint data"));
    }

    // Mix the reflectances using Kubelka-Munk theory
    let mixed = kubelka_munk_mix(&paint_reflectances, &weights);

    // Convert mixed reflectance to XYZ using CIE 1931 2-degree observer and D65 illuminant
    // Wavelengths: 400nm to 700nm in 10nm steps (31 values)
    // These are the standard color matching functions scaled by D65 illuminant
    let cmf_x: [f64; 31] = [
        0.0143, 0.0435, 0.1344, 0.2839, 0.3483, 0.3362, 0.2908, 0.1954, 0.0956,
        0.0320, 0.0049, 0.0093, 0.0633, 0.1655, 0.2904, 0.4334, 0.5945, 0.7621,
        0.9163, 1.0263, 1.0622, 1.0026, 0.8544, 0.6424, 0.4479, 0.2835, 0.1649,
        0.0874, 0.0468, 0.0227, 0.0114,
    ];
    let cmf_y: [f64; 31] = [
        0.0004, 0.0012, 0.0040, 0.0116, 0.0230, 0.0380, 0.0600, 0.0910, 0.1390,
        0.2080, 0.3230, 0.5030, 0.7100, 0.8620, 0.9540, 0.9950, 0.9950, 0.9520,
        0.8700, 0.7570, 0.6310, 0.5030, 0.3810, 0.2650, 0.1750, 0.1070, 0.0610,
        0.0320, 0.0170, 0.0082, 0.0041,
    ];
    let cmf_z: [f64; 31] = [
        0.0679, 0.2074, 0.6456, 1.3856, 1.7471, 1.7721, 1.6692, 1.2876, 0.8130,
        0.4652, 0.2720, 0.1582, 0.0782, 0.0422, 0.0203, 0.0087, 0.0039, 0.0021,
        0.0017, 0.0011, 0.0008, 0.0003, 0.0002, 0.0000, 0.0000, 0.0000, 0.0000,
        0.0000, 0.0000, 0.0000, 0.0000,
    ];

    // Compute XYZ by integrating reflectance * CMF
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    for i in 0..31 {
        x += mixed[i] * cmf_x[i];
        y += mixed[i] * cmf_y[i];
        z += mixed[i] * cmf_z[i];
    }

    // Normalize to D65 white point (sum of Y should equal 1 for perfect white)
    let y_sum: f64 = cmf_y.iter().sum();
    x /= y_sum;
    y /= y_sum;
    z /= y_sum;

    // XYZ to linear sRGB (D65 reference)
    let r_lin = 3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    let g_lin = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    let b_lin = 0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    // Apply sRGB gamma correction
    let gamma = |c: f64| {
        let c = c.max(0.0).min(1.0);
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    };

    let r = (gamma(r_lin) * 255.0).round() as u8;
    let g = (gamma(g_lin) * 255.0).round() as u8;
    let b = (gamma(b_lin) * 255.0).round() as u8;

    Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
}
