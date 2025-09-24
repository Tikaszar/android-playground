//! Samplers components - OPTIONAL (samplers feature)

#[cfg(feature = "samplers")]
pub mod shared;

#[cfg(feature = "samplers")]
pub use shared::*;
