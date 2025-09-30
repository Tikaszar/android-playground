//! Loaded module structure

use playground_modules_types::{ModuleMetadata, ModuleType, ViewAPI, ViewModelImpl};
use libloading::Library;
use std::path::PathBuf;

/// A loaded module with its library handle and metadata
pub struct LoadedModule {
    /// The dynamic library handle (kept alive)
    pub _library: Library,

    /// Module metadata
    pub metadata: ModuleMetadata,

    /// Module type (Core, System, Plugin, App)
    pub module_type: ModuleType,

    /// Path to the module file
    pub path: PathBuf,

    /// For Core modules: View APIs
    pub view_api: Option<ViewAPI>,

    /// For System modules: ViewModel implementations
    pub viewmodel_impl: Option<ViewModelImpl>,
}