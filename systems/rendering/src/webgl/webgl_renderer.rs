use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::base_renderer::BaseRenderer;
use crate::capabilities::{RendererCapabilities, RendererFeatures};
use crate::commands::{CommandBuffer, SyncPoint};
use crate::compute::ComputeResources;
use crate::error::RendererError;
use crate::graph::{RenderGraph, GraphTemplateId, PassId};
use crate::graph::pass::Pass;
use crate::metrics::RenderMetrics;
use crate::resources::*;
use crate::streaming::TileCoord;
use crate::sync::SyncPromise;

use super::context::WebGLContext;
use super::resource_manager::ResourceManager;
use super::shader_compiler::ShaderCompiler;
use super::state_cache::StateCache;

pub struct WebGLRenderer {
    context: Option<WebGLContext>,
    resources: ResourceManager,
    shader_compiler: ShaderCompiler,
    state_cache: StateCache,
    capabilities: RendererCapabilities,
    metrics: RenderMetrics,
    current_graph: Option<RenderGraph>,
    graph_templates: HashMap<GraphTemplateId, RenderGraph>,
    shader_watch_list: Vec<ShaderHandle>,
    initialized: bool,
}

impl WebGLRenderer {
    pub fn new() -> Self {
        log::info!("Creating WebGL renderer");
        
        let capabilities = RendererCapabilities {
            max_texture_size: 4096,
            max_texture_units: 16,
            max_vertex_attributes: 16,
            max_uniform_buffer_size: 65536,
            max_storage_buffer_size: 0, // Not supported in WebGL
            max_color_attachments: 8,
            max_compute_work_group_size: [0, 0, 0], // Not supported
            max_compute_work_group_count: [0, 0, 0], // Not supported
            supported_texture_formats: vec![
                TextureFormat::RGBA8,
                TextureFormat::RGB8,
                TextureFormat::RG8,
                TextureFormat::R8,
                TextureFormat::RGBA16F,
                TextureFormat::RGBA32F,
                TextureFormat::Depth24,
                TextureFormat::Depth24Stencil8,
            ],
            features: RendererFeatures {
                compute_shaders: false,
                geometry_shaders: false,
                tessellation_shaders: false,
                multi_draw_indirect: false,
                bindless_textures: false,
                ray_tracing: false,
                mesh_shaders: false,
                variable_rate_shading: false,
            },
        };

        Self {
            context: None,
            resources: ResourceManager::new(),
            shader_compiler: ShaderCompiler::new(),
            state_cache: StateCache::new(),
            capabilities,
            metrics: RenderMetrics::default(),
            current_graph: None,
            graph_templates: HashMap::new(),
            shader_watch_list: Vec::new(),
            initialized: false,
        }
    }

    pub fn with_canvas(canvas: HtmlCanvasElement) -> Result<Self, RendererError> {
        let mut renderer = Self::new();
        renderer.context = Some(WebGLContext::new(canvas)?);
        Ok(renderer)
    }

    fn gl(&self) -> Result<&WebGl2RenderingContext, RendererError> {
        self.context
            .as_ref()
            .map(|ctx| ctx.gl())
            .ok_or_else(|| RendererError::InitializationFailed("WebGL context not initialized".to_string()))
    }

    fn check_initialized(&self) -> Result<(), RendererError> {
        if !self.initialized {
            return Err(RendererError::InitializationFailed("Renderer not initialized".to_string()));
        }
        Ok(())
    }
}

impl BaseRenderer for WebGLRenderer {
    type Error = RendererError;

