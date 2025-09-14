use playground_core_rendering::{
    Renderer, RenderCommand, RenderCommandBatch, 
    RendererCapabilities, RenderResult, RenderError, RenderTargetWrapper
};
use playground_core_types::{Shared, shared, Handle};
use playground_core_ecs::{System as EcsSystem, ExecutionStage, EcsResult, WorldContract};
use crate::context::WebGLContext;
use crate::shader::ShaderProgram;
use crate::buffer::{VertexBuffer, IndexBuffer};
use crate::texture::Texture2D;
use std::collections::HashMap;
use async_trait::async_trait;
use nalgebra::{Matrix3, Vector2};

pub struct WebGLRenderer {
    context: Shared<WebGLContext>,
    shader_cache: Shared<HashMap<String, ShaderProgram>>,
    texture_cache: Shared<HashMap<u32, Texture2D>>,
    vertex_buffer: Shared<VertexBuffer>,
    index_buffer: Shared<IndexBuffer>,
    viewport: Shared<Viewport>,
    transform_stack: Shared<Vec<Matrix3<f32>>>,
    clip_stack: Shared<Vec<ClipRect>>,
    state_stack: Shared<Vec<RenderState>>,
    initialized: bool,
    frame_count: u64,
    command_count: u64,
}

#[derive(Clone, Copy)]
struct Viewport {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy)]
struct ClipRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone)]
struct RenderState {
    transform: Matrix3<f32>,
    clip_rect: Option<ClipRect>,
    opacity: f32,
}

impl WebGLRenderer {
    pub fn new() -> Self {
        Self {
            context: shared(WebGLContext::new()),
            shader_cache: shared(HashMap::new()),
            texture_cache: shared(HashMap::new()),
            vertex_buffer: shared(VertexBuffer::new(65536)),
            index_buffer: shared(IndexBuffer::new(65536 * 3)),
            viewport: shared(Viewport { x: 0, y: 0, width: 1920, height: 1080 }),
            transform_stack: shared(vec![Matrix3::identity()]),
            clip_stack: shared(Vec::new()),
            state_stack: shared(Vec::new()),
            initialized: false,
            frame_count: 0,
            command_count: 0,
        }
    }
    

    async fn execute_command(&mut self, command: &RenderCommand) -> RenderResult<()> {
        self.command_count += 1;
        
        match command {
            RenderCommand::Clear { color } => {
                self.clear(*color).await?;
            }
            RenderCommand::DrawQuad { position, size, color } => {
                self.draw_quad(*position, *size, *color).await?;
            }
            RenderCommand::DrawText { text, position, size, color } => {
                self.draw_text(text, *position, *size, *color).await?;
            }
            RenderCommand::DrawImage { texture_id, position, size, uv_min, uv_max } => {
                self.draw_image(*texture_id, *position, *size, *uv_min, *uv_max).await?;
            }
            RenderCommand::DrawLine { start, end, width, color } => {
                self.draw_line(*start, *end, *width, *color).await?;
            }
            RenderCommand::DrawCircle { center, radius, color, filled } => {
                self.draw_circle(*center, *radius, *color, *filled).await?;
            }
            RenderCommand::SetClipRect { position, size } => {
                self.set_clip_rect(*position, *size).await?;
            }
            RenderCommand::ClearClipRect => {
                self.clear_clip_rect().await?;
            }
            RenderCommand::SetTransform { matrix } => {
                self.set_transform(*matrix).await?;
            }
            RenderCommand::ResetTransform => {
                self.reset_transform().await?;
            }
            RenderCommand::PushState => {
                self.push_state().await?;
            }
            RenderCommand::PopState => {
                self.pop_state().await?;
            }
        }
        Ok(())
    }

    async fn clear(&mut self, color: [f32; 4]) -> RenderResult<()> {
        let context = self.context.read().await;
        context.clear_color(color[0], color[1], color[2], color[3]);
        context.clear();
        Ok(())
    }

