//! View to ViewModel Binding for Direct Access
//!
//! This module connects Core View APIs to System ViewModel implementations
//! at load time, enabling direct access with minimal overhead.
//!
//! Also manages Model pools with sharding and object recycling.

mod pool;
mod registry;
mod stats;

// Re-exports
pub use pool::ModelPool;
pub use registry::BindingRegistry;
pub use stats::BindingStats;
