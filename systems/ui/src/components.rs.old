//! UI-specific ECS components for internal state management

use async_trait::async_trait;
use bytes::{Bytes, BytesMut, BufMut};
use nalgebra::{Vector2, Vector4};
use playground_core_ecs::{Component, ComponentId, EcsResult, EcsError};
use std::collections::HashMap;
use uuid::Uuid;
use crate::element::ElementBounds;
use crate::layout::LayoutConstraints;
use crate::theme::ThemeId;

/// Component storing basic UI element data
#[derive(Debug, Clone)]
pub struct UiElementComponent {
    pub id: Uuid,
    pub name: String,
    pub tag: String,
    pub bounds: ElementBounds,
    pub children: Vec<playground_core_ecs::EntityId>,
    pub parent: Option<playground_core_ecs::EntityId>,
    pub visible: bool,
    pub interactive: bool,
    pub z_index: i32,
}

#[async_trait]
impl Component for UiElementComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize ID (16 bytes)
        buf.put_slice(self.id.as_bytes());
        
        // Serialize name length and name
        buf.put_u32_le(self.name.len() as u32);
        buf.put_slice(self.name.as_bytes());
        
        // Serialize tag length and tag
        buf.put_u32_le(self.tag.len() as u32);
        buf.put_slice(self.tag.as_bytes());
        
        // Serialize bounds (16 bytes: x, y, width, height as f32)
        buf.put_f32_le(self.bounds.position.x);
        buf.put_f32_le(self.bounds.position.y);
        buf.put_f32_le(self.bounds.size.x);
        buf.put_f32_le(self.bounds.size.y);
        
        // Serialize children count and IDs
        buf.put_u32_le(self.children.len() as u32);
        for child in &self.children {
            buf.put_u32_le(child.index());
            buf.put_u32_le(child.generation().value());
        }
        
        // Serialize parent (optional)
        if let Some(parent) = self.parent {
            buf.put_u8(1);
            buf.put_u32_le(parent.index());
            buf.put_u32_le(parent.generation().value());
        } else {
            buf.put_u8(0);
        }
        
        // Serialize flags and z_index
        buf.put_u8(if self.visible { 1 } else { 0 });
        buf.put_u8(if self.interactive { 1 } else { 0 });
        buf.put_i32_le(self.z_index);
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        // Implement deserialization when needed
        Err(EcsError::SerializationError("UiElementComponent deserialization not implemented".into()))
    }
}

/// Component storing layout constraints and results
#[derive(Debug, Clone)]
pub struct UiLayoutComponent {
    pub constraints: LayoutConstraints,
    pub computed_size: Vector2<f32>,
    pub computed_position: Vector2<f32>,
    pub padding: Vector4<f32>, // top, right, bottom, left
    pub margin: Vector4<f32>,  // top, right, bottom, left
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub align_self: AlignSelf,
    pub justify_self: JustifySelf,
}

#[derive(Debug, Clone, Copy)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
}

#[derive(Debug, Clone, Copy)]
pub enum JustifySelf {
    Auto,
    Start,
    End,
    Center,
    Stretch,
}

#[async_trait]
impl Component for UiLayoutComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize computed size and position
        buf.put_f32_le(self.computed_size.x);
        buf.put_f32_le(self.computed_size.y);
        buf.put_f32_le(self.computed_position.x);
        buf.put_f32_le(self.computed_position.y);
        
        // Serialize padding
        buf.put_f32_le(self.padding.x);
        buf.put_f32_le(self.padding.y);
        buf.put_f32_le(self.padding.z);
        buf.put_f32_le(self.padding.w);
        
        // Serialize margin
        buf.put_f32_le(self.margin.x);
        buf.put_f32_le(self.margin.y);
        buf.put_f32_le(self.margin.z);
        buf.put_f32_le(self.margin.w);
        
        // Serialize flex properties
        buf.put_f32_le(self.flex_grow);
        buf.put_f32_le(self.flex_shrink);
        buf.put_f32_le(self.flex_basis);
        
        // Serialize alignment
        buf.put_u8(self.align_self as u8);
        buf.put_u8(self.justify_self as u8);
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("UiLayoutComponent deserialization not implemented".into()))
    }
}

/// Component storing style and theme information
#[derive(Debug, Clone)]
pub struct UiStyleComponent {
    pub theme_id: ThemeId,
    pub background_color: Vector4<f32>, // RGBA
    pub border_color: Vector4<f32>,     // RGBA
    pub text_color: Vector4<f32>,       // RGBA
    pub border_width: f32,
    pub border_radius: f32,
    pub opacity: f32,
    pub custom_properties: HashMap<String, String>,
}

