//! Public API functions for client operations
//! 
//! These functions provide a convenient way to access client functionality
//! without needing to manage the client instance directly.

use once_cell::sync::Lazy;
use playground_core_types::{Handle, CoreResult};
use playground_core_rendering::RenderCommand;
use crate::{Client, ClientId, ClientConfig, ClientState, ClientStats, RenderTarget};
use crate::input::{InputEvent, KeyCode};

/// Global client instance
static CLIENT_INSTANCE: Lazy<Handle<Client>> = Lazy::new(|| {
    // Generate a unique client ID based on process ID and time
    let id = ClientId(std::process::id() as u64);
    Client::new(id)
});

/// Get the global client instance
pub fn get_client_instance() -> CoreResult<&'static Handle<Client>> {
    Ok(&*CLIENT_INSTANCE)
}

/// Initialize the client with the given configuration
pub async fn initialize_client(config: ClientConfig) -> CoreResult<()> {
    get_client_instance()?.initialize(config).await
}

/// Connect to a server
pub async fn connect_to_server(address: &str) -> CoreResult<()> {
    get_client_instance()?.connect(address).await
}

/// Disconnect from server
pub async fn disconnect_from_server() -> CoreResult<()> {
    get_client_instance()?.disconnect().await
}

/// Get current client state
pub async fn get_client_state() -> CoreResult<ClientState> {
    Ok(get_client_instance()?.state().await)
}

/// Get client ID
pub fn get_client_id() -> CoreResult<ClientId> {
    Ok(get_client_instance()?.id())
}

/// Send a message to the server
pub async fn send_to_server(data: Vec<u8>) -> CoreResult<()> {
    get_client_instance()?.send(data).await
}

/// Receive a message from the server (if available)
pub async fn receive_from_server() -> CoreResult<Option<Vec<u8>>> {
    get_client_instance()?.receive().await
}

/// Update the client (called each frame)
pub async fn update_client(delta_time: f32) -> CoreResult<()> {
    get_client_instance()?.update(delta_time).await
}

/// Get client statistics
pub async fn get_client_stats() -> CoreResult<ClientStats> {
    Ok(get_client_instance()?.stats().await)
}

// Rendering API (feature-gated)
#[cfg(feature = "rendering")]
pub async fn create_render_target(target: RenderTarget) -> CoreResult<u32> {
    get_client_instance()?.create_render_target(target).await
}

#[cfg(feature = "rendering")]
pub async fn destroy_render_target(id: u32) -> CoreResult<()> {
    get_client_instance()?.destroy_render_target(id).await
}

#[cfg(feature = "rendering")]
pub async fn get_current_render_target() -> CoreResult<Option<u32>> {
    Ok(get_client_instance()?.current_render_target().await)
}

#[cfg(feature = "rendering")]
pub async fn set_render_target(id: u32) -> CoreResult<()> {
    get_client_instance()?.set_render_target(id).await
}

#[cfg(feature = "rendering")]
pub async fn submit_render_commands(commands: Vec<RenderCommand>) -> CoreResult<()> {
    get_client_instance()?.render(commands).await
}

#[cfg(feature = "rendering")]
pub async fn present_frame() -> CoreResult<()> {
    get_client_instance()?.present().await
}

#[cfg(feature = "rendering")]
pub async fn resize_render_target(id: u32, width: u32, height: u32) -> CoreResult<()> {
    get_client_instance()?.resize(id, width, height).await
}

// Input API (feature-gated)
#[cfg(feature = "input")]
pub async fn poll_input_events() -> CoreResult<Vec<InputEvent>> {
    get_client_instance()?.poll_input().await
}

#[cfg(feature = "input")]
pub async fn set_input_capture(capture: bool) -> CoreResult<()> {
    get_client_instance()?.set_input_capture(capture).await
}

#[cfg(feature = "input")]
pub async fn is_key_pressed(key: KeyCode) -> CoreResult<bool> {
    Ok(get_client_instance()?.is_key_pressed(key).await)
}

#[cfg(feature = "input")]
pub async fn get_pointer_position() -> CoreResult<Option<(f32, f32)>> {
    Ok(get_client_instance()?.pointer_position().await)
}

// Audio API (feature-gated)
#[cfg(feature = "audio")]
pub async fn play_audio(data: Vec<u8>, format: crate::client::AudioFormat) -> CoreResult<u32> {
    get_client_instance()?.play_audio(data, format).await
}

#[cfg(feature = "audio")]
pub async fn stop_audio(id: u32) -> CoreResult<()> {
    get_client_instance()?.stop_audio(id).await
}

#[cfg(feature = "audio")]
pub async fn set_audio_volume(volume: f32) -> CoreResult<()> {
    get_client_instance()?.set_volume(volume).await
}

#[cfg(feature = "audio")]
pub async fn get_audio_volume() -> CoreResult<f32> {
    Ok(get_client_instance()?.volume().await)
}