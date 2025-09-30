//! Pure Rust function pointer table (no extern "C"!)

/// Pure Rust function pointer table (no extern "C"!)
pub struct ModuleVTable {
    /// Create a new instance of the module
    pub create: fn() -> *mut u8,
    /// Destroy a module instance
    pub destroy: fn(*mut u8),
    /// Initialize module with configuration
    pub initialize: fn(*mut u8, config: &[u8]) -> Result<(), String>,
    /// Shutdown module cleanly
    pub shutdown: fn(*mut u8) -> Result<(), String>,
    /// Call a method on the module
    pub call: fn(*mut u8, method: &str, args: &[u8]) -> Result<Vec<u8>, String>,
    /// Save module state for hot-reload
    pub save_state: fn(*const u8) -> Vec<u8>,
    /// Restore module state after hot-reload
    pub restore_state: fn(*mut u8, state: &[u8]) -> Result<(), String>,
    /// Get module capabilities/methods
    pub get_capabilities: fn() -> Vec<&'static str>,
}