    async fn draw_quad(&mut self, position: [f32; 2], size: [f32; 2], color: [f32; 4]) -> RenderResult<()> {
        let mut vertex_buffer = self.vertex_buffer.write().await;
        let mut index_buffer = self.index_buffer.write().await;
        
        let base_index = vertex_buffer.vertex_count() as u16;
        
        let color_struct = crate::buffer::Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        };
        
        let vertices = [
            [position[0], position[1], 0.0, 0.0],
            [position[0] + size[0], position[1], 1.0, 0.0],
            [position[0] + size[0], position[1] + size[1], 1.0, 1.0],
            [position[0], position[1] + size[1], 0.0, 1.0],
        ];
        
        for vertex in &vertices {
            vertex_buffer.push_vertex(vertex[0], vertex[1], vertex[2], vertex[3], color_struct);
        }
        
        let indices = [
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ];
        
        for index in &indices {
            index_buffer.push_index(*index);
        }
        
        Ok(())
    }

    async fn draw_text(&mut self, text: &str, position: [f32; 2], size: f32, color: [f32; 4]) -> RenderResult<()> {
        let char_width = size * 0.6;
        let mut x = position[0];
        
        for _ch in text.chars() {
            self.draw_quad(
                [x, position[1]],
                [char_width, size],
                color
            ).await?;
            x += char_width * 1.2;
        }
        
        Ok(())
    }

    async fn draw_image(&mut self, texture_id: u32, position: [f32; 2], size: [f32; 2], uv_min: [f32; 2], uv_max: [f32; 2]) -> RenderResult<()> {
        let mut vertex_buffer = self.vertex_buffer.write().await;
        let mut index_buffer = self.index_buffer.write().await;
        
        let base_index = vertex_buffer.vertex_count() as u16;
        
        let color = crate::buffer::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        
        let vertices = [
            [position[0], position[1], uv_min[0], uv_min[1]],
            [position[0] + size[0], position[1], uv_max[0], uv_min[1]],
            [position[0] + size[0], position[1] + size[1], uv_max[0], uv_max[1]],
            [position[0], position[1] + size[1], uv_min[0], uv_max[1]],
        ];
        
        for vertex in &vertices {
            vertex_buffer.push_vertex(vertex[0], vertex[1], vertex[2], vertex[3], color);
        }
        
        let indices = [
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ];
        
        for index in &indices {
            index_buffer.push_index(*index);
        }
        
        Ok(())
    }

    async fn draw_line(&mut self, start: [f32; 2], end: [f32; 2], width: f32, color: [f32; 4]) -> RenderResult<()> {
        let direction = Vector2::new(end[0] - start[0], end[1] - start[1]).normalize();
        let perpendicular = Vector2::new(-direction.y, direction.x) * (width * 0.5);
        
        let mut vertex_buffer = self.vertex_buffer.write().await;
        let mut index_buffer = self.index_buffer.write().await;
        
        let base_index = vertex_buffer.vertex_count() as u16;
        
        let color_struct = crate::buffer::Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        };
        
        let vertices = [
            Vector2::new(start[0], start[1]) - perpendicular,
            Vector2::new(start[0], start[1]) + perpendicular,
            Vector2::new(end[0], end[1]) + perpendicular,
            Vector2::new(end[0], end[1]) - perpendicular,
        ];
        
        for vertex in &vertices {
            vertex_buffer.push_vertex(vertex.x, vertex.y, 0.0, 0.0, color_struct);
        }
        
        let indices = [
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ];
        
        for index in &indices {
            index_buffer.push_index(*index);
        }
        
        Ok(())
    }

    async fn draw_circle(&mut self, center: [f32; 2], radius: f32, color: [f32; 4], filled: bool) -> RenderResult<()> {
        const SEGMENTS: usize = 32;
        
        if filled {
            let mut vertex_buffer = self.vertex_buffer.write().await;
            let mut index_buffer = self.index_buffer.write().await;
            
            let base_index = vertex_buffer.vertex_count() as u16;
            
            let color_struct = crate::buffer::Color {
                r: color[0],
                g: color[1],
                b: color[2],
                a: color[3],
            };
            
            vertex_buffer.push_vertex(center[0], center[1], 0.5, 0.5, color_struct);
            
            for i in 0..=SEGMENTS {
                let angle = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
                let x = center[0] + angle.cos() * radius;
                let y = center[1] + angle.sin() * radius;
                let u = 0.5 + angle.cos() * 0.5;
                let v = 0.5 + angle.sin() * 0.5;
                
                vertex_buffer.push_vertex(x, y, u, v, color_struct);
            }
            
            for i in 0..SEGMENTS {
                index_buffer.push_index(base_index);
                index_buffer.push_index(base_index + 1 + i as u16);
                index_buffer.push_index(base_index + 2 + i as u16);
            }
        } else {
            for i in 0..SEGMENTS {
                let angle1 = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
                let angle2 = ((i + 1) as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
                
                let start = [center[0] + angle1.cos() * radius, center[1] + angle1.sin() * radius];
                let end = [center[0] + angle2.cos() * radius, center[1] + angle2.sin() * radius];
                
                self.draw_line(start, end, 2.0, color).await?;
            }
        }
        
        Ok(())
    }

    async fn set_clip_rect(&mut self, position: [f32; 2], size: [f32; 2]) -> RenderResult<()> {
        self.clip_stack.write().await.push(ClipRect {
            x: position[0],
            y: position[1],
            width: size[0],
            height: size[1],
        });
        Ok(())
    }

    async fn clear_clip_rect(&mut self) -> RenderResult<()> {
        self.clip_stack.write().await.pop();
        Ok(())
    }

    async fn set_transform(&mut self, matrix: [[f32; 3]; 3]) -> RenderResult<()> {
        let transform = Matrix3::new(
            matrix[0][0], matrix[0][1], matrix[0][2],
            matrix[1][0], matrix[1][1], matrix[1][2],
            matrix[2][0], matrix[2][1], matrix[2][2],
        );
        *self.transform_stack.write().await = vec![transform];
        Ok(())
    }

    async fn reset_transform(&mut self) -> RenderResult<()> {
        *self.transform_stack.write().await = vec![Matrix3::identity()];
        Ok(())
    }

    async fn push_state(&mut self) -> RenderResult<()> {
        let transform = self.transform_stack.read().await.last().cloned().unwrap_or_else(Matrix3::identity);
        let clip_rect = self.clip_stack.read().await.last().cloned();
        
        self.state_stack.write().await.push(RenderState {
            transform,
            clip_rect,
            opacity: 1.0,
        });
        
        Ok(())
    }

    async fn pop_state(&mut self) -> RenderResult<()> {
        if let Some(state) = self.state_stack.write().await.pop() {
            *self.transform_stack.write().await = vec![state.transform];
            *self.clip_stack.write().await = if let Some(rect) = state.clip_rect {
                vec![rect]
            } else {
                Vec::new()
            };
        }
        Ok(())
    }

    async fn flush_buffers(&mut self) -> RenderResult<()> {
        let context = self.context.read().await;
        let vertex_buffer = self.vertex_buffer.read().await;
        let index_buffer = self.index_buffer.read().await;
        
        let vertex_count = vertex_buffer.vertex_count();
        let index_count = index_buffer.index_count();
        
        if vertex_count > 0 && index_count > 0 {
            // Flushing vertices and indices to GPU
            
            context.upload_vertices(&vertex_buffer);
            context.upload_indices(&index_buffer);
            context.draw_indexed(index_count);
            
            self.vertex_buffer.write().await.clear();
            self.index_buffer.write().await.clear();
        }
        
        Ok(())
    }
}

