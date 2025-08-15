use crate::resources::TextureFormat;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererCapabilities {
    pub max_texture_size: u32,
    pub max_texture_units: u32,
    pub max_vertex_attributes: u32,
    pub max_uniform_buffer_size: usize,
    pub max_storage_buffer_size: usize,
    pub max_color_attachments: u32,
    pub max_compute_work_group_size: [u32; 3],
    pub max_compute_work_group_count: [u32; 3],
    pub supported_texture_formats: Vec<TextureFormat>,
    pub features: RendererFeatures,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RendererFeatures {
    pub compute_shaders: bool,
    pub geometry_shaders: bool,
    pub tessellation_shaders: bool,
    pub multi_draw_indirect: bool,
    pub bindless_textures: bool,
    pub ray_tracing: bool,
    pub mesh_shaders: bool,
    pub variable_rate_shading: bool,
}

impl Default for RendererFeatures {
    fn default() -> Self {
        Self {
            compute_shaders: false,
            geometry_shaders: false,
            tessellation_shaders: false,
            multi_draw_indirect: false,
            bindless_textures: false,
            ray_tracing: false,
            mesh_shaders: false,
            variable_rate_shading: false,
        }
    }
}