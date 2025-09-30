//! Errors that can occur in module operations

use serde::{Serialize, Deserialize};

/// Result type for module operations
pub type ModuleResult<T> = Result<T, ModuleError>;

/// Errors that can occur in module operations
#[derive(Debug, Serialize, Deserialize)]
pub enum ModuleError {
    /// Module not found
    NotFound(String),
    /// Module failed to load
    LoadFailed(String),
    /// Module initialization failed
    InitFailed(String),
    /// Module method call failed
    CallFailed(String),
    /// Module state serialization failed
    SerializationFailed(String),
    /// Module dependency not satisfied
    DependencyNotMet(String),
    /// Module version incompatible
    VersionMismatch(String),
    /// Module already loaded
    AlreadyLoaded(String),
    /// Module state invalid
    InvalidState(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Module not found: {}", msg),
            Self::LoadFailed(msg) => write!(f, "Module load failed: {}", msg),
            Self::InitFailed(msg) => write!(f, "Module init failed: {}", msg),
            Self::CallFailed(msg) => write!(f, "Module call failed: {}", msg),
            Self::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            Self::DependencyNotMet(msg) => write!(f, "Dependency not met: {}", msg),
            Self::VersionMismatch(msg) => write!(f, "Version mismatch: {}", msg),
            Self::AlreadyLoaded(msg) => write!(f, "Module already loaded: {}", msg),
            Self::InvalidState(msg) => write!(f, "Invalid module state: {}", msg),
        }
    }
}

impl std::error::Error for ModuleError {}