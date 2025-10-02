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
pub mod types;
pub mod view;
pub mod viewmodel;

// Re-exports
pub use error::{ModuleError, ModuleResult};
pub use metadata::ModuleMetadata;
pub use model::{ModelData, ModelId, ModelTrait, ModelType, ModelTypeInfo};
pub use module::{Module, ModuleDependency, ModuleLifecycle, ModuleType};
pub use types::{Handle, Shared};
pub use view::{ViewId, ViewTrait};
pub use viewmodel::ViewModelTrait;
