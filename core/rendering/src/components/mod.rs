//! All rendering components for the ECS system
//!
//! Components are organized by category:
//! - shared: Components used by both 2D and 3D
//! - r2d: 2D-specific components (feature-gated)
//! - r3d: 3D-specific components (feature-gated)

pub mod shared;

#[path = "2d/mod.rs"]
mod r2d;

#[path = "3d/mod.rs"]
mod r3d;

// Re-export shared components at root level
pub use shared::*;

// Re-export 2D components
#[cfg(feature = "core-2d")]
pub use r2d::*;

// Re-export 3D components
#[cfg(feature = "core-3d")]
pub use r3d::*;