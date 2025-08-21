use nalgebra::Vector4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{UiError, UiResult};

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
pub struct ThemeColors {
    pub background: Vector4<f32>,
    pub surface: Vector4<f32>,
    pub surface_variant: Vector4<f32>,
    pub primary: Vector4<f32>,
    pub secondary: Vector4<f32>,
    pub text: Vector4<f32>,
    pub text_secondary: Vector4<f32>,
    pub border: Vector4<f32>,
    pub hover: Vector4<f32>,
    pub error: Vector4<f32>,
    pub warning: Vector4<f32>,
    pub success: Vector4<f32>,
    pub info: Vector4<f32>,
    pub editor_background: Vector4<f32>,
    pub line_number: Vector4<f32>,
    pub highlight: Vector4<f32>,
    pub cursor: Vector4<f32>,
    pub keyword: Vector4<f32>,
    pub string: Vector4<f32>,
    pub number: Vector4<f32>,
    pub comment: Vector4<f32>,
    pub function: Vector4<f32>,
    pub type_color: Vector4<f32>,
    pub custom: HashMap<String, Vector4<f32>>,
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: Vector4::new(0.1, 0.1, 0.1, 1.0),
            surface: Vector4::new(0.15, 0.15, 0.15, 1.0),
            surface_variant: Vector4::new(0.2, 0.2, 0.2, 1.0),
            primary: Vector4::new(0.2, 0.6, 1.0, 1.0),
            secondary: Vector4::new(1.0, 0.4, 0.7, 1.0),
            text: Vector4::new(0.95, 0.95, 0.95, 1.0),
            text_secondary: Vector4::new(0.7, 0.7, 0.7, 1.0),
            border: Vector4::new(0.3, 0.3, 0.3, 1.0),
            hover: Vector4::new(0.25, 0.25, 0.25, 1.0),
            error: Vector4::new(1.0, 0.3, 0.3, 1.0),
            warning: Vector4::new(1.0, 0.7, 0.3, 1.0),
            success: Vector4::new(0.3, 1.0, 0.3, 1.0),
            info: Vector4::new(0.3, 0.7, 1.0, 1.0),
            editor_background: Vector4::new(0.08, 0.08, 0.08, 1.0),
            line_number: Vector4::new(0.5, 0.5, 0.5, 1.0),
            highlight: Vector4::new(0.2, 0.3, 0.5, 0.3),
            cursor: Vector4::new(1.0, 1.0, 1.0, 1.0),
            keyword: Vector4::new(0.8, 0.4, 0.9, 1.0),
            string: Vector4::new(0.4, 0.8, 0.4, 1.0),
            number: Vector4::new(0.4, 0.7, 1.0, 1.0),
            comment: Vector4::new(0.5, 0.5, 0.5, 1.0),
            function: Vector4::new(1.0, 0.8, 0.4, 1.0),
            type_color: Vector4::new(0.4, 0.8, 0.8, 1.0),
            custom: HashMap::new(),
        }
    }
    
    pub fn light() -> Self {
        Self {
            background: Vector4::new(0.98, 0.98, 0.98, 1.0),
            surface: Vector4::new(1.0, 1.0, 1.0, 1.0),
            surface_variant: Vector4::new(0.95, 0.95, 0.95, 1.0),
            primary: Vector4::new(0.0, 0.4, 0.8, 1.0),
            secondary: Vector4::new(0.8, 0.2, 0.5, 1.0),
            text: Vector4::new(0.1, 0.1, 0.1, 1.0),
            text_secondary: Vector4::new(0.4, 0.4, 0.4, 1.0),
            border: Vector4::new(0.8, 0.8, 0.8, 1.0),
            hover: Vector4::new(0.9, 0.9, 0.9, 1.0),
            error: Vector4::new(0.8, 0.1, 0.1, 1.0),
            warning: Vector4::new(0.8, 0.5, 0.1, 1.0),
            success: Vector4::new(0.1, 0.7, 0.1, 1.0),
            info: Vector4::new(0.1, 0.5, 0.8, 1.0),
            editor_background: Vector4::new(1.0, 1.0, 1.0, 1.0),
            line_number: Vector4::new(0.6, 0.6, 0.6, 1.0),
            highlight: Vector4::new(0.8, 0.9, 1.0, 0.3),
            cursor: Vector4::new(0.0, 0.0, 0.0, 1.0),
            keyword: Vector4::new(0.6, 0.2, 0.7, 1.0),
            string: Vector4::new(0.2, 0.6, 0.2, 1.0),
            number: Vector4::new(0.2, 0.5, 0.8, 1.0),
            comment: Vector4::new(0.6, 0.6, 0.6, 1.0),
            function: Vector4::new(0.8, 0.6, 0.2, 1.0),
            type_color: Vector4::new(0.2, 0.6, 0.6, 1.0),
            custom: HashMap::new(),
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

pub struct ThemeManager {
    themes: HashMap<ThemeId, Theme>,
    current: ThemeId,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert(ThemeId::Dark, Theme::dark());
        themes.insert(ThemeId::Light, Theme::light());
        
        Self {
            themes,
            current: ThemeId::Dark,
        }
    }
    
    pub fn load_default_themes(&mut self) -> UiResult<()> {
        self.themes.insert(ThemeId::Dark, Theme::dark());
        self.themes.insert(ThemeId::Light, Theme::light());
        Ok(())
    }
    
    pub fn add_theme(&mut self, theme: Theme) -> ThemeId {
        let id = ThemeId::Custom(self.themes.len() as u32);
        self.themes.insert(id, theme);
        id
    }
    
    pub fn get_theme(&self, id: ThemeId) -> UiResult<&Theme> {
        self.themes.get(&id)
            .ok_or_else(|| UiError::ThemeNotFound(id.to_string()))
    }
    
    pub fn has_theme(&self, id: ThemeId) -> bool {
        self.themes.contains_key(&id)
    }
    
    pub fn set_current(&mut self, id: ThemeId) -> UiResult<()> {
        if self.has_theme(id) {
            self.current = id;
            Ok(())
        } else {
            Err(UiError::ThemeNotFound(id.to_string()))
        }
    }
    
    pub fn current(&self) -> ThemeId {
        self.current
    }
}