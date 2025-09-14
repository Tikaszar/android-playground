//! Generic client contracts

use async_trait::async_trait;
use crate::types::*;
use crate::input::InputEvent;
use playground_core_rendering::RenderCommand;
use playground_core_types::CoreResult;

/// Generic contract for any client implementation
#[async_trait]
pub trait ClientContract: Send + Sync {
    /// Initialize the client
    async fn initialize(&mut self, config: ClientConfig) -> CoreResult<()>;
    
    /// Connect to a server
    async fn connect(&mut self, address: &str) -> CoreResult<()>;
    
    /// Disconnect from server
    async fn disconnect(&mut self) -> CoreResult<()>;
    
    /// Get current client state
    fn state(&self) -> ClientState;
    
    /// Get client ID
    fn id(&self) -> ClientId;
    
    /// Send a message to the server
    async fn send(&mut self, data: Vec<u8>) -> CoreResult<()>;
    
    /// Receive a message from the server (if available)
    async fn receive(&mut self) -> CoreResult<Option<Vec<u8>>>;
    
    /// Update the client (called each frame)
    async fn update(&mut self, delta_time: f32) -> CoreResult<()>;
    
    /// Get client statistics
    fn stats(&self) -> ClientStats;
    
    /// Get client capabilities
    fn capabilities(&self) -> ClientCapabilities;
}

/// Contract for clients that can render
#[async_trait]
pub trait RenderingClientContract: ClientContract {
    /// Create a render target
    async fn create_render_target(&mut self, target: RenderTarget) -> CoreResult<u32>;
    
    /// Destroy a render target
    async fn destroy_render_target(&mut self, id: u32) -> CoreResult<()>;
    
    /// Get current render target
    fn current_render_target(&self) -> Option<u32>;
    
    /// Set current render target
    async fn set_render_target(&mut self, id: u32) -> CoreResult<()>;
    
    /// Submit render commands
    async fn render(&mut self, commands: Vec<RenderCommand>) -> CoreResult<()>;
    
    /// Present the rendered frame
    async fn present(&mut self) -> CoreResult<()>;
    
    /// Resize render target
    async fn resize(&mut self, id: u32, width: u32, height: u32) -> CoreResult<()>;
}

/// Contract for clients that can handle input
#[async_trait]
pub trait InputClientContract: ClientContract {
    /// Poll for input events
    async fn poll_input(&mut self) -> CoreResult<Vec<InputEvent>>;
    
    /// Set input capture mode (e.g., capture mouse)
    async fn set_input_capture(&mut self, capture: bool) -> CoreResult<()>;
    
    /// Check if a key is currently pressed
    fn is_key_pressed(&self, key: crate::input::KeyCode) -> bool;
    
    /// Get current pointer position
    fn pointer_position(&self) -> Option<(f32, f32)>;
}

/// Contract for audio-capable clients
#[async_trait]
pub trait AudioClientContract: ClientContract {
    /// Play an audio buffer
    async fn play_audio(&mut self, data: Vec<u8>, format: AudioFormat) -> CoreResult<u32>;
    
    /// Stop audio playback
    async fn stop_audio(&mut self, id: u32) -> CoreResult<()>;
    
    /// Set audio volume (0.0 to 1.0)
    async fn set_volume(&mut self, volume: f32) -> CoreResult<()>;
    
    /// Get current volume
    fn volume(&self) -> f32;
}

/// Audio format information
#[derive(Debug, Clone, Copy)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u8,
    pub bits_per_sample: u8,
}