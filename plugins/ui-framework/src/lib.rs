mod plugin;
mod panel_manager;
mod mcp_handler;
mod browser_bridge;
mod ui_state;
mod components;
mod channel_manager;
mod message_system;

pub use plugin::UiFrameworkPlugin;
pub use components::*;
pub use channel_manager::ChannelManager;
pub use message_system::MessageSystem;