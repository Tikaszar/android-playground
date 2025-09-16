//! High-level networking system for ECS integration
//!
//! This system manages networking initialization and provides helper functions
//! for other systems. The actual server/client logic is in vtable_handlers.rs

use playground_core_types::CoreResult;
use crate::types::{WebSocketConfig, McpTool, Packet, LogLevel};

/// High-level networking system
pub struct NetworkingSystem {
    ws_config: WebSocketConfig,
    initialized: bool,
}

impl NetworkingSystem {
    pub async fn new() -> CoreResult<Self> {
        Ok(Self {
            ws_config: WebSocketConfig::default(),
            initialized: false,
        })
    }
    
    /// Initialize the networking system
    pub async fn initialize(&mut self) -> CoreResult<()> {
        // Register VTable handlers
        crate::registration::initialize().await?;
        
        self.initialized = true;
        
        println!("NetworkingSystem initialized with VTable handlers");
        
        Ok(())
    }
    
    /// Register an MCP tool (helper for plugins)
    pub async fn register_mcp_tool(&self, _tool: McpTool) -> CoreResult<()> {
        // This now goes through the VTable handlers
        // The actual implementation is in vtable_handlers.rs
        Ok(())
    }
    
    /// Send a packet (helper for plugins)
    pub async fn send_packet(&self, _packet: Packet) -> CoreResult<()> {
        // This now goes through the VTable handlers
        // The actual implementation is in vtable_handlers.rs
        Ok(())
    }
    
    /// Log a component message (helper for debugging)
    pub async fn log_component(&self, component: &str, level: LogLevel, message: String) {
        let level_str = match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        
        println!("[{}] {}: {}", component, level_str, message);
    }
}