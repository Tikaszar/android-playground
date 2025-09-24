//! All rendering components for the ECS system
//!
//! Components are organized by category:
//! - shared: MANDATORY components used by both 2D and 3D
//! - r2d: 2D-specific components (feature-gated with core-2d)
//! - r3d: 3D-specific components (feature-gated with core-3d)
//! - Feature directories: Optional components gated by features

// MANDATORY components
pub mod shared;

// Core rendering modes
#[cfg(feature = "core-2d")]
#[path = "2d/mod.rs"]
mod r2d;

#[cfg(feature = "core-3d")]
#[path = "3d/mod.rs"]
mod r3d;

// OPTIONAL feature-gated component modules
#[cfg(feature = "targets")]
pub mod targets;

#[cfg(feature = "shaders")]
pub mod shaders;

#[cfg(feature = "textures")]
pub mod textures;

#[cfg(feature = "buffers")]
pub mod buffers;

#[cfg(feature = "uniforms")]
pub mod uniforms;

#[cfg(feature = "samplers")]
pub mod samplers;

#[cfg(feature = "pipelines")]
pub mod pipelines;

#[cfg(feature = "commands")]
pub mod commands;

#[cfg(feature = "passes")]
pub mod passes;

// Re-export MANDATORY shared components at root level
pub use shared::*;

// Re-export 2D components
#[cfg(feature = "core-2d")]
pub use r2d::*;

// Re-export 3D components
#[cfg(feature = "core-3d")]
pub use r3d::*;

// Re-export feature-gated components
#[cfg(feature = "targets")]
pub use targets::*;

#[cfg(feature = "shaders")]
pub use shaders::*;

#[cfg(feature = "textures")]
pub use textures::*;

#[cfg(feature = "buffers")]
pub use buffers::*;

#[cfg(feature = "uniforms")]
pub use uniforms::*;

#[cfg(feature = "samplers")]
pub use samplers::*;

#[cfg(feature = "pipelines")]
pub use pipelines::*;

#[cfg(feature = "commands")]
pub use commands::*;

#[cfg(feature = "passes")]
pub use passes::*;