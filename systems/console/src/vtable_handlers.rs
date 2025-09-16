//! VTable command handlers for console operations

use bytes::Bytes;
use playground_core_ecs::{VTableResponse};
use playground_core_types::{CoreResult, CoreError, Handle};
use playground_core_console::{Console, LogLevel, LogEntry, Progress, OutputStyle};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use crate::{terminal::Terminal, dashboard::Dashboard};

// Global console implementation
static CONSOLE_IMPL: OnceCell<ConsoleImpl> = OnceCell::new();

struct ConsoleImpl {
    terminal: Arc<Terminal>,
    dashboard: Option<Arc<Dashboard>>,
    console_handle: Handle<Console>,
}

/// Initialize the console implementation
pub async fn initialize() -> CoreResult<()> {
    // Create terminal implementation
    let terminal = Arc::new(Terminal::new(true));
    
    // Create dashboard if enabled
    let dashboard = if std::env::var("ENABLE_DASHBOARD").is_ok() {
        Some(Arc::new(Dashboard::new().await?))
    } else {
        None
    };
    
    // Create and store the console handle
    let console_handle = Console::new();
    
    CONSOLE_IMPL.set(ConsoleImpl {
        terminal,
        dashboard,
        console_handle: console_handle.clone(),
    }).map_err(|_| CoreError::AlreadyInitialized)?;
    
    Ok(())
}

fn get_impl() -> CoreResult<&'static ConsoleImpl> {
    CONSOLE_IMPL.get().ok_or(CoreError::NotInitialized)
}

