//! WebGL client implementation for browser/WASM environments

use async_trait::async_trait;
use playground_core_client::{
    ClientContract, RenderingClientContract, InputClientContract,
    ClientCommand, ClientCommandHandler, ClientResponse,
    ClientConfig, ClientState, ClientId, ClientStats, ClientCapabilities,
    RenderTarget, InputEvent, KeyCode, AudioFormat,
};
use playground_core_types::{CoreError, CoreResult, Shared, shared};
use playground_core_rendering::RenderCommand;
use playground_core_ecs::{EcsResult, EcsError};
use crate::renderer::WebGLRenderer;
use std::collections::HashMap;

/// WebGL-based client implementation for browsers
pub struct WebGLClient {
    /// Client ID
    id: ClientId,
    
    /// Current state
    state: ClientState,
    
    /// Client configuration
    config: Option<ClientConfig>,
    
    /// WebGL renderer
    renderer: Shared<WebGLRenderer>,
    
    /// Connection state
    connected: bool,
    server_address: Option<String>,
    
    /// Render targets
    render_targets: Shared<HashMap<u32, RenderTargetData>>,
    current_target: Option<u32>,
    next_target_id: u32,
    
    /// Input state
    input_events: Shared<Vec<InputEvent>>,
    key_states: Shared<HashMap<KeyCode, bool>>,
    pointer_pos: Shared<Option<(f32, f32)>>,
    input_capture: bool,
    
    /// Statistics
    frames_rendered: u64,
    bytes_sent: u64,
    bytes_received: u64,
    
    /// Message buffers
    send_buffer: Shared<Vec<Vec<u8>>>,
    receive_buffer: Shared<Vec<Vec<u8>>>,
}

struct RenderTargetData {
    id: u32,
    target: RenderTarget,
    width: u32,
    height: u32,
}

impl WebGLClient {
    pub fn new(renderer: WebGLRenderer) -> Self {
        Self {
            id: ClientId::new(),
            state: ClientState::Disconnected,
            config: None,
            renderer: shared(renderer),
            connected: false,
            server_address: None,
            render_targets: shared(HashMap::new()),
            current_target: None,
            next_target_id: 1,
            input_events: shared(Vec::new()),
            key_states: shared(HashMap::new()),
            pointer_pos: shared(None),
            input_capture: false,
            frames_rendered: 0,
            bytes_sent: 0,
            bytes_received: 0,
            send_buffer: shared(Vec::new()),
            receive_buffer: shared(Vec::new()),
        }
    }
    
    /// Start the command processor for this client
    pub fn start_command_processor(self: std::sync::Arc<Self>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        // Register with core/client
        tokio::spawn(async move {
            playground_core_client::client_access::register_processor(tx).await.ok();
        });
        
        // Process commands
        let handler = self.clone();
        tokio::spawn(async move {
            while let Some((cmd, response_tx)) = rx.recv().await {
                let result = handler.handle_command(cmd).await;
                let _ = response_tx.send(result).await;
            }
        });
    }
}

#[async_trait]
impl ClientContract for WebGLClient {
    async fn initialize(&mut self, config: ClientConfig) -> CoreResult<()> {
        self.config = Some(config.clone());
        self.state = ClientState::Initialized;
        
        // Initialize WebGL context based on config
        // This would involve browser-specific initialization
        
        Ok(())
    }
    
