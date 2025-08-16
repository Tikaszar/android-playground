//! Theme definitions

use nalgebra::Vector4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// UI theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ThemeColors::dark(),
            typography: ThemeTypography::default(),
            spacing: ThemeSpacing::default(),
            borders: ThemeBorders::default(),
        }
    }
    
    /// Create light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ThemeColors::light(),
            typography: ThemeTypography::default(),
            spacing: ThemeSpacing::default(),
            borders: ThemeBorders::default(),
        }
    }
}

/// Theme colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: Vector4<f32>,
    pub surface: Vector4<f32>,
    pub primary: Vector4<f32>,
    pub secondary: Vector4<f32>,
    pub text: Vector4<f32>,
    pub text_secondary: Vector4<f32>,
    pub border: Vector4<f32>,
    pub error: Vector4<f32>,
    pub warning: Vector4<f32>,
    pub success: Vector4<f32>,
    pub info: Vector4<f32>,
    pub custom: HashMap<String, Vector4<f32>>,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: Vector4::new(0.1, 0.1, 0.1, 1.0),
            surface: Vector4::new(0.15, 0.15, 0.15, 1.0),
            primary: Vector4::new(0.2, 0.6, 1.0, 1.0),
            secondary: Vector4::new(1.0, 0.4, 0.7, 1.0),
            text: Vector4::new(0.95, 0.95, 0.95, 1.0),
            text_secondary: Vector4::new(0.7, 0.7, 0.7, 1.0),
            border: Vector4::new(0.3, 0.3, 0.3, 1.0),
            error: Vector4::new(1.0, 0.3, 0.3, 1.0),
            warning: Vector4::new(1.0, 0.7, 0.3, 1.0),
            success: Vector4::new(0.3, 1.0, 0.3, 1.0),
            info: Vector4::new(0.3, 0.7, 1.0, 1.0),
            custom: HashMap::new(),
        }
    }
    
    pub fn light() -> Self {
        Self {
            background: Vector4::new(0.98, 0.98, 0.98, 1.0),
            surface: Vector4::new(1.0, 1.0, 1.0, 1.0),
            primary: Vector4::new(0.1, 0.5, 0.9, 1.0),
            secondary: Vector4::new(0.9, 0.3, 0.6, 1.0),
            text: Vector4::new(0.1, 0.1, 0.1, 1.0),
            text_secondary: Vector4::new(0.4, 0.4, 0.4, 1.0),
            border: Vector4::new(0.8, 0.8, 0.8, 1.0),
            error: Vector4::new(0.9, 0.2, 0.2, 1.0),
            warning: Vector4::new(0.9, 0.6, 0.2, 1.0),
            success: Vector4::new(0.2, 0.9, 0.2, 1.0),
            info: Vector4::new(0.2, 0.6, 0.9, 1.0),
            custom: HashMap::new(),
        }
    }
}

/// Theme typography
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub font_family_mono: String,
    pub size_small: f32,
    pub size_normal: f32,
    pub size_large: f32,
    pub size_heading: f32,
    pub line_height: f32,
}

impl Default for ThemeTypography {
    fn default() -> Self {
        Self {
            font_family: "system-ui".to_string(),
            font_family_mono: "monospace".to_string(),
            size_small: 12.0,
            size_normal: 14.0,
            size_large: 18.0,
            size_heading: 24.0,
            line_height: 1.5,
        }
    }
}

/// Theme spacing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
        }
    }
}

/// Theme borders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBorders {
    pub width_thin: f32,
    pub width_normal: f32,
    pub width_thick: f32,
    pub radius_small: f32,
    pub radius_normal: f32,
    pub radius_large: f32,
    pub radius_round: f32,
}

impl Default for ThemeBorders {
    fn default() -> Self {
        Self {
            width_thin: 1.0,
            width_normal: 2.0,
            width_thick: 4.0,
            radius_small: 2.0,
            radius_normal: 4.0,
            radius_large: 8.0,
            radius_round: 999.0,
        }
    }
}