    fn initialize(&mut self) -> Result<(), Self::Error> {
        if self.initialized {
            return Ok(());
        }

        log::info!("Initializing WebGL renderer");

        let gl = self.gl()?;
        
        // Enable common features
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.cull_face(WebGl2RenderingContext::BACK);
        
        // Set default clear color
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        // Query actual capabilities
        if let Some(max_texture_size) = gl.get_parameter(WebGl2RenderingContext::MAX_TEXTURE_SIZE).ok() {
            if let Some(size) = max_texture_size.as_f64() {
                self.capabilities.max_texture_size = size as u32;
            }
        }

        self.initialized = true;
        log::info!("WebGL renderer initialized successfully");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        log::info!("Shutting down WebGL renderer");
        self.resources.clear();
        self.shader_compiler.clear_cache();
        self.state_cache.reset();
        self.initialized = false;
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<(), Self::Error> {
        self.check_initialized()?;
        self.metrics.reset();
        let start_time = js_sys::Date::now();
        self.metrics.frame_time_ms = start_time as f32;
        Ok(())
    }

    fn render(&mut self, graph: &RenderGraph) -> Result<(), Self::Error> {
        self.check_initialized()?;
        
        let gl = self.gl()?;
        
        // Clear default framebuffer
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // Execute render graph passes
        for pass_id in graph.execution_order() {
            if let Some(pass) = graph.get_pass(*pass_id) {
                log::debug!("Executing pass: {}", pass.name());
                // TODO: Create command buffer and execute pass
            }
        }

        self.metrics.draw_calls += 1;
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), Self::Error> {
        self.check_initialized()?;
        let end_time = js_sys::Date::now();
        self.metrics.frame_time_ms = (end_time as f32) - self.metrics.frame_time_ms;
        Ok(())
    }

    fn present(&mut self) -> Result<(), Self::Error> {
        // WebGL automatically presents when returning to browser event loop
        Ok(())
    }

    fn create_texture(&mut self, desc: &TextureDesc) -> Result<TextureHandle, Self::Error> {
        let gl = self.gl()?;
        
        let texture = gl.create_texture()
            .ok_or_else(|| RendererError::ResourceCreationFailed("Failed to create texture".to_string()))?;

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        
        // Set texture parameters
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, WebGl2RenderingContext::LINEAR as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, WebGl2RenderingContext::LINEAR as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);

        let handle = self.resources.store_texture(texture);
        log::debug!("Created texture with handle {:?}", handle);
        Ok(handle)
    }

    fn create_vertex_buffer(&mut self, data: &[u8], format: &VertexFormat) -> Result<VertexBuffer, Self::Error> {
        let gl = self.gl()?;
        
        let buffer = gl.create_buffer()
            .ok_or_else(|| RendererError::ResourceCreationFailed("Failed to create vertex buffer".to_string()))?;

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
        
        // SAFETY: WebGL expects raw bytes
        let array = unsafe { js_sys::Uint8Array::view(data) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &array,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        let handle = self.resources.store_buffer(buffer);
        let vertex_count = (data.len() / format.stride) as u32;
        let vb = VertexBuffer::new(handle, format.clone(), vertex_count);
        self.resources.store_vertex_buffer(handle, vb.clone());

        log::debug!("Created vertex buffer with {} vertices", vertex_count);
        Ok(vb)
    }

    fn create_index_buffer(&mut self, data: &[u8], index_type: IndexType) -> Result<IndexBuffer, Self::Error> {
        let gl = self.gl()?;
        
        let buffer = gl.create_buffer()
            .ok_or_else(|| RendererError::ResourceCreationFailed("Failed to create index buffer".to_string()))?;

        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        
        let array = unsafe { js_sys::Uint8Array::view(data) };
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &array,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        let handle = self.resources.store_buffer(buffer);
        let index_size = match index_type {
            IndexType::U16 => 2,
            IndexType::U32 => 4,
        };
        let index_count = (data.len() / index_size) as u32;
        let ib = IndexBuffer::new(handle, index_type, index_count);
        self.resources.store_index_buffer(handle, ib.clone());

        log::debug!("Created index buffer with {} indices", index_count);
        Ok(ib)
    }

    fn create_uniform_buffer(&mut self, size: usize) -> Result<UniformBuffer, Self::Error> {
        let gl = self.gl()?;
        
        let buffer = gl.create_buffer()
            .ok_or_else(|| RendererError::ResourceCreationFailed("Failed to create uniform buffer".to_string()))?;

        gl.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&buffer));
        gl.buffer_data_with_i32(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            size as i32,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        let handle = self.resources.store_buffer(buffer);
        let ub = UniformBuffer::new(handle, size);
        self.resources.store_uniform_buffer(handle, ub.clone());

        log::debug!("Created uniform buffer with size {}", size);
        Ok(ub)
    }

