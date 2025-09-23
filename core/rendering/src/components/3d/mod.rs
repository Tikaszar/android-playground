//! 3D-specific rendering components

#[cfg(feature = "core-3d")]
pub mod transform3d;

#[cfg(all(feature = "core-3d", feature = "buffers"))]
pub mod mesh;

#[cfg(feature = "core-3d")]
pub mod mesh_renderer;

#[cfg(feature = "core-3d")]
pub mod camera3d;

// Core 3D components
#[cfg(feature = "core-3d")]
pub use transform3d::Transform3D;

#[cfg(feature = "core-3d")]
pub use camera3d::{Camera3D, ProjectionType};

#[cfg(feature = "core-3d")]
pub use mesh_renderer::MeshRenderer;

// Buffer-dependent components
#[cfg(all(feature = "core-3d", feature = "buffers"))]
pub use mesh::Mesh;