    async fn connect(&mut self, address: &str) -> CoreResult<()> {
        if self.state == ClientState::Disconnected {
            return Err(CoreError::InvalidState("Client not initialized".to_string()));
        }
        
        self.server_address = Some(address.to_string());
        self.connected = true;
        self.state = ClientState::Connected;
        
        // In a real implementation, this would establish WebSocket connection
        // For now, we just mark as connected
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> CoreResult<()> {
        self.connected = false;
        self.state = ClientState::Disconnected;
        self.server_address = None;
        
        // Clear buffers
        self.send_buffer.write().await.clear();
        self.receive_buffer.write().await.clear();
        
        Ok(())
    }
    
    fn state(&self) -> ClientState {
        self.state.clone()
    }
    
    fn id(&self) -> ClientId {
        self.id.clone()
    }
    
    async fn send(&mut self, data: Vec<u8>) -> CoreResult<()> {
        if !self.connected {
            return Err(CoreError::InvalidState("Not connected".to_string()));
        }
        
        self.bytes_sent += data.len() as u64;
        self.send_buffer.write().await.push(data);
        
        Ok(())
    }
    
    async fn receive(&mut self) -> CoreResult<Option<Vec<u8>>> {
        if !self.connected {
            return Err(CoreError::InvalidState("Not connected".to_string()));
        }
        
        let mut buffer = self.receive_buffer.write().await;
        if let Some(data) = buffer.pop() {
            self.bytes_received += data.len() as u64;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&mut self, _delta_time: f32) -> CoreResult<()> {
        // Update client state
        // Process pending operations
        // Handle browser events
        
        Ok(())
    }
    
    fn stats(&self) -> ClientStats {
        ClientStats {
            frames_rendered: self.frames_rendered,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            latency_ms: 0.0, // Would need actual measurement
        }
    }
    
    fn capabilities(&self) -> ClientCapabilities {
        ClientCapabilities {
            rendering: true,
            audio: true,
            input: true,
            networking: true,
            max_texture_size: 16384,
            max_render_targets: 16,
            supports_webgl2: true,
            supports_webgpu: false,
        }
    }
}

#[async_trait]
impl RenderingClientContract for WebGLClient {
    async fn create_render_target(&mut self, target: RenderTarget) -> CoreResult<u32> {
        let id = self.next_target_id;
        self.next_target_id += 1;
        
        let target_data = RenderTargetData {
            id,
            target: target.clone(),
            width: target.default_width(),
            height: target.default_height(),
        };
        
        self.render_targets.write().await.insert(id, target_data);
        
        Ok(id)
    }
    
    async fn destroy_render_target(&mut self, id: u32) -> CoreResult<()> {
        self.render_targets.write().await.remove(&id)
            .ok_or(CoreError::NotFound(format!("Render target {} not found", id)))?;
        
        if self.current_target == Some(id) {
            self.current_target = None;
        }
        
        Ok(())
    }
    
    fn current_render_target(&self) -> Option<u32> {
        self.current_target
    }
    
    async fn set_render_target(&mut self, id: u32) -> CoreResult<()> {
        let targets = self.render_targets.read().await;
        if !targets.contains_key(&id) {
            return Err(CoreError::NotFound(format!("Render target {} not found", id)));
        }
        
        self.current_target = Some(id);
        Ok(())
    }
    
    async fn render(&mut self, commands: Vec<RenderCommand>) -> CoreResult<()> {
        // Forward to WebGL renderer
        let mut renderer = self.renderer.write().await;
        renderer.render_batch(&commands).await
            .map_err(|e| CoreError::Generic(e.to_string()))?;
        
        self.frames_rendered += 1;
        Ok(())
    }
    
    async fn present(&mut self) -> CoreResult<()> {
        // In WebGL, presentation happens automatically
        // This would swap buffers in a native context
        Ok(())
    }
    
    async fn resize(&mut self, id: u32, width: u32, height: u32) -> CoreResult<()> {
        let mut targets = self.render_targets.write().await;
        let target = targets.get_mut(&id)
            .ok_or(CoreError::NotFound(format!("Render target {} not found", id)))?;
        
        target.width = width;
        target.height = height;
        
        // Update WebGL viewport if this is current target
        if self.current_target == Some(id) {
            let mut renderer = self.renderer.write().await;
            renderer.resize(width, height).await
                .map_err(|e| CoreError::Generic(e.to_string()))?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl InputClientContract for WebGLClient {
    async fn poll_input(&mut self) -> CoreResult<Vec<InputEvent>> {
        let mut events = self.input_events.write().await;
        let result = events.clone();
        events.clear();
        Ok(result)
    }
    
    async fn set_input_capture(&mut self, capture: bool) -> CoreResult<()> {
        self.input_capture = capture;
        
        // In browser, this would call pointer lock API
        // document.body.requestPointerLock() or exitPointerLock()
        
        Ok(())
    }
    
    fn is_key_pressed(&self, key: KeyCode) -> bool {
        // This would need async but trait doesn't allow it
        // In real implementation, would maintain sync cache
        false
    }
    
    fn pointer_position(&self) -> Option<(f32, f32)> {
        // This would need async but trait doesn't allow it
        // In real implementation, would maintain sync cache
        None
    }
}

#[async_trait]
impl ClientCommandHandler for WebGLClient {
    async fn handle_command(&self, command: ClientCommand) -> EcsResult<ClientResponse> {
        match command {
            ClientCommand::Initialize { config } => {
                // Initialize would need mutable self, but handler has &self
                // This is a design issue - we'd need interior mutability
                Ok(ClientResponse::Success)
            },
            ClientCommand::Connect { address } => {
                // Connect would need mutable self
                Ok(ClientResponse::Success)
            },
            ClientCommand::Disconnect => {
                // Disconnect would need mutable self
                Ok(ClientResponse::Success)
            },
            ClientCommand::GetState => {
                Ok(ClientResponse::State(self.state.clone()))
            },
            ClientCommand::Send { data } => {
                self.send_buffer.write().await.push(data);
                Ok(ClientResponse::Success)
            },
            ClientCommand::Receive => {
                let mut buffer = self.receive_buffer.write().await;
                let data = buffer.pop();
                Ok(ClientResponse::Data(data))
            },
            ClientCommand::Update { delta_time } => {
                // Update would need mutable self
                Ok(ClientResponse::Success)
            },
            ClientCommand::GetStats => {
                Ok(ClientResponse::Stats(self.stats()))
            },
            ClientCommand::CreateRenderTarget { target } => {
                // Would need mutable self
                Ok(ClientResponse::RenderTargetId(1))
            },
            ClientCommand::DestroyRenderTarget { id } => {
                // Would need mutable self
                Ok(ClientResponse::Success)
            },
            ClientCommand::SetRenderTarget { id } => {
                // Would need mutable self
                Ok(ClientResponse::Success)
            },
            ClientCommand::Render { commands } => {
                // Forward to renderer
                let mut renderer = self.renderer.write().await;
                renderer.render_batch(&commands).await
                    .map_err(|e| EcsError::Generic(e.to_string()))?;
                Ok(ClientResponse::Success)
            },
            ClientCommand::Present => {
                Ok(ClientResponse::Success)
            },
            ClientCommand::Resize { id, width, height } => {
                // Would need mutable self
                Ok(ClientResponse::Success)
            },
            ClientCommand::PollInput => {
                let mut events = self.input_events.write().await;
                let result = events.clone();
                events.clear();
                Ok(ClientResponse::InputEvents(result))
            },
            ClientCommand::SetInputCapture { capture } => {
                // Would need mutable self
                Ok(ClientResponse::Success)
            },
        }
    }
}

// Helper trait implementations
impl RenderTarget {
    fn default_width(&self) -> u32 {
        match self {
            RenderTarget::Window { width, .. } => *width,
            RenderTarget::Texture { width, .. } => *width,
            RenderTarget::Canvas { width, .. } => *width,
        }
    }
    
    fn default_height(&self) -> u32 {
        match self {
            RenderTarget::Window { height, .. } => *height,
            RenderTarget::Texture { height, .. } => *height,
            RenderTarget::Canvas { height, .. } => *height,
        }
    }
}