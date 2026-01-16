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
    use crate::services::paint_mixing::get_default_t_matrix;
    use crate::services::lhtss::LHTSS;
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

    // Mix the reflectances
    let lhtss = LHTSS::new(get_default_t_matrix());
    let mixed = lhtss.mix_reflectance(&paint_reflectances, &weights);

    // Convert to RGB hex
    let xyz = lhtss.reflectance_to_xyz(&mixed);
    let lab = lhtss.xyz_to_lab(&xyz);

    // Simple Lab to sRGB conversion (approximate)
    let y = (lab[0] + 16.0) / 116.0;
    let x = lab[1] / 500.0 + y;
    let z = y - lab[2] / 200.0;

    let f_inv = |t: f64| {
        if t > 0.206893 {
            t.powi(3)
        } else {
            (t - 16.0 / 116.0) / 7.787
        }
    };

    let xr = f_inv(x) * 0.95047;
    let yr = f_inv(y);
    let zr = f_inv(z) * 1.08883;

    // XYZ to sRGB
    let r = 3.2406 * xr - 1.5372 * yr - 0.4986 * zr;
    let g = -0.9689 * xr + 1.8758 * yr + 0.0415 * zr;
    let b = 0.0557 * xr - 0.2040 * yr + 1.0570 * zr;

    let gamma = |c: f64| {
        let c = c.max(0.0).min(1.0);
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    };

    let r = (gamma(r) * 255.0).round() as u8;
    let g = (gamma(g) * 255.0).round() as u8;
    let b = (gamma(b) * 255.0).round() as u8;

    Ok(format!("#{:02x}{:02x}{:02x}", r, g, b))
}
