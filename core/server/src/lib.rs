pub mod channel;
pub mod packet;
pub mod batcher;
pub mod websocket;
pub mod handlers;
pub mod mcp;
pub mod dashboard;

pub use channel::{ChannelManager, ChannelInfo};
pub use packet::{Packet, Priority};
pub use batcher::FrameBatcher;
pub use websocket::{WebSocketState, websocket_handler};
pub use mcp::McpServer;
pub use handlers::{list_plugins, reload_plugin, root};
pub use dashboard::Dashboard;