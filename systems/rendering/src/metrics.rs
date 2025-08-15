use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetrics {
    pub frame_time_ms: f32,
    pub draw_calls: u32,
    pub triangles_rendered: u32,
    pub vertices_processed: u32,
    pub state_changes: u32,
    pub texture_switches: u32,
    pub shader_switches: u32,
    pub pipeline_switches: u32,
    pub buffer_uploads_bytes: usize,
    pub texture_uploads_bytes: usize,
    pub gpu_memory_used: usize,
    pub cpu_memory_used: usize,
    pub command_buffers_submitted: u32,
}

impl Default for RenderMetrics {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            draw_calls: 0,
            triangles_rendered: 0,
            vertices_processed: 0,
            state_changes: 0,
            texture_switches: 0,
            shader_switches: 0,
            pipeline_switches: 0,
            buffer_uploads_bytes: 0,
            texture_uploads_bytes: 0,
            gpu_memory_used: 0,
            cpu_memory_used: 0,
            command_buffers_submitted: 0,
        }
    }
}

impl RenderMetrics {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    pub fn log_debug(&self) {
        log::debug!("=== Render Metrics ===");
        log::debug!("Frame time: {:.2}ms", self.frame_time_ms);
        log::debug!("Draw calls: {}", self.draw_calls);
        log::debug!("Triangles: {}", self.triangles_rendered);
        log::debug!("State changes: {}", self.state_changes);
        log::debug!("GPU memory: {} MB", self.gpu_memory_used / (1024 * 1024));
        log::debug!("CPU memory: {} MB", self.cpu_memory_used / (1024 * 1024));
    }
}