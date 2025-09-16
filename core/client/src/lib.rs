//! Core client data structures and types
//!
//! This package defines ONLY data structures - NO LOGIC!
//! All client implementation logic lives in systems packages like
//! systems/webgl (browser) or systems/native (future).
//! 
//! This follows the "abstract base class" pattern where core defines
//! structure and systems provide behavior.

// Data structure modules
pub mod client;
pub mod operations;
pub mod types;
pub mod input;
pub mod api;

// Re-export the main Client struct
pub use client::Client;

#[cfg(feature = "audio")]
pub use client::{AudioFormat, AudioTrackInfo};

// Re-export all types
pub use types::{
    ClientId,
    ClientState,
    ClientCapabilities,
    ClientConfig,
    RenderTarget,
    ClientStats,
};

// Re-export input types
pub use input::{
    InputEvent,
    InputState,
    KeyboardEvent,
    KeyState,
    KeyCode,
    Modifiers,
    PointerEvent,
    PointerButtons,
    PointerEventType,
    TouchEvent,
    TouchEventType,
    GamepadEvent,
    GamepadEventType,
    TextEvent,
    WindowEvent,
    Touch,
    GamepadState,
};

// Re-export API functions
pub use api::{
    get_client_instance,
    initialize_client,
    connect_to_server,
    disconnect_from_server,
    get_client_state,
    get_client_id,
    send_to_server,
    receive_from_server,
    update_client,
    get_client_stats,
};

#[cfg(feature = "rendering")]
pub use api::{
    create_render_target,
    destroy_render_target,
    get_current_render_target,
    set_render_target,
    submit_render_commands,
    present_frame,
    resize_render_target,
};

#[cfg(feature = "input")]
pub use api::{
    poll_input_events,
    set_input_capture,
    is_key_pressed,
    get_pointer_position,
};

#[cfg(feature = "audio")]
pub use api::{
    play_audio,
    stop_audio,
    set_audio_volume,
    get_audio_volume,
};