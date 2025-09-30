//! Dependency on another module

/// Dependency on another module
#[derive(Debug, Clone)]
pub struct ModuleDependency {
    /// Name of the dependency
    pub name: &'static str,

    /// Version requirement (semver)
    pub version_req: &'static str,

    /// Required features from this dependency
    pub features: &'static [&'static str],
}