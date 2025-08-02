use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub payload: serde_json::Value,
}

pub struct Context {
    pub resources: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    pub messages: Vec<Message>,
}

pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub frame_time: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Plugin load failed: {0}")]
    LoadFailed(String),
    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),
}