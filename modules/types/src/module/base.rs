//! Core module interface for MVVM architecture

use crate::metadata::ModuleMetadata;
use super::lifecycle::ModuleLifecycle;
use super::r#type::ModuleType;

/// The main module interface exposed by all modules
pub struct Module {
    /// Module metadata (name, version, dependencies)
    pub metadata: &'static ModuleMetadata,

    /// Module type determines what it provides
    pub module_type: ModuleType,

    /// Functions for module lifecycle
    pub lifecycle: ModuleLifecycle,
}