use nalgebra::Vector4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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