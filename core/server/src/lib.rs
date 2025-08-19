pub mod channel;
pub mod packet;
pub mod batcher;
pub mod websocket;
pub mod handlers;
pub mod mcp;

pub use channel::{ChannelManager, ChannelInfo};
pub use packet::{Packet, Priority};
pub use batcher::FrameBatcher;
pub use websocket::{WebSocketState, websocket_handler};
pub use mcp::{McpServer, McpMessage, McpRequest, McpResponse};
pub use handlers::{list_plugins, reload_plugin, root};