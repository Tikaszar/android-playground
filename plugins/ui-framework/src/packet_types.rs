/// Packet types for UI Framework Plugin communication
/// 
/// The UI Framework Plugin uses a single channel and differentiates
/// messages using these packet type constants.

// Messages from browser to server (1-99)
pub const PACKET_TYPE_MCP_TOOL_CALL: u16 = 1;
pub const PACKET_TYPE_PANEL_UPDATE: u16 = 2;
pub const PACKET_TYPE_CHAT_MESSAGE: u16 = 3;
pub const PACKET_TYPE_UI_EVENT: u16 = 4;
pub const PACKET_TYPE_TOUCH_EVENT: u16 = 5;
pub const PACKET_TYPE_KEY_EVENT: u16 = 6;
pub const PACKET_TYPE_RESIZE: u16 = 7;

// Messages from server to browser (100-199)
pub const PACKET_TYPE_RENDER_BATCH: u16 = 100;
pub const PACKET_TYPE_UI_UPDATE: u16 = 101;
pub const PACKET_TYPE_COMPONENT_UPDATE: u16 = 102;
pub const PACKET_TYPE_SHOW_COMPONENT: u16 = 103;
pub const PACKET_TYPE_HIDE_COMPONENT: u16 = 104;
pub const PACKET_TYPE_UPDATE_CHAT: u16 = 105;