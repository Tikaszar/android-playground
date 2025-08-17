//! ECS Components for rendering system internal state management
//!
//! This module defines components used by the rendering system to track
//! GPU resources, render graph state, and frame metrics using core/ecs.

use async_trait::async_trait;
use bytes::{Bytes, BytesMut, BufMut};
use playground_ecs::{Component, ComponentId, EcsResult, EcsError};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::resources::{TextureHandle, ShaderHandle, PipelineHandle, RenderTargetHandle};
use crate::resources::buffer::{VertexBuffer, IndexBuffer, UniformBuffer, StorageBuffer};
use crate::graph::pass::PassId;
use crate::capabilities::RendererFeatures;

/// Component tracking texture resource state
#[derive(Debug, Clone)]
pub struct TextureResourceComponent {
    pub handle: TextureHandle,
    pub width: u32,
    pub height: u32,
    pub format: u32, // WebGL format or Vulkan format
    pub mip_levels: u32,
    pub current_lod: u32,
    pub memory_usage: usize,
    pub last_used_frame: u64,
    pub debug_name: Option<String>,
}

#[async_trait]
impl Component for TextureResourceComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize handle
        buf.put_u64_le(self.handle.id());
        buf.put_u32_le(self.handle.generation());
        
        // Serialize dimensions and format
        buf.put_u32_le(self.width);
        buf.put_u32_le(self.height);
        buf.put_u32_le(self.format);
        buf.put_u32_le(self.mip_levels);
        buf.put_u32_le(self.current_lod);
        
        // Serialize memory and frame info
        buf.put_u64_le(self.memory_usage as u64);
        buf.put_u64_le(self.last_used_frame);
        
        // Serialize debug name
        if let Some(ref name) = self.debug_name {
            buf.put_u8(1);
            buf.put_u32_le(name.len() as u32);
            buf.put_slice(name.as_bytes());
        } else {
            buf.put_u8(0);
        }
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        // Implement when needed for hot-reload
        Err(EcsError::SerializationError("TextureResourceComponent deserialization not implemented".into()))
    }
}

/// Component tracking buffer resource state
#[derive(Debug, Clone)]
pub struct BufferResourceComponent {
    pub buffer_type: BufferType,
    pub size: usize,
    pub usage_flags: u32,
    pub memory_usage: usize,
    pub last_updated_frame: u64,
    pub debug_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum BufferType {
    Vertex(VertexBuffer),
    Index(IndexBuffer),
    Uniform(UniformBuffer),
    Storage(StorageBuffer),
}

#[async_trait]
impl Component for BufferResourceComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize buffer type (simplified - just the discriminant)
        let type_id = match self.buffer_type {
            BufferType::Vertex(_) => 0u8,
            BufferType::Index(_) => 1u8,
            BufferType::Uniform(_) => 2u8,
            BufferType::Storage(_) => 3u8,
        };
        buf.put_u8(type_id);
        
        // Serialize size and usage
        buf.put_u64_le(self.size as u64);
        buf.put_u32_le(self.usage_flags);
        buf.put_u64_le(self.memory_usage as u64);
        buf.put_u64_le(self.last_updated_frame);
        
        // Serialize debug name
        if let Some(ref name) = self.debug_name {
            buf.put_u8(1);
            buf.put_u32_le(name.len() as u32);
            buf.put_slice(name.as_bytes());
        } else {
            buf.put_u8(0);
        }
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("BufferResourceComponent deserialization not implemented".into()))
    }
}

/// Component tracking shader resource state
#[derive(Debug, Clone)]
pub struct ShaderResourceComponent {
    pub handle: ShaderHandle,
    pub path: PathBuf,
    pub stage: u32, // ShaderStage as u32
    pub compiled: bool,
    pub last_modified: Option<std::time::SystemTime>,
    pub watch_enabled: bool,
    pub compilation_errors: Vec<String>,
    pub debug_name: Option<String>,
}

#[async_trait]
impl Component for ShaderResourceComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize handle
        buf.put_u64_le(self.handle.id());
        buf.put_u32_le(self.handle.generation());
        
        // Serialize path
        let path_str = self.path.to_string_lossy();
        buf.put_u32_le(path_str.len() as u32);
        buf.put_slice(path_str.as_bytes());
        
        // Serialize stage and flags
        buf.put_u32_le(self.stage);
        buf.put_u8(if self.compiled { 1 } else { 0 });
        buf.put_u8(if self.watch_enabled { 1 } else { 0 });
        
        // Serialize error count (not the errors themselves for now)
        buf.put_u32_le(self.compilation_errors.len() as u32);
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("ShaderResourceComponent deserialization not implemented".into()))
    }
}

/// Component tracking pipeline state
#[derive(Debug, Clone)]
pub struct PipelineResourceComponent {
    pub handle: PipelineHandle,
    pub vertex_shader: ShaderHandle,
    pub fragment_shader: ShaderHandle,
    pub blend_state: u32,      // Encoded blend state
    pub depth_state: u32,      // Encoded depth state
    pub rasterizer_state: u32, // Encoded rasterizer state
    pub active: bool,
    pub debug_name: Option<String>,
}

#[async_trait]
impl Component for PipelineResourceComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize handles
        buf.put_u64_le(self.handle.id());
        buf.put_u32_le(self.handle.generation());
        buf.put_u64_le(self.vertex_shader.id());
        buf.put_u32_le(self.vertex_shader.generation());
        buf.put_u64_le(self.fragment_shader.id());
        buf.put_u32_le(self.fragment_shader.generation());
        
        // Serialize state
        buf.put_u32_le(self.blend_state);
        buf.put_u32_le(self.depth_state);
        buf.put_u32_le(self.rasterizer_state);
        buf.put_u8(if self.active { 1 } else { 0 });
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("PipelineResourceComponent deserialization not implemented".into()))
    }
}

