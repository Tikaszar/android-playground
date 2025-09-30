//! Runtime Module Registry and Orchestration
//!
//! This module coordinates all the MVVM components at runtime,
//! managing module loading, binding, and hot-reload.

mod info;
mod registry;
mod stats;

// Re-exports
pub use info::{ModuleInfo, ModuleState};
pub use registry::ModuleRegistry;
pub use stats::RegistryStats;