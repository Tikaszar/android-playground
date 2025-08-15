use crate::resources::TextureHandle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTargetAttachment {
    pub texture: TextureHandle,
    pub mip_level: u32,
    pub array_layer: u32,
    pub clear_value: ClearValue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClearValue {
    Color([f32; 4]),
    DepthStencil { depth: f32, stencil: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderTargetDesc {
    pub color_attachments: Vec<RenderTargetAttachment>,
    pub depth_attachment: Option<RenderTargetAttachment>,
    pub width: u32,
    pub height: u32,
}