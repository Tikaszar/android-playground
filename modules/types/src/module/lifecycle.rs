//! Module lifecycle function pointers

/// Pure Rust function pointers for module lifecycle
pub struct ModuleLifecycle {
    /// Initialize module with configuration
    pub initialize: fn(config: &[u8]) -> Result<(), String>,

    /// Shutdown module cleanly
    pub shutdown: fn() -> Result<(), String>,

    /// Save module state for hot-reload
    pub save_state: fn() -> Vec<u8>,

    /// Restore module state after hot-reload
    pub restore_state: fn(state: &[u8]) -> Result<(), String>,
}