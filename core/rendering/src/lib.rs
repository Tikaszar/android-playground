//! Core rendering data structures and types
//!
//! This package defines ONLY data structures - NO LOGIC!
//! All rendering implementation logic lives in systems packages like
//! systems/webgl (browser) or systems/vulkan (future native).
//!
//! This follows the "abstract base class" pattern where core defines
//! structure and systems provide behavior.

// Data structure modules
pub mod renderer;
pub mod types;
pub mod components;
pub mod resources;
pub mod commands;
pub mod operations;
pub mod api;
pub mod error;

// Re-export the main Renderer struct
pub use renderer::Renderer;

// Re-export type aliases and data structures
pub use types::{
    Float, Double, Int, UInt, Index, Byte, ResourceId,
    Vec2, Vec3, Vec4, Quat, Mat2, Mat3, Mat4,
    ColorRGB, ColorRGBA, ColorRGB8, ColorRGBA8,
    Viewport, Rect, BoundingBox, BoundingSphere, Color,
    RendererCapabilities, RendererConfig, RendererStats,
    PowerPreference,
};

// Re-export error types
pub use error::{RenderError, RenderResult};

// Re-export ALL components
pub use components::*;

// Re-export command types
pub use commands::RenderCommand;

#[cfg(feature = "commands")]
pub use commands::{CommandBufferInfo, CommandBufferState};

// Re-export API functions
pub use api::{
    get_renderer_instance,
    initialize_renderer,
    shutdown_renderer,
    get_capabilities,
    get_stats,
    switch_backend,
    submit_frame,
    present,
};

#[cfg(feature = "commands")]
pub use api::{
    create_command_buffer,
    begin_recording,
    end_recording,
    submit_commands,
};

#[cfg(feature = "targets")]
pub use api::{
    create_render_target,
    destroy_render_target,
    set_render_target,
    resize_render_target,
};

#[cfg(feature = "shaders")]
pub use api::{
    create_shader,
    destroy_shader,
};

#[cfg(feature = "textures")]
pub use api::{
    create_texture,
    update_texture,
    destroy_texture,
};

#[cfg(feature = "buffers")]
pub use api::{
    create_buffer,
    update_buffer,
    destroy_buffer,
};

#[cfg(feature = "pipelines")]
pub use api::{
    create_pipeline,
    destroy_pipeline,
};

#[cfg(feature = "passes")]
pub use api::{
    create_render_pass,
    destroy_render_pass,
};

// High-level helpers
#[cfg(all(feature = "core-2d", feature = "textures"))]
pub use api::draw_sprite;

#[cfg(all(feature = "core-3d", feature = "buffers"))]
pub use api::draw_model;