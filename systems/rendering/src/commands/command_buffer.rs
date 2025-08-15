use crate::resources::{PipelineHandle, VertexBuffer, IndexBuffer, UniformBuffer, TextureHandle};
use crate::resources::render_target::RenderTargetHandle;

pub trait CommandBuffer: Send + Sync {
    fn begin(&mut self);
    fn end(&mut self);
    
    fn set_pipeline(&mut self, pipeline: PipelineHandle);
    fn set_vertex_buffer(&mut self, slot: u32, buffer: &VertexBuffer);
    fn set_index_buffer(&mut self, buffer: &IndexBuffer);
    fn set_uniform_buffer(&mut self, slot: u32, buffer: &UniformBuffer);
    fn set_texture(&mut self, slot: u32, texture: TextureHandle);
    
    fn set_render_target(&mut self, target: RenderTargetHandle);
    fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32);
    fn set_scissor(&mut self, x: u32, y: u32, width: u32, height: u32);
    
    fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32);
    fn draw_indexed(&mut self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32);
    
    fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32);
    fn clear_depth(&mut self, depth: f32);
    fn clear_stencil(&mut self, stencil: u32);
}