//! Dependency on another module

use serde::{Deserialize, Serialize};

/// Dependency on another module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    /// Name of the dependency
    pub name: &'static str,

    /// Version requirement (semver)
    pub version_req: &'static str,

    /// Required features from this dependency
    pub features: &'static [&'static str],
}