    fn create_storage_buffer(&mut self, _size: usize, _read_only: bool) -> Result<StorageBuffer, Self::Error> {
        Err(RendererError::NotSupported("Storage buffers not supported in WebGL".to_string()))
    }

    fn create_shader(&mut self, path: &str, stage: ShaderStage) -> Result<ShaderHandle, Self::Error> {
        // In a real implementation, we'd load from file
        // For now, use path as source
        let source = path;
        
        let gl = self.gl()?;
        let shader = self.shader_compiler.compile_shader(gl, source, stage)?;
        
        // Note: We store individual shaders but need to link them into programs
        // This is a simplified version
        Ok(Handle::new(0, 0))
    }

    fn create_compute_shader(&mut self, _path: &str) -> Result<ShaderHandle, Self::Error> {
        Err(RendererError::NotSupported("Compute shaders not supported in WebGL".to_string()))
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> Result<PipelineHandle, Self::Error> {
        // TODO: Create VAO and link shaders into program
        log::debug!("Creating pipeline");
        Ok(Handle::new(0, 0))
    }

    fn create_render_target(&mut self, desc: &RenderTargetDesc) -> Result<RenderTargetHandle, Self::Error> {
        let gl = self.gl()?;
        
        let framebuffer = gl.create_framebuffer()
            .ok_or_else(|| RendererError::ResourceCreationFailed("Failed to create framebuffer".to_string()))?;

        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));

        // TODO: Attach color and depth textures based on desc

