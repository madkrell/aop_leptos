use serde::{Deserialize, Serialize};

/// Spectral reflectance data for a paint color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralData {
    pub color_name: String,
    pub spectral_curve: Vec<f64>,
    pub hex_color: String,
}

/// Result of a paint mixing optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MixingResult {
    pub paints: Vec<String>,
    pub weights: Vec<f64>,
    pub error: f64,
    pub hex_colors: Vec<String>,
}

/// Internal mixture representation during optimization
#[derive(Debug, Clone)]
pub struct PaintMixture {
    pub paints: Vec<String>,
    pub weights: Vec<f64>,
    pub error: f64,
    pub hex_colors: Vec<String>,
}

/// Errors that can occur during color mixing
#[derive(Debug, thiserror::Error)]
pub enum ColorError {
    #[error("Missing required color: {0}")]
    MissingColor(String),
    #[error("No valid mixture found")]
    NoValidMixture,
    #[error("Optimization error: {0}")]
    OptimizationError(String),
}

/// Available mix choice strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MixChoice {
    BlackWhite2Colors,
    BlackWhite3Colors,
    AllAvailableColors,
    NeutralGreys,
    NoBlack,
}

impl MixChoice {
    pub fn as_str(&self) -> &'static str {
        match self {
            MixChoice::BlackWhite2Colors => "black + white + 2 colours",
            MixChoice::BlackWhite3Colors => "black + white + 3 colours",
            MixChoice::AllAvailableColors => "all available colours",
            MixChoice::NeutralGreys => "neutral greys",
            MixChoice::NoBlack => "no black",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "black + white + 2 colours" => Some(MixChoice::BlackWhite2Colors),
            "black + white + 3 colours" => Some(MixChoice::BlackWhite3Colors),
            "all available colours" => Some(MixChoice::AllAvailableColors),
            "neutral greys" => Some(MixChoice::NeutralGreys),
            "no black" => Some(MixChoice::NoBlack),
            _ => None,
        }
    }

    pub fn all() -> Vec<MixChoice> {
        vec![
            MixChoice::BlackWhite2Colors,
            MixChoice::BlackWhite3Colors,
            MixChoice::AllAvailableColors,
            MixChoice::NeutralGreys,
            MixChoice::NoBlack,
        ]
    }
}
