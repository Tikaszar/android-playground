//! Registry statistics

use playground_modules_binding::BindingStats;

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_modules: usize,
    pub loaded_modules: usize,
    pub bound_modules: usize,
    pub failed_modules: usize,
    pub binding_stats: BindingStats,
}