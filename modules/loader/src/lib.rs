//! Module Loader with THE Single Unsafe Block
//!
//! This module contains the ONLY unsafe code in the entire codebase.
//! All unsafe operations are contained in a single block when loading modules.

mod loaded_module;
mod loader;

// Re-exports
pub use loaded_module::LoadedModule;
pub use loader::ModuleLoader;