//! Statistics about binding state

/// Statistics about binding state
#[derive(Debug, Clone)]
pub struct BindingStats {
    pub active_bindings: usize,
    pub pending_views: usize,
    pub pending_viewmodels: usize,
}