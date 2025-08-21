//! WebSocket message definitions and handlers for UI system

use bytes::{Bytes, BytesMut, BufMut, Buf};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use nalgebra::{Vector2, Vector4};
use crate::error::{UiError, UiResult};

/// UI system packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum UiPacketType {
    // Client to Server
    CreateElement = 1,
    UpdateElement = 2,
    DeleteElement = 3,
    InputEvent = 4,
    ResizeScreen = 5,
    RequestState = 6,
    
    // Server to Client  
    ElementCreated = 100,
    ElementUpdated = 101,
    ElementDeleted = 102,
    StateSync = 103,
    RenderBatch = 104,
    ThemeUpdate = 105,
    
    // Terminal-specific
    TerminalInput = 200,
    TerminalOutput = 201,
    TerminalConnect = 202,
    TerminalDisconnect = 203,
    TerminalState = 204,
}

impl TryFrom<u16> for UiPacketType {
    type Error = UiError;
    
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(UiPacketType::CreateElement),
            2 => Ok(UiPacketType::UpdateElement),
            3 => Ok(UiPacketType::DeleteElement),
            4 => Ok(UiPacketType::InputEvent),
            5 => Ok(UiPacketType::ResizeScreen),
            6 => Ok(UiPacketType::RequestState),
            100 => Ok(UiPacketType::ElementCreated),
            101 => Ok(UiPacketType::ElementUpdated),
            102 => Ok(UiPacketType::ElementDeleted),
            103 => Ok(UiPacketType::StateSync),
            104 => Ok(UiPacketType::RenderBatch),
            105 => Ok(UiPacketType::ThemeUpdate),
            200 => Ok(UiPacketType::TerminalInput),
            201 => Ok(UiPacketType::TerminalOutput),
            202 => Ok(UiPacketType::TerminalConnect),
            203 => Ok(UiPacketType::TerminalDisconnect),
            204 => Ok(UiPacketType::TerminalState),
            _ => Err(UiError::InvalidOperation(format!("Invalid packet type: {}", value))),
        }
    }
}

/// Create element message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateElementMessage {
    pub parent_id: Option<Uuid>,
    pub element_type: String,
    pub name: String,
    pub position: Vector2<f32>,
    pub size: Vector2<f32>,
}

/// Update element message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateElementMessage {
    pub element_id: Uuid,
    pub position: Option<Vector2<f32>>,
    pub size: Option<Vector2<f32>>,
    pub visible: Option<bool>,
    pub style: Option<StyleUpdate>,
}

/// Style update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleUpdate {
    pub background_color: Option<Vector4<f32>>,
    pub border_color: Option<Vector4<f32>>,
    pub text_color: Option<Vector4<f32>>,
    pub border_width: Option<f32>,
    pub border_radius: Option<f32>,
    pub opacity: Option<f32>,
}

/// Input event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEventMessage {
    pub element_id: Option<Uuid>,
    pub event_type: String,
    pub position: Option<Vector2<f32>>,
    pub key: Option<String>,
    pub text: Option<String>,
    pub modifiers: Option<InputModifiers>,
}

/// Input modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Terminal input message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInputMessage {
    pub terminal_id: Uuid,
    pub input: String,
}

/// Terminal output message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputMessage {
    pub terminal_id: Uuid,
    pub output: String,
    pub is_error: bool,
}

/// Terminal connect message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConnectMessage {
    pub terminal_id: Uuid,
    pub shell_path: Option<String>,
    pub working_dir: Option<String>,
}

/// Terminal state message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalStateMessage {
    pub terminal_id: Uuid,
    pub connected: bool,
    pub ready: bool,
}

/// Render batch message - contains multiple render commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderBatchMessage {
    pub frame_id: u64,
    pub commands: Vec<RenderCommand>,
}

/// Individual render command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderCommand {
    Clear { color: Vector4<f32> },
    DrawQuad { position: Vector2<f32>, size: Vector2<f32>, color: Vector4<f32> },
    DrawText { position: Vector2<f32>, text: String, size: f32, color: Vector4<f32> },
    DrawImage { position: Vector2<f32>, size: Vector2<f32>, texture_id: u32 },
    SetClipRect { position: Vector2<f32>, size: Vector2<f32> },
    ClearClipRect,
}

/// Message serialization helpers
pub fn serialize_message<T: Serialize>(msg: &T) -> UiResult<Bytes> {
    let json = serde_json::to_vec(msg)
        .map_err(|e| UiError::SerializationError(e.to_string()))?;
    Ok(Bytes::from(json))
}

pub fn deserialize_message<T: for<'de> Deserialize<'de>>(data: &Bytes) -> UiResult<T> {
    serde_json::from_slice(data)
        .map_err(|e| UiError::SerializationError(e.to_string()))
}

/// Binary protocol helpers for efficient encoding
pub mod binary {
    use super::*;
    
    /// Encode a Vector2 to bytes
    pub fn encode_vec2(buf: &mut BytesMut, v: &Vector2<f32>) {
        buf.put_f32(v.x);
        buf.put_f32(v.y);
    }
    
    /// Decode a Vector2 from bytes
    pub fn decode_vec2(buf: &mut Bytes) -> Vector2<f32> {
        let x = buf.get_f32();
        let y = buf.get_f32();
        Vector2::new(x, y)
    }
    
    /// Encode a Vector4 to bytes
    pub fn encode_vec4(buf: &mut BytesMut, v: &Vector4<f32>) {
        buf.put_f32(v.x);
        buf.put_f32(v.y);
        buf.put_f32(v.z);
        buf.put_f32(v.w);
    }
    
    /// Decode a Vector4 from bytes
    pub fn decode_vec4(buf: &mut Bytes) -> Vector4<f32> {
        let x = buf.get_f32();
        let y = buf.get_f32();
        let z = buf.get_f32();
        let w = buf.get_f32();
        Vector4::new(x, y, z, w)
    }
    
    /// Encode a UUID to bytes
    pub fn encode_uuid(buf: &mut BytesMut, id: &Uuid) {
        buf.put_slice(id.as_bytes());
    }
    
    /// Decode a UUID from bytes
    pub fn decode_uuid(buf: &mut Bytes) -> UiResult<Uuid> {
        if buf.remaining() < 16 {
            return Err(UiError::SerializationError("Not enough bytes for UUID".to_string()));
        }
        let mut bytes = [0u8; 16];
        buf.copy_to_slice(&mut bytes);
        Ok(Uuid::from_bytes(bytes))
    }
    
    /// Encode a string to bytes
    pub fn encode_string(buf: &mut BytesMut, s: &str) {
        let bytes = s.as_bytes();
        buf.put_u32(bytes.len() as u32);
        buf.put_slice(bytes);
    }
    
    /// Decode a string from bytes
    pub fn decode_string(buf: &mut Bytes) -> UiResult<String> {
        if buf.remaining() < 4 {
            return Err(UiError::SerializationError("Not enough bytes for string length".to_string()));
        }
        let len = buf.get_u32() as usize;
        if buf.remaining() < len {
            return Err(UiError::SerializationError(format!("Not enough bytes for string: expected {}, got {}", len, buf.remaining())));
        }
        let mut bytes = vec![0u8; len];
        buf.copy_to_slice(&mut bytes);
        String::from_utf8(bytes)
            .map_err(|e| UiError::SerializationError(format!("Invalid UTF-8: {}", e)))
    }
}