//! The main module interface exposed by all hot-loadable modules

use crate::{ModuleMetadata, ModuleVTable};

/// The main module interface exposed by all hot-loadable modules
/// Every module must export a static PLAYGROUND_MODULE symbol
pub struct Module {
    /// Module metadata (name, version, dependencies)
    pub metadata: &'static ModuleMetadata,
    /// Function pointer table for module operations
    pub vtable: &'static ModuleVTable,
}