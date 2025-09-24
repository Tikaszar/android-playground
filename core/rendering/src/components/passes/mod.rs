//! Passes components - OPTIONAL (passes feature)

#[cfg(feature = "passes")]
pub mod shared;

#[cfg(feature = "passes")]
pub use shared::*;
