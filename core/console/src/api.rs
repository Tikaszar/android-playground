//! Public API functions for console operations
//! 
//! These functions use the global console instance through core/ecs registry.

use playground_core_types::{CoreResult, CoreError};
use playground_core_ecs;
use crate::{Console, LogLevel, LogEntry};

#[cfg(feature = "progress")]
use crate::Progress;

#[cfg(feature = "input")]
use crate::input::InputEvent;

#[cfg(feature = "styling")]
use crate::OutputStyle;

/// Get the global console instance from the world registry
async fn get_console() -> CoreResult<playground_core_types::Handle<Console>> {
    let world = playground_core_ecs::get_world().await?;
    
    // Get console from world's resource registry via VTable
    let response = world.vtable.send_command(
        "console.registry",
        "get".to_string(),
        bytes::Bytes::new()
    ).await?;
    
    if !response.success {
        return Err(CoreError::NotInitialized);
    }
    
    // The response contains a handle to the Console
    // In practice, systems/console will register the console instance
    // For now, return error until systems/console is implemented
    Err(CoreError::NotRegistered("console".to_string()))
}

// Output API functions
#[cfg(feature = "output")]
pub async fn write(text: &str) -> CoreResult<()> {
    let console = get_console().await?;
    console.write(text).await
}

#[cfg(feature = "output")]
pub async fn write_line(text: &str) -> CoreResult<()> {
    let console = get_console().await?;
    console.write_line(text).await
}

#[cfg(all(feature = "output", feature = "styling"))]
pub async fn write_styled(text: &str, style: OutputStyle) -> CoreResult<()> {
    let console = get_console().await?;
    console.write_styled(text, style).await
}

#[cfg(feature = "output")]
pub async fn clear() -> CoreResult<()> {
    let console = get_console().await?;
    console.clear().await
}

#[cfg(feature = "output")]
pub async fn flush() -> CoreResult<()> {
    let console = get_console().await?;
    console.flush().await
}

// Logging API functions
#[cfg(feature = "logging")]
pub async fn log(level: LogLevel, message: String) -> CoreResult<()> {
    let console = get_console().await?;
    console.log_simple(level, message).await
}

#[cfg(feature = "logging")]
pub async fn log_component(component: &str, level: LogLevel, message: String) -> CoreResult<()> {
    let console = get_console().await?;
    console.log_component(component, level, message).await
}

#[cfg(feature = "logging")]
pub async fn log_entry(entry: LogEntry) -> CoreResult<()> {
    let console = get_console().await?;
    console.log(entry).await
}

#[cfg(feature = "logging")]
pub async fn get_recent_logs(count: usize) -> CoreResult<Vec<LogEntry>> {
    let console = get_console().await?;
    console.get_recent_logs(count).await
}

#[cfg(feature = "logging")]
pub async fn get_component_logs(component: &str, count: usize) -> CoreResult<Vec<LogEntry>> {
    let console = get_console().await?;
    console.get_component_logs(component, count).await
}

#[cfg(feature = "logging")]
pub async fn clear_logs() -> CoreResult<()> {
    let console = get_console().await?;
    console.clear_logs().await
}

#[cfg(feature = "logging")]
pub async fn get_log_level() -> CoreResult<LogLevel> {
    let console = get_console().await?;
    console.get_log_level().await
}

#[cfg(feature = "logging")]
pub async fn set_log_level(level: LogLevel) -> CoreResult<()> {
    let console = get_console().await?;
    console.set_log_level(level).await
}

// Progress API functions
#[cfg(feature = "progress")]
pub async fn update_progress(progress: Progress) -> CoreResult<()> {
    let console = get_console().await?;
    console.update_progress(progress).await
}

#[cfg(feature = "progress")]
pub async fn clear_progress(id: &str) -> CoreResult<()> {
    let console = get_console().await?;
    console.clear_progress(id).await
}

#[cfg(feature = "progress")]
pub async fn clear_all_progress() -> CoreResult<()> {
    let console = get_console().await?;
    console.clear_all_progress().await
}

// Input API functions
#[cfg(feature = "input")]
pub async fn read_line() -> CoreResult<String> {
    let console = get_console().await?;
    console.read_line().await
}

#[cfg(feature = "input")]
pub async fn read_event() -> CoreResult<Option<InputEvent>> {
    let console = get_console().await?;
    console.read_event().await
}

#[cfg(feature = "input")]
pub async fn has_input() -> CoreResult<bool> {
    let console = get_console().await?;
    console.has_input().await
}