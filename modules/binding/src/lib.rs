//! View to ViewModel Binding for Direct Function Calls
//!
//! This module connects Core View APIs to System ViewModel implementations
//! at load time, enabling direct function calls with minimal overhead.

mod binding;
mod registry;
mod stats;

// Re-exports
pub use binding::Binding;
pub use registry::BindingRegistry;
pub use stats::BindingStats;