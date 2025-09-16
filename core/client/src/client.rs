//! Concrete Client data structure - NO LOGIC, just data fields!
//! 
//! This is like an abstract base class - defines structure only.
//! All actual implementation logic is in systems/webgl (browser) or
//! systems/native (future native client).

use std::collections::HashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_ecs::VTable;
use crate::types::*;
use crate::input::InputState;

/// The concrete Client struct - data fields only, no logic!
/// 
/// Like an abstract base class in OOP - structure but no behavior.
/// All actual client operations are implemented in systems packages.
pub struct Client {
    /// The VTable for system dispatch
    pub vtable: VTable,
    
    /// Client ID
    pub id: ClientId,
    
    /// Client state
    pub state: Shared<ClientState>,
    
    /// Client statistics
    pub stats: Shared<ClientStats>,
    
    /// Client configuration
    pub config: Shared<ClientConfig>,
    
    /// Connection address
    pub server_address: Shared<Option<String>>,
    
    /// Render targets (for rendering-capable clients)
    #[cfg(feature = "rendering")]
    pub render_targets: Shared<HashMap<u32, RenderTarget>>,
    
    /// Current render target ID
    #[cfg(feature = "rendering")]
    pub current_render_target: Shared<Option<u32>>,
    
    /// Input state (for input-capable clients)
    #[cfg(feature = "input")]
    pub input_state: Shared<InputState>,
    
    /// Audio volume (for audio-capable clients)
    #[cfg(feature = "audio")]
    pub audio_volume: Shared<f32>,
    
    /// Active audio tracks
    #[cfg(feature = "audio")]
    pub audio_tracks: Shared<HashMap<u32, AudioTrackInfo>>,
    
    /// Client capabilities
    pub capabilities: ClientCapabilities,
}

/// Information about an active audio track
#[cfg(feature = "audio")]
#[derive(Debug, Clone)]
pub struct AudioTrackInfo {
    pub id: u32,
    pub format: AudioFormat,
    pub duration: Option<f32>,
    pub is_playing: bool,
}

/// Audio format information
#[cfg(feature = "audio")]
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u8,
    pub bits_per_sample: u8,
}

impl Client {
    /// Create a new Client instance - just data initialization, no logic!
    pub fn new(id: ClientId) -> Handle<Self> {
        handle(Self {
            vtable: VTable::new(),
            id,
            state: shared(ClientState::Disconnected),
            stats: shared(ClientStats::default()),
            config: shared(ClientConfig::default()),
            server_address: shared(None),
            
            #[cfg(feature = "rendering")]
            render_targets: shared(HashMap::new()),
            
            #[cfg(feature = "rendering")]
            current_render_target: shared(None),
            
            #[cfg(feature = "input")]
            input_state: shared(InputState::default()),
            
            #[cfg(feature = "audio")]
            audio_volume: shared(1.0),
            
            #[cfg(feature = "audio")]
            audio_tracks: shared(HashMap::new()),
            
            capabilities: ClientCapabilities::default(),
        })
    }
}