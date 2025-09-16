//! Core console package - data structures and API only, NO LOGIC!
//! 
//! This package defines console contracts that can be implemented by any backend.
//! All actual implementation logic is in systems/console.

// Module declarations
pub mod console;
pub mod types;
pub mod api;

#[cfg(feature = "output")]
pub mod output;

#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "progress")]
pub mod progress;

#[cfg(feature = "input")]
pub mod input;

#[cfg(feature = "input")]
pub mod input_api;

// Re-exports - NO implementation here, just exports!
pub use console::Console;
pub use types::*;

// Feature-gated re-exports
#[cfg(feature = "input")]
pub use input::*;

// Public API re-exports
pub use api::*;