use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillMode {
    Solid,
    Wireframe,
    Points,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub fill_mode: FillMode,
    pub depth_bias: f32,
    pub depth_bias_slope_scale: f32,
    pub depth_bias_clamp: f32,
    pub scissor_test_enabled: bool,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            fill_mode: FillMode::Solid,
            depth_bias: 0.0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            scissor_test_enabled: false,
        }
    }
}