#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),
}
