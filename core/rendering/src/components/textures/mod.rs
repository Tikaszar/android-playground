//! Texture components - OPTIONAL (textures feature)

#[cfg(feature = "textures")]
pub mod shared;

#[cfg(feature = "textures")]
pub use shared::*;