/// Handle output commands
pub async fn handle_output_command(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "write" => {
            let text: String = match bincode::deserialize(&payload) {
                Ok(t) => t,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.write(&text).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "write_styled" => {
            let (text, style): (String, OutputStyle) = match bincode::deserialize(&payload) {
                Ok(t) => t,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.write_styled(&text, style).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "write_line" => {
            let text: String = match bincode::deserialize(&payload) {
                Ok(t) => t,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.write_line(&text).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "clear" => {
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.clear().await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "flush" => {
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.flush().await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        _ => VTableResponse {
            success: false,
            payload: None,
            error: Some(format!("Unknown output operation: {}", operation)),
        }
    }
}

/// Handle logging commands
pub async fn handle_logging_command(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "log" => {
            let entry: LogEntry = match bincode::deserialize(&payload) {
                Ok(e) => e,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    // Log to terminal
                    if let Err(e) = impl_.terminal.log(&entry).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                    
                    // Log to dashboard if available
                    if let Some(dashboard) = &impl_.dashboard {
                        let _ = dashboard.log(&entry).await;
                    }
                    
                    // Store in console data structure
                    #[cfg(feature = "logging")]
                    {
                        let mut logs = impl_.console_handle.log_entries.write().await;
                        logs.push(entry.clone());
                        
                        // Also store component-specific logs
                        if let Some(component) = &entry.component {
                            let mut component_logs = impl_.console_handle.component_logs.write().await;
                            component_logs.entry(component.clone())
                                .or_insert_with(Vec::new)
                                .push(entry);
                        }
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "get_recent" => {
            let count: usize = match bincode::deserialize(&payload) {
                Ok(c) => c,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    let logs = impl_.terminal.get_recent_logs(count).await;
                    match bincode::serialize(&logs) {
                        Ok(data) => VTableResponse {
                            success: true,
                            payload: Some(Bytes::from(data)),
                            error: None,
                        },
                        Err(e) => VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        }
                    }
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
        },
        "get_component" => {
            let (component, count): (String, usize) = match bincode::deserialize(&payload) {
                Ok(c) => c,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    let logs = impl_.terminal.get_component_logs(&component, count).await;
                    match bincode::serialize(&logs) {
                        Ok(data) => VTableResponse {
                            success: true,
                            payload: Some(Bytes::from(data)),
                            error: None,
                        },
                        Err(e) => VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        }
                    }
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
        },
        "clear" => {
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.clear_logs().await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                    
                    #[cfg(feature = "logging")]
                    {
                        let mut logs = impl_.console_handle.log_entries.write().await;
                        logs.clear();
                        
                        let mut component_logs = impl_.console_handle.component_logs.write().await;
                        component_logs.clear();
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "get_level" => {
            match get_impl() {
                Ok(impl_) => {
                    let level = impl_.terminal.get_log_level().await;
                    match bincode::serialize(&level) {
                        Ok(data) => VTableResponse {
                            success: true,
                            payload: Some(Bytes::from(data)),
                            error: None,
                        },
                        Err(e) => VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        }
                    }
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
        },
        "set_level" => {
            let level: LogLevel = match bincode::deserialize(&payload) {
                Ok(l) => l,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.set_log_level(level).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                    
                    #[cfg(feature = "logging")]
                    {
                        let mut console_level = impl_.console_handle.log_level.write().await;
                        *console_level = level;
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        _ => VTableResponse {
            success: false,
            payload: None,
            error: Some(format!("Unknown logging operation: {}", operation)),
        }
    }
}

/// Handle progress commands
pub async fn handle_progress_command(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "update" => {
            let progress: Progress = match bincode::deserialize(&payload) {
                Ok(p) => p,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.update_progress(progress.clone()).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                    
                    #[cfg(feature = "progress")]
                    {
                        let mut indicators = impl_.console_handle.progress_indicators.write().await;
                        indicators.insert(progress.id.clone(), progress);
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "clear" => {
            let id: String = match bincode::deserialize(&payload) {
                Ok(i) => i,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };
            
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.clear_progress(&id).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                    
                    #[cfg(feature = "progress")]
                    {
                        let mut indicators = impl_.console_handle.progress_indicators.write().await;
                        indicators.remove(&id);
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        "clear_all" => {
            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.clear_all_progress().await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                    
                    #[cfg(feature = "progress")]
                    {
                        let mut indicators = impl_.console_handle.progress_indicators.write().await;
                        indicators.clear();
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
            
            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        _ => VTableResponse {
            success: false,
            payload: None,
            error: Some(format!("Unknown progress operation: {}", operation)),
        }
    }
}

/// Handle input commands
pub async fn handle_input_command(operation: String, _payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "read_line" => {
            match get_impl() {
                Ok(impl_) => {
                    match impl_.terminal.read_line().await {
                        Ok(line) => {
                            match bincode::serialize(&line) {
                                Ok(data) => VTableResponse {
                                    success: true,
                                    payload: Some(Bytes::from(data)),
                                    error: None,
                                },
                                Err(e) => VTableResponse {
                                    success: false,
                                    payload: None,
                                    error: Some(e.to_string()),
                                }
                            }
                        },
                        Err(e) => VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        }
                    }
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
        },
        "read_event" => {
            match get_impl() {
                Ok(impl_) => {
                    match impl_.terminal.read_event().await {
                        Ok(event) => {
                            match bincode::serialize(&event) {
                                Ok(data) => VTableResponse {
                                    success: true,
                                    payload: Some(Bytes::from(data)),
                                    error: None,
                                },
                                Err(e) => VTableResponse {
                                    success: false,
                                    payload: None,
                                    error: Some(e.to_string()),
                                }
                            }
                        },
                        Err(e) => VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        }
                    }
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
        },
        "has_input" => {
            match get_impl() {
                Ok(impl_) => {
                    let has = impl_.terminal.has_input().await;
                    match bincode::serialize(&has) {
                        Ok(data) => VTableResponse {
                            success: true,
                            payload: Some(Bytes::from(data)),
                            error: None,
                        },
                        Err(e) => VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        }
                    }
                },
                Err(e) => VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }
        },
        _ => VTableResponse {
            success: false,
            payload: None,
            error: Some(format!("Unknown input operation: {}", operation)),
        }
    }
}

/// Handle registry commands (for getting console handle)
pub async fn handle_registry_command(operation: String, _payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "get" => {
            // For now, we can't actually return the Handle<Console> through bytes
            // This would need a more complex approach in real implementation
            VTableResponse {
                success: false,
                payload: None,
                error: Some("Console handle registry not yet implemented".to_string()),
            }
        },
        _ => VTableResponse {
            success: false,
            payload: None,
            error: Some(format!("Unknown registry operation: {}", operation)),
        }
    }
}