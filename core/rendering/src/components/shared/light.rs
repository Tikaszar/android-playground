//! Light component

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: ColorRGB,
    pub intensity: Float,
    pub cast_shadows: bool,
    pub shadow_strength: Float,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LightType {
    Ambient,
    Directional {
        direction: Vec3,
    },
    Point {
        position: Vec3,
        range: Float,
        attenuation: Vec3,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        range: Float,
        inner_angle: Float,
        outer_angle: Float,
        attenuation: Vec3,
    },
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Ambient,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            cast_shadows: false,
            shadow_strength: 1.0,
        }
    }
}

impl_component_data!(Light);