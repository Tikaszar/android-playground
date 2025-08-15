use crate::error::RendererError;
use crate::graph::RenderGraph;
use crate::graph::pass::PassId;
use crate::graph::GraphTemplateId;
use crate::resources::{
    TextureHandle, TextureDesc, TextureRegion,
    ShaderHandle, ShaderStage,
    PipelineHandle, PipelineDesc,
    RenderTargetHandle, RenderTargetDesc,
    Handle,
};
use crate::resources::buffer::{
    VertexBuffer, IndexBuffer, UniformBuffer, StorageBuffer,
    VertexFormat, IndexType,
};
use crate::commands::{CommandBuffer, SyncPoint};
use crate::sync::SyncPromise;
use crate::metrics::RenderMetrics;
use crate::capabilities::RendererCapabilities;
use crate::compute::ComputeResources;
use crate::streaming::TileCoord;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub trait BaseRenderer: Send + Sync {
    type Error: From<RendererError>;
    
    // === Lifecycle ===
    fn initialize(&mut self) -> Result<(), Self::Error>;
    fn shutdown(&mut self) -> Result<(), Self::Error>;
    
    // === Frame Management (single batched draw) ===
    fn begin_frame(&mut self) -> Result<(), Self::Error>;
    fn render(&mut self, graph: &RenderGraph) -> Result<(), Self::Error>;
    fn end_frame(&mut self) -> Result<(), Self::Error>;
    fn present(&mut self) -> Result<(), Self::Error>;
    
    // === Resource Creation ===
    fn create_texture(&mut self, desc: &TextureDesc) -> Result<TextureHandle, Self::Error>;
    fn create_vertex_buffer(&mut self, data: &[u8], format: &VertexFormat) -> Result<VertexBuffer, Self::Error>;
    fn create_index_buffer(&mut self, data: &[u8], index_type: IndexType) -> Result<IndexBuffer, Self::Error>;
    fn create_uniform_buffer(&mut self, size: usize) -> Result<UniformBuffer, Self::Error>;
    fn create_storage_buffer(&mut self, size: usize, read_only: bool) -> Result<StorageBuffer, Self::Error>;
    fn create_shader(&mut self, path: &str, stage: ShaderStage) -> Result<ShaderHandle, Self::Error>;
    fn create_compute_shader(&mut self, path: &str) -> Result<ShaderHandle, Self::Error>;
    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineHandle, Self::Error>;
    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> Result<RenderTargetHandle, Self::Error>;
    
    // === Resource Updates ===
    fn update_vertex_buffer(&mut self, buffer: &VertexBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error>;
    fn update_index_buffer(&mut self, buffer: &IndexBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error>;
    fn update_uniform_buffer(&mut self, buffer: &UniformBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error>;
    fn update_storage_buffer(&mut self, buffer: &StorageBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error>;
    fn update_texture(&mut self, handle: TextureHandle, data: &[u8], region: Option<TextureRegion>) -> Result<(), Self::Error>;
    
    // === Resource Destruction ===
    fn destroy_texture(&mut self, handle: TextureHandle) -> Result<(), Self::Error>;
    fn destroy_vertex_buffer(&mut self, buffer: VertexBuffer) -> Result<(), Self::Error>;
    fn destroy_index_buffer(&mut self, buffer: IndexBuffer) -> Result<(), Self::Error>;
    fn destroy_uniform_buffer(&mut self, buffer: UniformBuffer) -> Result<(), Self::Error>;
    fn destroy_storage_buffer(&mut self, buffer: StorageBuffer) -> Result<(), Self::Error>;
    fn destroy_shader(&mut self, handle: ShaderHandle) -> Result<(), Self::Error>;
    fn destroy_pipeline(&mut self, handle: PipelineHandle) -> Result<(), Self::Error>;
    
    // === Command System ===
    fn create_command_buffer(&mut self) -> Arc<RwLock<dyn CommandBuffer>>;
    fn submit_command_buffer(&mut self, buffer: Arc<RwLock<dyn CommandBuffer>>, dependencies: Vec<SyncPoint>);
    
    // === Graph Management ===
    fn set_render_graph(&mut self, graph: RenderGraph) -> Result<(), Self::Error>;
    fn swap_graph_template(&mut self, template_id: GraphTemplateId) -> Result<(), Self::Error>;
    fn add_render_pass(&mut self, pass: Box<dyn crate::graph::pass::Pass>) -> Result<PassId, Self::Error>;
    fn add_compute_pass(&mut self, pass: Box<dyn crate::graph::pass::Pass>) -> Result<PassId, Self::Error>;
    fn remove_pass(&mut self, id: PassId) -> Result<(), Self::Error>;
    
    // === Compute (stubbed in WebGL) ===
    fn dispatch_compute(&mut self, x: u32, y: u32, z: u32, shader: ShaderHandle, resources: &ComputeResources) -> Result<(), Self::Error>;
    
    // === Texture Streaming ===
    fn request_texture_lod(&mut self, handle: TextureHandle, lod: u32) -> SyncPromise<()>;
    fn stream_texture_tile(&mut self, handle: TextureHandle, tile: TileCoord) -> SyncPromise<()>;
    fn get_streaming_budget(&self) -> usize;
    fn update_streaming_priorities(&mut self, priorities: HashMap<TextureHandle, f32>);
    
    // === Debug ===
    fn set_debug_name<T>(&mut self, handle: Handle<T>, name: &str) -> Result<(), Self::Error>;
    fn push_debug_marker(&mut self, name: &str);
    fn pop_debug_marker(&mut self);
    
    // === Metrics ===
    fn get_metrics(&self) -> &RenderMetrics;
    fn reset_metrics(&mut self);
    
    // === Hot-Reload ===
    fn register_shader_watch(&mut self, handle: ShaderHandle) -> Result<(), Self::Error>;
    fn check_shader_updates(&mut self) -> Vec<ShaderHandle>;
    fn reload_shader(&mut self, handle: ShaderHandle) -> Result<(), Self::Error>;
    
    // === Capabilities ===
    fn capabilities(&self) -> &RendererCapabilities;
    fn validate_handle<T>(&self, handle: Handle<T>) -> bool;
    
    // === Synchronization ===
    fn wait_idle(&mut self) -> Result<(), Self::Error>;
    fn create_sync_point(&mut self) -> SyncPromise<()>;
    
    // === Device Recovery ===
    fn handle_device_lost(&mut self) -> Result<(), Self::Error>;
}