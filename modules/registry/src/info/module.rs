//! Information about a registered module

use super::state::ModuleState;
use playground_modules_types::ModuleType;
use std::path::PathBuf;

/// Information about a registered module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub module_type: ModuleType,
    pub state: ModuleState,
    pub path: Option<PathBuf>,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}