#[async_trait]
impl Component for UiStyleComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        // Serialize theme ID
        buf.put_u32_le(self.theme_id.0);
        
        // Serialize colors
        buf.put_f32_le(self.background_color.x);
        buf.put_f32_le(self.background_color.y);
        buf.put_f32_le(self.background_color.z);
        buf.put_f32_le(self.background_color.w);
        
        buf.put_f32_le(self.border_color.x);
        buf.put_f32_le(self.border_color.y);
        buf.put_f32_le(self.border_color.z);
        buf.put_f32_le(self.border_color.w);
        
        buf.put_f32_le(self.text_color.x);
        buf.put_f32_le(self.text_color.y);
        buf.put_f32_le(self.text_color.z);
        buf.put_f32_le(self.text_color.w);
        
        // Serialize other properties
        buf.put_f32_le(self.border_width);
        buf.put_f32_le(self.border_radius);
        buf.put_f32_le(self.opacity);
        
        // Serialize custom properties count
        buf.put_u32_le(self.custom_properties.len() as u32);
        for (key, value) in &self.custom_properties {
            buf.put_u32_le(key.len() as u32);
            buf.put_slice(key.as_bytes());
            buf.put_u32_le(value.len() as u32);
            buf.put_slice(value.as_bytes());
        }
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("UiStyleComponent deserialization not implemented".into()))
    }
}

/// Component marking elements that need re-rendering
#[derive(Debug, Clone)]
pub struct UiDirtyComponent {
    pub layout_dirty: bool,
    pub style_dirty: bool,
    pub content_dirty: bool,
    pub last_render_frame: u64,
}

#[async_trait]
impl Component for UiDirtyComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        buf.put_u8(if self.layout_dirty { 1 } else { 0 });
        buf.put_u8(if self.style_dirty { 1 } else { 0 });
        buf.put_u8(if self.content_dirty { 1 } else { 0 });
        buf.put_u64_le(self.last_render_frame);
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("UiDirtyComponent deserialization not implemented".into()))
    }
}

/// Component storing input state and handlers
#[derive(Debug, Clone)]
pub struct UiInputComponent {
    pub accepts_input: bool,
    pub has_focus: bool,
    pub hover: bool,
    pub pressed: bool,
    pub tab_index: Option<i32>,
    pub last_interaction: Option<std::time::Instant>,
}

#[async_trait]
impl Component for UiInputComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        buf.put_u8(if self.accepts_input { 1 } else { 0 });
        buf.put_u8(if self.has_focus { 1 } else { 0 });
        buf.put_u8(if self.hover { 1 } else { 0 });
        buf.put_u8(if self.pressed { 1 } else { 0 });
        
        if let Some(tab_index) = self.tab_index {
            buf.put_u8(1);
            buf.put_i32_le(tab_index);
        } else {
            buf.put_u8(0);
        }
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("UiInputComponent deserialization not implemented".into()))
    }
}

/// Component for WebSocket connection state (for terminal, etc)
#[derive(Debug, Clone)]
pub struct UiWebSocketComponent {
    pub channel_id: u16,
    pub connected: bool,
    pub last_message: Option<std::time::Instant>,
    pub pending_messages: Vec<Bytes>,
}

#[async_trait]
impl Component for UiWebSocketComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        buf.put_u16_le(self.channel_id);
        buf.put_u8(if self.connected { 1 } else { 0 });
        buf.put_u32_le(self.pending_messages.len() as u32);
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("UiWebSocketComponent deserialization not implemented".into()))
    }
}

/// Component for text content
#[derive(Debug, Clone)]
pub struct UiTextComponent {
    pub text: String,
    pub font_size: f32,
    pub font_family: String,
    pub font_weight: FontWeight,
    pub text_align: TextAlign,
    pub line_height: f32,
    pub letter_spacing: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum FontWeight {
    Thin,
    Light,
    Regular,
    Medium,
    Bold,
    Black,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[async_trait]
impl Component for UiTextComponent {
    fn component_id() -> ComponentId {
        std::any::TypeId::of::<Self>()
    }
    
    async fn serialize(&self) -> EcsResult<Bytes> {
        let mut buf = BytesMut::new();
        
        buf.put_u32_le(self.text.len() as u32);
        buf.put_slice(self.text.as_bytes());
        
        buf.put_f32_le(self.font_size);
        
        buf.put_u32_le(self.font_family.len() as u32);
        buf.put_slice(self.font_family.as_bytes());
        
        buf.put_u8(self.font_weight as u8);
        buf.put_u8(self.text_align as u8);
        buf.put_f32_le(self.line_height);
        buf.put_f32_le(self.letter_spacing);
        
        Ok(buf.freeze())
    }
    
    async fn deserialize(_data: &Bytes) -> EcsResult<Self> where Self: Sized {
        Err(EcsError::SerializationError("UiTextComponent deserialization not implemented".into()))
    }
}