#[async_trait]
impl Renderer for WebGLRenderer {
    async fn initialize(&mut self) -> RenderResult<()> {
        // Initializing WebGL renderer
        
        self.context.write().await.initialize()
            .map_err(|e| {
                let err_msg = format!("WebGL initialization failed: {}", e);
                // Can't use async in closure, so we'll log after
                RenderError::InitializationFailed(err_msg.clone())
            })?;
        
        self.initialized = true;
        
        // WebGL renderer initialized successfully
        Ok(())
    }
    
    async fn begin_frame(&mut self) -> RenderResult<()> {
        self.frame_count += 1;
        self.command_count = 0;
        
        if self.frame_count % 60 == 0 {
            // Frame started
        }
        
        self.vertex_buffer.write().await.clear();
        self.index_buffer.write().await.clear();
        Ok(())
    }
    
    async fn execute_commands(&mut self, batch: &RenderCommandBatch) -> RenderResult<()> {
        let command_count = batch.commands().len();
        
        if command_count > 0 {
            // Executing render commands
        }
        
        if let Some(viewport) = batch.viewport() {
            // Setting viewport
            
            *self.viewport.write().await = Viewport {
                x: viewport.x,
                y: viewport.y,
                width: viewport.width,
                height: viewport.height,
            };
            let context = self.context.read().await;
            context.set_viewport(viewport.x as i32, viewport.y as i32, viewport.width as i32, viewport.height as i32);
        }
        
        for command in batch.commands() {
            self.execute_command(command).await?;
        }
        
        if batch.commands().len() > 100 {
            // Flushing buffers
            self.flush_buffers().await?;
        }
        
        Ok(())
    }
    
