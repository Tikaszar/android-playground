//! Pure Rust Module Interface for Hot-Loading
//!
//! This crate defines the module interface for the entire Playground engine.
//! ALL modules (Core, Systems, Plugins, Apps) expose this interface for hot-loading.
//!
//! Key Features:
//! - Pure Rust (no extern "C" or repr(C))
//! - Direct function pointers for performance (1-5ns overhead)
//! - State preservation across reloads
//! - Semantic versioning for dependencies

// Module definitions
pub mod module;
pub mod metadata;
pub mod module_type;
pub mod dependency;
pub mod vtable;
pub mod state;
pub mod error;
pub mod call;
pub mod macros;
pub mod noop;

// Re-exports
pub use module::Module;
pub use metadata::ModuleMetadata;
pub use module_type::ModuleType;
pub use dependency::Dependency;
pub use vtable::ModuleVTable;
pub use state::ModuleState;
pub use error::{ModuleError, ModuleResult};
pub use call::{ModuleCall, ModuleResponse};
pub use noop::NOOP_VTABLE;