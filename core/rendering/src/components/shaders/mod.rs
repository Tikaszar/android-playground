//! Shader components - OPTIONAL (shaders feature)

#[cfg(feature = "shaders")]
pub mod shared;

#[cfg(feature = "shaders")]
pub use shared::*;