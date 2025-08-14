pub mod loader;
pub mod r#trait;

pub use loader::PluginLoader;
pub use r#trait::{CreatePluginFn, Plugin};

// Re-export Stateful from types for convenience
pub use playground_types::Stateful;