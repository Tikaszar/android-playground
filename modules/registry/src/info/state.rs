//! Module lifecycle state

/// Module lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Not loaded yet
    Unloaded,

    /// Currently loading
    Loading,

    /// Loaded and initialized
    Loaded,

    /// Bound to View/ViewModel
    Bound,

    /// Being hot-reloaded
    Reloading,

    /// Failed to load
    Failed,
}