        let handle = self.resources.store_framebuffer(framebuffer);
        log::debug!("Created render target {:?}", handle);
        Ok(handle)
    }

    fn update_vertex_buffer(&mut self, buffer: &VertexBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
        let gl = self.gl()?;
        
        if let Some(webgl_buffer) = self.resources.get_buffer(buffer.handle) {
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(webgl_buffer));
            let array = unsafe { js_sys::Uint8Array::view(data) };
            gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                offset as i32,
                &array,
            );
            self.metrics.buffer_uploads_bytes += data.len();
        } else {
            return Err(RendererError::InvalidHandle);
        }
        Ok(())
    }

    fn update_index_buffer(&mut self, buffer: &IndexBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
        let gl = self.gl()?;
        
        if let Some(webgl_buffer) = self.resources.get_buffer(buffer.handle) {
            gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(webgl_buffer));
            let array = unsafe { js_sys::Uint8Array::view(data) };
            gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                offset as i32,
                &array,
            );
            self.metrics.buffer_uploads_bytes += data.len();
        } else {
            return Err(RendererError::InvalidHandle);
        }
        Ok(())
    }

    fn update_uniform_buffer(&mut self, buffer: &UniformBuffer, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
        let gl = self.gl()?;
        
        if let Some(webgl_buffer) = self.resources.get_buffer(buffer.handle) {
            gl.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(webgl_buffer));
            let array = unsafe { js_sys::Uint8Array::view(data) };
            gl.buffer_sub_data_with_i32_and_array_buffer_view(
                WebGl2RenderingContext::UNIFORM_BUFFER,
                offset as i32,
                &array,
            );
            self.metrics.buffer_uploads_bytes += data.len();
        } else {
            return Err(RendererError::InvalidHandle);
        }
        Ok(())
    }

    fn update_storage_buffer(&mut self, _buffer: &StorageBuffer, _offset: usize, _data: &[u8]) -> Result<(), Self::Error> {
        Err(RendererError::NotSupported("Storage buffers not supported in WebGL".to_string()))
    }

    fn update_texture(&mut self, handle: TextureHandle, data: &[u8], region: Option<TextureRegion>) -> Result<(), Self::Error> {
        let gl = self.gl()?;
        
        if let Some(texture) = self.resources.get_texture(handle) {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            // TODO: Upload texture data based on region
            self.metrics.texture_uploads_bytes += data.len();
        } else {
            return Err(RendererError::InvalidHandle);
        }
        Ok(())
    }

    fn destroy_texture(&mut self, handle: TextureHandle) -> Result<(), Self::Error> {
        if let Some(texture) = self.resources.remove_texture(handle) {
            let gl = self.gl()?;
            gl.delete_texture(Some(&texture));
        }
        Ok(())
    }

    fn destroy_vertex_buffer(&mut self, buffer: VertexBuffer) -> Result<(), Self::Error> {
        if let Some(webgl_buffer) = self.resources.remove_buffer(buffer.handle) {
            let gl = self.gl()?;
            gl.delete_buffer(Some(&webgl_buffer));
        }
        Ok(())
    }

    fn destroy_index_buffer(&mut self, buffer: IndexBuffer) -> Result<(), Self::Error> {
        if let Some(webgl_buffer) = self.resources.remove_buffer(buffer.handle) {
            let gl = self.gl()?;
            gl.delete_buffer(Some(&webgl_buffer));
        }
        Ok(())
    }

    fn destroy_uniform_buffer(&mut self, buffer: UniformBuffer) -> Result<(), Self::Error> {
        if let Some(webgl_buffer) = self.resources.remove_buffer(buffer.handle) {
            let gl = self.gl()?;
            gl.delete_buffer(Some(&webgl_buffer));
        }
        Ok(())
    }

    fn destroy_storage_buffer(&mut self, _buffer: StorageBuffer) -> Result<(), Self::Error> {
        Err(RendererError::NotSupported("Storage buffers not supported in WebGL".to_string()))
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) -> Result<(), Self::Error> {
        if let Some(program) = self.resources.remove_program(handle) {
            let gl = self.gl()?;
            gl.delete_program(Some(&program));
        }
        Ok(())
    }

    fn destroy_pipeline(&mut self, _handle: PipelineHandle) -> Result<(), Self::Error> {
        // TODO: Delete VAO and program
        Ok(())
    }

    fn create_command_buffer(&mut self) -> Arc<RwLock<dyn CommandBuffer>> {
        Arc::new(RwLock::new(WebGLCommandBuffer::new()))
    }

    fn submit_command_buffer(&mut self, _buffer: Arc<RwLock<dyn CommandBuffer>>, _dependencies: Vec<SyncPoint>) {
        // TODO: Execute command buffer
        self.metrics.command_buffers_submitted += 1;
    }

    fn set_render_graph(&mut self, graph: RenderGraph) -> Result<(), Self::Error> {
        self.current_graph = Some(graph);
        Ok(())
    }

    fn swap_graph_template(&mut self, template_id: GraphTemplateId) -> Result<(), Self::Error> {
        if let Some(graph) = self.graph_templates.get(&template_id).cloned() {
            self.current_graph = Some(graph);
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn add_render_pass(&mut self, pass: Box<dyn Pass>) -> Result<PassId, Self::Error> {
        if let Some(graph) = &mut self.current_graph {
            Ok(graph.add_pass(pass))
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn add_compute_pass(&mut self, _pass: Box<dyn Pass>) -> Result<PassId, Self::Error> {
        Err(RendererError::NotSupported("Compute passes not supported in WebGL".to_string()))
    }

    fn remove_pass(&mut self, id: PassId) -> Result<(), Self::Error> {
        if let Some(graph) = &mut self.current_graph {
            graph.remove_pass(id);
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn dispatch_compute(&mut self, _x: u32, _y: u32, _z: u32, _shader: ShaderHandle, _resources: &ComputeResources) -> Result<(), Self::Error> {
        Err(RendererError::NotSupported("Compute shaders not supported in WebGL".to_string()))
    }

    fn request_texture_lod(&mut self, _handle: TextureHandle, _lod: u32) -> SyncPromise<()> {
        // TODO: Implement texture streaming
        SyncPromise::default()
    }

    fn stream_texture_tile(&mut self, _handle: TextureHandle, _tile: TileCoord) -> SyncPromise<()> {
        // TODO: Implement texture streaming
        SyncPromise::default()
    }

    fn get_streaming_budget(&self) -> usize {
        256 * 1024 * 1024 // 256MB default
    }

    fn update_streaming_priorities(&mut self, _priorities: HashMap<TextureHandle, f32>) {
        // TODO: Implement streaming priorities
    }

    fn set_debug_name<T>(&mut self, _handle: Handle<T>, name: &str) -> Result<(), Self::Error> {
        log::debug!("Debug name set: {}", name);
        Ok(())
    }

    fn push_debug_marker(&mut self, name: &str) {
        log::debug!("Debug marker pushed: {}", name);
    }

    fn pop_debug_marker(&mut self) {
        log::debug!("Debug marker popped");
    }

    fn get_metrics(&self) -> &RenderMetrics {
        &self.metrics
    }

    fn reset_metrics(&mut self) {
        self.metrics.reset();
    }

    fn register_shader_watch(&mut self, handle: ShaderHandle) -> Result<(), Self::Error> {
        self.shader_watch_list.push(handle);
        Ok(())
    }

    fn check_shader_updates(&mut self) -> Vec<ShaderHandle> {
        // TODO: Check file modification times
        Vec::new()
    }

    fn reload_shader(&mut self, _handle: ShaderHandle) -> Result<(), Self::Error> {
        // TODO: Reload and recompile shader
        Ok(())
    }

    fn capabilities(&self) -> &RendererCapabilities {
        &self.capabilities
    }

    fn validate_handle<T>(&self, handle: Handle<T>) -> bool {
        handle.is_valid()
    }

    fn wait_idle(&mut self) -> Result<(), Self::Error> {
        // WebGL is synchronous
        Ok(())
    }

    fn create_sync_point(&mut self) -> SyncPromise<()> {
        SyncPromise::default()
    }

    fn handle_device_lost(&mut self) -> Result<(), Self::Error> {
        log::error!("WebGL context lost, attempting recovery");
        // TODO: Recreate context and resources
        self.shutdown()?;
        self.initialize()?;
        Ok(())
    }
}

// Simple command buffer implementation for WebGL
struct WebGLCommandBuffer {
    commands: Vec<RenderCommand>,
}

enum RenderCommand {
    SetPipeline(PipelineHandle),
    SetVertexBuffer(u32, BufferHandle),
    SetIndexBuffer(BufferHandle),
    Draw(u32, u32, u32, u32),
    DrawIndexed(u32, u32, u32, i32, u32),
    Clear(f32, f32, f32, f32),
}

impl WebGLCommandBuffer {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

impl CommandBuffer for WebGLCommandBuffer {
    fn begin(&mut self) {
        self.commands.clear();
    }

    fn end(&mut self) {
        // No-op
    }

    fn set_pipeline(&mut self, pipeline: PipelineHandle) {
        self.commands.push(RenderCommand::SetPipeline(pipeline));
    }

    fn set_vertex_buffer(&mut self, slot: u32, buffer: &VertexBuffer) {
        self.commands.push(RenderCommand::SetVertexBuffer(slot, buffer.handle));
    }

    fn set_index_buffer(&mut self, buffer: &IndexBuffer) {
        self.commands.push(RenderCommand::SetIndexBuffer(buffer.handle));
    }

    fn set_uniform_buffer(&mut self, _slot: u32, _buffer: &UniformBuffer) {
        // TODO
    }

    fn set_texture(&mut self, _slot: u32, _texture: TextureHandle) {
        // TODO
    }

    fn set_render_target(&mut self, _target: RenderTargetHandle) {
        // TODO
    }

    fn set_viewport(&mut self, _x: f32, _y: f32, _width: f32, _height: f32) {
        // TODO
    }

    fn set_scissor(&mut self, _x: u32, _y: u32, _width: u32, _height: u32) {
        // TODO
    }

    fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        self.commands.push(RenderCommand::Draw(vertex_count, instance_count, first_vertex, first_instance));
    }

    fn draw_indexed(&mut self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        self.commands.push(RenderCommand::DrawIndexed(index_count, instance_count, first_index, vertex_offset, first_instance));
    }

    fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.commands.push(RenderCommand::Clear(r, g, b, a));
    }

    fn clear_depth(&mut self, _depth: f32) {
        // TODO
    }

    fn clear_stencil(&mut self, _stencil: u32) {
        // TODO
    }
}