pub mod context;
pub mod core_error;
pub mod error;
pub mod event;
pub mod message;
pub mod networking;
pub mod plugin_metadata;
pub mod render_context;
pub mod server;
pub mod shared;
pub mod stateful;

// Re-export commonly used types
pub use context::Context;
pub use core_error::{CoreError, CoreResult};
pub use error::PluginError;
pub use event::Event;
pub use message::Message;
pub use networking::{ChannelId, Priority, Packet, ControlMessageType};
pub use plugin_metadata::{PluginId, PluginMetadata, Version};
pub use render_context::RenderContext;
pub use shared::{Handle, handle, Shared, shared};
pub use stateful::Stateful;
