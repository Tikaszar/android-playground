use serde::{Deserialize, Serialize};
use super::colors::ThemeColors;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeId {
    Dark,
    Light,
    Custom(u32),
}

impl Default for ThemeId {
    fn default() -> Self {
        ThemeId::Dark
    }
}

impl std::fmt::Display for ThemeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeId::Dark => write!(f, "Dark"),
            ThemeId::Light => write!(f, "Light"),
            ThemeId::Custom(id) => write!(f, "Custom({})", id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ThemeColors::dark(),
            typography: ThemeTypography::default(),
            spacing: ThemeSpacing::default(),
            borders: ThemeBorders::default(),
        }
    }
    
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub font_size_base: f32,
    pub font_size_small: f32,
    pub font_size_large: f32,
    pub font_size_heading: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for ThemeTypography {
    fn default() -> Self {
        Self {
            font_family: "monospace".to_string(),
            font_size_base: 14.0,
            font_size_small: 12.0,
            font_size_large: 16.0,
            font_size_heading: 20.0,
            line_height: 1.5,
            letter_spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub unit: f32,
    pub padding_small: f32,
    pub padding_medium: f32,
    pub padding_large: f32,
    pub margin_small: f32,
    pub margin_medium: f32,
    pub margin_large: f32,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            unit: 8.0,
            padding_small: 4.0,
            padding_medium: 8.0,
            padding_large: 16.0,
            margin_small: 4.0,
            margin_medium: 8.0,
            margin_large: 16.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBorders {
    pub radius_small: f32,
    pub radius_medium: f32,
    pub radius_large: f32,
    pub width_thin: f32,
    pub width_medium: f32,
    pub width_thick: f32,
}

impl Default for ThemeBorders {
    fn default() -> Self {
        Self {
            radius_small: 2.0,
            radius_medium: 4.0,
            radius_large: 8.0,
            width_thin: 1.0,
            width_medium: 2.0,
            width_thick: 4.0,
        }
    }
}