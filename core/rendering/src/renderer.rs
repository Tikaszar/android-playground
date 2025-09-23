//! Concrete Renderer data structure - NO LOGIC, just data fields!
//!
//! This is like an abstract base class - defines structure only.
//! All actual implementation logic is in systems/webgl (browser) or
//! systems/vulkan (future native).

use std::collections::HashMap;
use playground_core_types::{Handle, handle, Shared, shared};
use playground_core_ecs::VTable;
use crate::types::*;
use crate::resources::*;

/// The concrete Renderer struct - data fields only, no logic!
///
/// Like an abstract base class in OOP - structure but no behavior.
/// All actual rendering operations are implemented in systems packages.
pub struct Renderer {
    /// The VTable for system dispatch
    pub vtable: VTable,

    /// Renderer capabilities reported by the backend
    pub capabilities: Shared<RendererCapabilities>,

    /// Renderer configuration
    pub config: Shared<RendererConfig>,

    /// Renderer statistics
    pub stats: Shared<RendererStats>,

    /// Whether the renderer is initialized
    pub is_initialized: Shared<bool>,

    /// Active backend name ("webgl", "vulkan", etc.)
    pub active_backend: Shared<String>,

    // Core rendering state
    #[cfg(feature = "targets")]
    pub render_targets: Shared<HashMap<ResourceId, RenderTargetInfo>>,

    #[cfg(feature = "targets")]
    pub current_target: Shared<Option<ResourceId>>,

    // Resource management
    #[cfg(feature = "shaders")]
    pub shaders: Shared<HashMap<ResourceId, ShaderInfo>>,

    #[cfg(feature = "textures")]
    pub textures: Shared<HashMap<ResourceId, TextureInfo>>,

    #[cfg(feature = "buffers")]
    pub buffers: Shared<HashMap<ResourceId, BufferInfo>>,

    #[cfg(feature = "uniforms")]
    pub uniform_buffers: Shared<HashMap<ResourceId, UniformBufferInfo>>,

    #[cfg(feature = "samplers")]
    pub samplers: Shared<HashMap<ResourceId, SamplerInfo>>,

    // Pipeline state
    #[cfg(feature = "pipelines")]
    pub pipelines: Shared<HashMap<ResourceId, PipelineInfo>>,

    #[cfg(feature = "pipelines")]
    pub current_pipeline: Shared<Option<ResourceId>>,

    // Render passes
    #[cfg(feature = "passes")]
    pub render_passes: Shared<HashMap<ResourceId, RenderPassInfo>>,

    #[cfg(feature = "passes")]
    pub active_pass: Shared<Option<ResourceId>>,

    // Command recording
    #[cfg(feature = "commands")]
    pub command_buffers: Shared<HashMap<ResourceId, CommandBufferInfo>>,

    #[cfg(feature = "commands")]
    pub recording_buffer: Shared<Option<ResourceId>>,

    /// Next available resource ID
    next_resource_id: Shared<ResourceId>,
}

impl Renderer {
    /// Create a new Renderer instance - just data initialization, no logic!
    pub fn new() -> Handle<Self> {
        handle(Self {
            vtable: VTable::new(),
            capabilities: shared(RendererCapabilities::default()),
            config: shared(RendererConfig::default()),
            stats: shared(RendererStats::default()),
            is_initialized: shared(false),
            active_backend: shared(String::new()),

            #[cfg(feature = "targets")]
            render_targets: shared(HashMap::new()),

            #[cfg(feature = "targets")]
            current_target: shared(None),

            #[cfg(feature = "shaders")]
            shaders: shared(HashMap::new()),

            #[cfg(feature = "textures")]
            textures: shared(HashMap::new()),

            #[cfg(feature = "buffers")]
            buffers: shared(HashMap::new()),

            #[cfg(feature = "uniforms")]
            uniform_buffers: shared(HashMap::new()),

            #[cfg(feature = "samplers")]
            samplers: shared(HashMap::new()),

            #[cfg(feature = "pipelines")]
            pipelines: shared(HashMap::new()),

            #[cfg(feature = "pipelines")]
            current_pipeline: shared(None),

            #[cfg(feature = "passes")]
            render_passes: shared(HashMap::new()),

            #[cfg(feature = "passes")]
            active_pass: shared(None),

            #[cfg(feature = "commands")]
            command_buffers: shared(HashMap::new()),

            #[cfg(feature = "commands")]
            recording_buffer: shared(None),

            next_resource_id: shared(1),
        })
    }

    /// Generate a new resource ID
    pub async fn generate_resource_id(&self) -> ResourceId {
        let mut next_id = self.next_resource_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    }
}