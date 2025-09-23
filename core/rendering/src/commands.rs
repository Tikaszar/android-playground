//! Render command definitions

use serde::{Serialize, Deserialize};
use playground_core_ecs::{Entity, EntityRef};
use crate::types::*;

/// Render commands that can be submitted to the renderer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderCommand {
    // Frame control
    BeginFrame {
        camera: EntityRef,
    },

    EndFrame,

    Present,

    // Clear operations
    Clear {
        color: Option<ColorRGBA>,
        depth: Option<Float>,
        stencil: Option<UInt>,
    },

    // Entity-based rendering
    RenderEntity {
        entity: EntityRef,
    },

    RenderEntities {
        entities: Vec<EntityRef>,
    },

    RenderLayer {
        layer: UInt,
    },

    RenderAllEntities,

    // State changes
    SetViewport {
        viewport: Viewport,
    },

    #[cfg(feature = "targets")]
    SetRenderTarget {
        target: Option<ResourceId>,
    },

    // Resource operations
    #[cfg(feature = "textures")]
    LoadTexture {
        entity: EntityRef,
        data: Vec<u8>,
    },

    #[cfg(feature = "buffers")]
    LoadMesh {
        entity: EntityRef,
        vertices: Vec<u8>,
        indices: Vec<u8>,
    },

    #[cfg(feature = "shaders")]
    CompileShader {
        entity: EntityRef,
        source: String,
        shader_type: ShaderStage,
    },

    // Pipeline state (for custom rendering)
    #[cfg(feature = "pipelines")]
    SetPipeline {
        pipeline: ResourceId,
    },

    #[cfg(feature = "pipelines")]
    SetBlendMode {
        mode: BlendMode,
    },

    #[cfg(feature = "pipelines")]
    SetDepthTest {
        enabled: bool,
    },

    #[cfg(feature = "pipelines")]
    SetCullMode {
        mode: CullMode,
    },

    // Batching hints
    #[cfg(feature = "batching")]
    BeginBatch {
        batch_type: BatchType,
    },

    #[cfg(feature = "batching")]
    EndBatch,

    #[cfg(feature = "batching")]
    FlushBatches,

    // Debug rendering
    #[cfg(feature = "debug")]
    DebugDrawLine {
        start: Vec3,
        end: Vec3,
        color: ColorRGBA,
    },

    #[cfg(feature = "debug")]
    DebugDrawBox {
        bounds: BoundingBox,
        color: ColorRGBA,
    },

    #[cfg(feature = "debug")]
    DebugDrawSphere {
        center: Vec3,
        radius: Float,
        color: ColorRGBA,
    },
}

// Supporting types
#[cfg(feature = "shaders")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum BlendMode {
    None,
    Alpha,
    Additive,
    Multiply,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[cfg(feature = "batching")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum BatchType {
    Opaque,
    Transparent,
    UI,
}

// Command buffer support
#[cfg(feature = "commands")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandBufferInfo {
    pub id: ResourceId,
    pub state: CommandBufferState,
    pub level: CommandBufferLevel,
}

#[cfg(feature = "commands")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CommandBufferState {
    Initial,
    Recording,
    Executable,
    Invalid,
}

#[cfg(feature = "commands")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CommandBufferLevel {
    Primary,
    Secondary,
}