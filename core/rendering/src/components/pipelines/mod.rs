//! Pipelines components - OPTIONAL (pipelines feature)

#[cfg(feature = "pipelines")]
pub mod shared;

#[cfg(feature = "pipelines")]
pub use shared::*;
