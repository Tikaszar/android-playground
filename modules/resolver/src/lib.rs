//! Cargo.toml Module Declaration Resolver
//!
//! This module reads module declarations from Cargo.toml files
//! and resolves which Systems implement which Core modules.

mod config;
mod resolver;

// Re-exports
pub use config::{AppModuleConfig, ModuleDeclaration, SystemProvides};
pub use resolver::ModuleResolver;