/// Component tracking render target state
#[derive(Debug, Clone)]
pub struct RenderTargetComponent {
    pub handle: RenderTargetHandle,
    pub width: u32,
    pub height: u32,
    pub color_attachments: u8,
    pub has_depth: bool,
    pub has_stencil: bool,
    pub memory_usage: usize,
    pub debug_name: Option<String>,
}

#[async_trait]
impl Component for RenderTargetComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        buf.put_u64_le(self.handle.id());
        buf.put_u32_le(self.handle.generation());
        buf.put_u32_le(self.width);
        buf.put_u32_le(self.height);
        buf.put_u8(self.color_attachments);
        buf.put_u8(if self.has_depth { 1 } else { 0 });
        buf.put_u8(if self.has_stencil { 1 } else { 0 });
        buf.put_u64_le(self.memory_usage as u64);
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("RenderTargetComponent deserialization not implemented".into()))
    }
}

/// Component tracking render graph pass state
#[derive(Debug, Clone)]
pub struct RenderPassComponent {
    pub pass_id: PassId,
    pub name: String,
    pub pass_type: PassType,
    pub dependencies: Vec<PassId>,
    pub resources_read: HashSet<ResourceId>,
    pub resources_written: HashSet<ResourceId>,
    pub execution_order: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum PassType {
    Render,
    Compute,
    Copy,
    Blit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceId {
    Texture(u32),
    Buffer(u32),
    RenderTarget(u32),
}

#[async_trait]
impl Component for RenderPassComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize pass ID
        buf.put_u64_le(self.pass_id.value());
        
        // Serialize name
        buf.put_u32_le(self.name.len() as u32);
        buf.put_slice(self.name.as_bytes());
        
        // Serialize type
        let type_id = match self.pass_type {
            PassType::Render => 0u8,
            PassType::Compute => 1u8,
            PassType::Copy => 2u8,
            PassType::Blit => 3u8,
        };
        buf.put_u8(type_id);
        
        // Serialize dependencies
        buf.put_u32_le(self.dependencies.len() as u32);
        for dep in &self.dependencies {
            buf.put_u64_le(dep.value());
        }
        
        // Serialize execution order and enabled flag
        buf.put_u32_le(self.execution_order);
        buf.put_u8(if self.enabled { 1 } else { 0 });
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("RenderPassComponent deserialization not implemented".into()))
    }
}

/// Component tracking frame state
#[derive(Debug, Clone)]
pub struct FrameStateComponent {
    pub frame_number: u64,
    pub frame_time_ms: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
    pub draw_calls: u32,
    pub triangles_rendered: u64,
    pub state_changes: u32,
    pub texture_memory_used: usize,
    pub buffer_memory_used: usize,
}

#[async_trait]
impl Component for FrameStateComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        buf.put_u64_le(self.frame_number);
        buf.put_f32_le(self.frame_time_ms);
        buf.put_f32_le(self.cpu_time_ms);
        buf.put_f32_le(self.gpu_time_ms);
        buf.put_u32_le(self.draw_calls);
        buf.put_u64_le(self.triangles_rendered);
        buf.put_u32_le(self.state_changes);
        buf.put_u64_le(self.texture_memory_used as u64);
        buf.put_u64_le(self.buffer_memory_used as u64);
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("FrameStateComponent deserialization not implemented".into()))
    }
}

/// Component tracking renderer capabilities
#[derive(Debug, Clone)]
pub struct CapabilitiesComponent {
    pub max_texture_size: u32,
    pub max_vertex_attributes: u32,
    pub max_uniform_buffer_size: u32,
    pub max_storage_buffer_size: u32,
    pub features: RendererFeatures,
    pub renderer_name: String,
    pub vendor: String,
}

#[async_trait]
impl Component for CapabilitiesComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        buf.put_u32_le(self.max_texture_size);
        buf.put_u32_le(self.max_vertex_attributes);
        buf.put_u32_le(self.max_uniform_buffer_size);
        buf.put_u32_le(self.max_storage_buffer_size);
        
        // Serialize features as individual bools
        buf.put_u8(if self.features.compute_shaders { 1 } else { 0 });
        buf.put_u8(if self.features.geometry_shaders { 1 } else { 0 });
        buf.put_u8(if self.features.tessellation_shaders { 1 } else { 0 });
        buf.put_u8(if self.features.multi_draw_indirect { 1 } else { 0 });
        buf.put_u8(if self.features.bindless_textures { 1 } else { 0 });
        buf.put_u8(if self.features.ray_tracing { 1 } else { 0 });
        buf.put_u8(if self.features.mesh_shaders { 1 } else { 0 });
        buf.put_u8(if self.features.variable_rate_shading { 1 } else { 0 });
        
        // Serialize strings
        buf.put_u32_le(self.renderer_name.len() as u32);
        buf.put_slice(self.renderer_name.as_bytes());
        buf.put_u32_le(self.vendor.len() as u32);
        buf.put_slice(self.vendor.as_bytes());
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("CapabilitiesComponent deserialization not implemented".into()))
    }
}

/// Component for texture streaming priorities
#[derive(Debug, Clone)]
pub struct StreamingPriorityComponent {
    pub texture_handle: TextureHandle,
    pub priority: f32,
    pub requested_lod: u32,
    pub loading: bool,
}

#[async_trait]
impl Component for StreamingPriorityComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        buf.put_u64_le(self.texture_handle.id());
        buf.put_u32_le(self.texture_handle.generation());
        buf.put_f32_le(self.priority);
        buf.put_u32_le(self.requested_lod);
        buf.put_u8(if self.loading { 1 } else { 0 });
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_bytes: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("StreamingPriorityComponent deserialization not implemented".into()))
    }
}