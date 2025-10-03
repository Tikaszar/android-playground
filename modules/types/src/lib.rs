//! MVVM Base Types for Module System
//!
//! This crate defines the core types for the MVVM-based module architecture:
//! - Model: Data structures only
//! - View: API contracts only
//! - ViewModel: Implementation logic only

pub mod error;
pub mod metadata;
pub mod model;
pub mod module;
pub mod view;
pub mod viewmodel;

// Thread-safe types
pub mod handle;
pub mod shared;
pub mod atomic;
pub mod once;

// Re-exports
pub use error::{ModuleError, ModuleResult};
pub use metadata::ModuleMetadata;
pub use model::{ModelData, ModelId, ModelTrait, ModelType, ModelTypeInfo, model_type_of};
pub use module::{Module, ModuleDependency, ModuleLifecycle, ModuleType};
pub use view::{ViewId, ViewTrait};
pub use viewmodel::{ViewModelTrait};

// Thread-safe type re-exports
pub use handle::{Handle, handle};
pub use shared::{Shared, shared};
pub use atomic::{Atomic, atomic, Ordering};
pub use once::{Once, once, once_with};

// Re-export common atomic types from std
pub use std::sync::atomic::{AtomicU64, AtomicU32, AtomicBool};
