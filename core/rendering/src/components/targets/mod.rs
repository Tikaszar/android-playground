//! Render target components - OPTIONAL (targets feature)

#[cfg(feature = "targets")]
pub mod shared;

#[cfg(feature = "targets")]
pub use shared::*;