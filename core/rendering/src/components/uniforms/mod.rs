//! Uniforms components - OPTIONAL (uniforms feature)

#[cfg(feature = "uniforms")]
pub mod shared;

#[cfg(feature = "uniforms")]
pub use shared::*;
