//! Client API module for systems/logic
//! 
//! This provides the public API for client operations, hiding the implementation
//! details of how clients are managed. Apps and plugins should use these functions
//! instead of directly accessing core/client or systems/webgl.

use playground_core_client::{
    ClientConfig, ClientState, ClientId, ClientStats, RenderTarget, InputEvent
};
use playground_core_rendering::RenderCommand;
use playground_core_ecs::EcsResult;

/// Initialize a client with the given configuration
pub async fn initialize_client(config: ClientConfig) -> EcsResult<()> {
    playground_core_client::client_access::initialize(config).await
}

/// Connect the client to a server
pub async fn connect_client(address: &str) -> EcsResult<()> {
    playground_core_client::client_access::connect(address).await
}

/// Disconnect the client from the server
pub async fn disconnect_client() -> EcsResult<()> {
    playground_core_client::client_access::disconnect().await
}

/// Get the current client state
pub async fn get_client_state() -> EcsResult<ClientState> {
    playground_core_client::client_access::get_state().await
}

/// Submit render commands to the client
pub async fn render_to_client(commands: Vec<RenderCommand>) -> EcsResult<()> {
    playground_core_client::client_access::render(commands).await
}

/// Poll for input events from the client
pub async fn poll_client_input() -> EcsResult<Vec<InputEvent>> {
    playground_core_client::client_access::poll_input().await
}

/// Client management functions for generic client operations
pub mod client_management {
    use super::*;
    
    /// Send data to the server through the client
    pub async fn send_to_server(data: Vec<u8>) -> EcsResult<()> {
        // This would use the client command processor
        // For now, return success
        Ok(())
    }
    
    /// Receive data from the server through the client
    pub async fn receive_from_server() -> EcsResult<Option<Vec<u8>>> {
        // This would use the client command processor
        // For now, return None
        Ok(None)
    }
    
    /// Update the client (called each frame)
    pub async fn update_client(delta_time: f32) -> EcsResult<()> {
        // This would send an update command to the client
        Ok(())
    }
    
    /// Get client statistics
    pub async fn get_client_stats() -> EcsResult<ClientStats> {
        // This would query the client for stats
        Ok(ClientStats {
            frames_rendered: 0,
            bytes_sent: 0,
            bytes_received: 0,
            latency_ms: 0.0,
        })
    }
}

/// Rendering-specific client operations
pub mod rendering {
    use super::*;
    
    /// Create a render target
    pub async fn create_render_target(target: RenderTarget) -> EcsResult<u32> {
        // This would use the client command processor
        Ok(1)
    }
    
    /// Destroy a render target
    pub async fn destroy_render_target(id: u32) -> EcsResult<()> {
        // This would use the client command processor
        Ok(())
    }
    
    /// Set the current render target
    pub async fn set_render_target(id: u32) -> EcsResult<()> {
        // This would use the client command processor
        Ok(())
    }
    
    /// Resize a render target
    pub async fn resize_render_target(id: u32, width: u32, height: u32) -> EcsResult<()> {
        // This would use the client command processor
        Ok(())
    }
    
    /// Present the rendered frame
    pub async fn present_frame() -> EcsResult<()> {
        // This would use the client command processor
        Ok(())
    }
}

/// Input-specific client operations
pub mod input {
    use super::*;
    use playground_core_client::KeyCode;
    
    /// Set input capture mode (e.g., pointer lock)
    pub async fn set_input_capture(capture: bool) -> EcsResult<()> {
        // This would use the client command processor
        Ok(())
    }
    
    /// Check if a key is currently pressed
    pub async fn is_key_pressed(key: KeyCode) -> EcsResult<bool> {
        // This would query the client
        Ok(false)
    }
    
    /// Get current pointer position
    pub async fn get_pointer_position() -> EcsResult<Option<(f32, f32)>> {
        // This would query the client
        Ok(None)
    }
}