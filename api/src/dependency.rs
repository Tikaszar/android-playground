//! Module dependency declaration

/// Module dependency declaration
pub struct Dependency {
    /// Name of the required module
    pub name: &'static str,
    /// Version requirement (e.g., "^1.0", ">=1.0, <2.0")
    pub version_req: &'static str,
    /// Required features from the module
    pub features: &'static [&'static str],
}