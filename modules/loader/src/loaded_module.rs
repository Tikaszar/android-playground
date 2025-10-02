//! Loaded module structure

use playground_modules_types::{
    Handle, ModelTypeInfo, ModuleMetadata, ModuleType, ViewModelTrait, ViewTrait,
};
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

    /// For Core modules: View trait object
    pub view: Option<Handle<dyn ViewTrait>>,

    /// For Core modules: Model type information for pool initialization
    pub models: Option<&'static [ModelTypeInfo]>,

    /// For System modules: ViewModel trait object
    pub viewmodel: Option<Handle<dyn ViewModelTrait>>,
}