    async fn end_frame(&mut self) -> RenderResult<()> {
        self.flush_buffers().await?;
        
        if self.frame_count % 60 == 0 {
            // Frame completed
        }
        
        Ok(())
    }
    
    async fn present(&mut self) -> RenderResult<()> {
        // WebGL automatically presents when the frame ends
        Ok(())
    }
    
    async fn create_render_target(&mut self, width: u32, height: u32) -> RenderResult<RenderTargetWrapper> {
        // Return error for now since WebGL render targets aren't implemented yet
        // When implemented, would create a framebuffer and wrap it in RenderTargetWrapper
        Err(RenderError::UnsupportedFeature("create_render_target not yet implemented".into()))
    }
    
    async fn shutdown(&mut self) -> RenderResult<()> {
        // Shutting down WebGL renderer
        
        self.initialized = false;
        
        // WebGL renderer shutdown complete
        
        Ok(())
    }
    
    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            max_texture_size: 4096,
            max_render_targets: 8,
            supports_compute: false,
            supports_instancing: true,
            supports_tessellation: false,
            max_vertex_attributes: 16,
            max_uniform_buffer_size: 65536,
        }
    }
    
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

// Additional methods for WebGLRenderer
impl WebGLRenderer {
    pub async fn render_batch(&mut self, commands: &[RenderCommand]) -> RenderResult<()> {
        // Process batch of render commands
        for command in commands {
            self.process_command(command.clone()).await?;
        }
        Ok(())
    }
    
    pub async fn resize(&mut self, width: u32, height: u32) -> RenderResult<()> {
        // Resizing renderer
        *self.viewport.write().await = Viewport { x: 0, y: 0, width, height };
        let context = self.context.read().await;
        context.set_viewport(0, 0, width as i32, height as i32);
        Ok(())
    }
}

// Implement the ECS System trait for WebGLRenderer
#[async_trait]
impl EcsSystem for WebGLRenderer {
    fn name(&self) -> &str {
        "WebGLRenderer"
    }
    
    fn stage(&self) -> ExecutionStage {
        ExecutionStage::Render
    }
    
    async fn initialize(&mut self) -> EcsResult<()> {
        // Initialize the renderer
        Renderer::initialize(self).await
            .map_err(|e| playground_core_ecs::EcsError::SystemError(format!("Failed to initialize WebGL: {}", e)))
    }
    
    async fn update(&mut self, delta_time: f32) -> EcsResult<()> {
        // In a proper implementation, this would:
        // 1. Query the World for RenderCommandBatch components
        // 2. Process each batch through the renderer
        // 3. Present the frame
        
        // For now, just track frame timing
        self.frame_count += 1;
        
        // Note: The actual rendering will be triggered by receiving
        // RenderCommandBatch through channels from other systems
        
        Ok(())
    }
    
    async fn cleanup(&mut self) -> EcsResult<()> {
        // Shutdown the renderer
        Renderer::shutdown(self).await
            .map_err(|e| playground_core_ecs::EcsError::SystemError(format!("Failed to shutdown WebGL: {}", e)))
    }
}