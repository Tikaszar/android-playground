//! Buffer components - OPTIONAL (buffers feature)

#[cfg(feature = "buffers")]
pub mod shared;

#[cfg(feature = "buffers")]
pub use shared::*;