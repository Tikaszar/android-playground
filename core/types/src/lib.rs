pub mod atomic;
pub mod context;
pub mod core_error;
pub mod error;
pub mod event;
pub mod message;
pub mod networking;
pub mod once;
pub mod plugin_metadata;
pub mod render_context;
pub mod server;
pub mod shared;
pub mod stateful;

// Re-export commonly used types
pub use atomic::{Atomic, atomic};
pub use context::Context;
pub use core_error::{CoreError, CoreResult, EntityIdError, ComponentIdError};
pub use error::PluginError;
pub use event::Event;
pub use message::Message;
pub use networking::{ChannelId, Priority, Packet, ControlMessageType};
pub use once::{Once, once, once_with};
pub use plugin_metadata::{PluginId, PluginMetadata, Version};
pub use render_context::RenderContext;
pub use shared::{Handle, handle, Shared, shared};
pub use stateful::Stateful;

/// Temporary LogLevel until console system is properly set up
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
