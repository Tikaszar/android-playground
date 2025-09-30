//! Module types and interfaces

mod base;
pub mod dependency;  // Make public for metadata.rs
mod lifecycle;
mod r#type;

pub use base::Module;
pub use dependency::ModuleDependency;
pub use lifecycle::ModuleLifecycle;
pub use r#type::ModuleType;