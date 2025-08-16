//! Conversational chat interface

use crate::element::{Element, ElementBase, ElementBounds, ElementId};
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use crate::theme::Theme;
use crate::error::UiResult;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::VecDeque;
use uuid::Uuid;

/// Message type in the chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    User,
    Assistant,
    System,
}

/// Code block within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub id: Uuid,
    pub language: String,
    pub content: String,
    pub editable: bool,
    pub focused_lines: Option<std::ops::Range<usize>>,
}

/// Chat message with optional inline code blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub message_type: MessageType,
    pub timestamp: u64,
    pub content: String,
    pub code_blocks: Vec<CodeBlock>,
    pub context_actions: Vec<ContextAction>,
}

/// Context action buttons for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAction {
    pub id: Uuid,
    pub label: String,
    pub action: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Save { file_path: String },
    Run { command: String },
    OpenInEditor,
    CopyToClipboard,
    ShowDiff,
}

/// Chat interface for conversational IDE
pub struct ChatInterface {
    base: ElementBase,
    messages: VecDeque<ChatMessage>,
    max_messages: usize,
    scroll_offset: f32,
    input_buffer: String,
    is_focused: bool,
    message_elements: Vec<ElementId>,
}

impl ChatInterface {
    pub fn new() -> Self {
        Self {
            base: ElementBase::new(),
            messages: VecDeque::new(),
            max_messages: 1000,
            scroll_offset: 0.0,
            input_buffer: String::new(),
            is_focused: false,
            message_elements: Vec::new(),
        }
    }
    
    pub fn add_message(&mut self, message: ChatMessage) {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
        self.base.mark_dirty();
    }
    
    pub fn add_user_message(&mut self, content: String) {
        let message = ChatMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::User,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            content,
            code_blocks: Vec::new(),
            context_actions: Vec::new(),
        };
        self.add_message(message);
    }
    
    pub fn add_assistant_message(&mut self, content: String, code_blocks: Vec<CodeBlock>) {
        let mut context_actions = Vec::new();
        
        // Add default context actions for code blocks
        if !code_blocks.is_empty() {
            context_actions.push(ContextAction {
                id: Uuid::new_v4(),
                label: "Open in IDE".to_string(),
                action: ActionType::OpenInEditor,
            });
            context_actions.push(ContextAction {
                id: Uuid::new_v4(),
                label: "Copy".to_string(),
                action: ActionType::CopyToClipboard,
            });
        }
        
        let message = ChatMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::Assistant,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            content,
            code_blocks,
            context_actions,
        };
        self.add_message(message);
    }
    
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.base.mark_dirty();
    }
    
    pub fn get_visible_messages(&self) -> Vec<&ChatMessage> {
        // Calculate visible messages based on scroll_offset and viewport
        let viewport_height = self.base.bounds.size.y;
        let message_height = 100.0; // Estimated height per message
        let visible_count = (viewport_height / message_height).ceil() as usize + 2;
        let skip_count = (self.scroll_offset / message_height).floor() as usize;
        
        self.messages
            .iter()
            .skip(skip_count)
            .take(visible_count)
            .collect()
    }
    
    pub fn handle_context_action(&mut self, message_id: Uuid, action_id: Uuid) -> UiResult<()> {
        if let Some(message) = self.messages.iter().find(|m| m.id == message_id) {
            if let Some(action) = message.context_actions.iter().find(|a| a.id == action_id) {
                match &action.action {
                    ActionType::Save { file_path } => {
                        // TODO: Implement save functionality
                    }
                    ActionType::Run { command } => {
                        // TODO: Implement run functionality
                    }
                    ActionType::OpenInEditor => {
                        // TODO: Switch to IDE view with code
                    }
                    ActionType::CopyToClipboard => {
                        // TODO: Copy to clipboard
                    }
                    ActionType::ShowDiff => {
                        // TODO: Show diff view
                    }
                }
            }
        }
        Ok(())
    }
}

impl Element for ChatInterface {
    fn id(&self) -> Uuid {
        self.base.id
    }
    
    fn type_name(&self) -> &str {
        "ChatInterface"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        // Layout messages vertically with proper spacing
        let mut current_y = 10.0;
        let padding = 10.0;
        let message_spacing = 15.0;
        
        for message in self.get_visible_messages() {
            // Calculate message height based on content
            let message_height = self.calculate_message_height(message);
            current_y += message_height + message_spacing;
        }
        
        Ok(LayoutResult::new(
            constraints.available_size,
            Vector2::new(0.0, current_y),
        ))
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::PointerDown { position, .. } => {
                if self.base.bounds.contains(*position) {
                    self.is_focused = true;
                    return InputResult { handled: EventHandled::Yes, request_focus: true };
                }
            }
            InputEvent::Scroll { delta, .. } => {
                if self.is_focused {
                    self.scroll_offset = (self.scroll_offset - delta.y).max(0.0);
                    self.base.mark_dirty();
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
            InputEvent::TextInput { text } => {
                if self.is_focused {
                    self.input_buffer.push_str(text);
                    self.base.mark_dirty();
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
            InputEvent::KeyDown { key, .. } => {
                use crate::input::Key;
                if self.is_focused {
                    match key {
                        Key::Enter => {
                            if !self.input_buffer.is_empty() {
                                let message = self.input_buffer.clone();
                                self.add_user_message(message);
                                self.input_buffer.clear();
                                self.base.mark_dirty();
                            }
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        Key::Backspace => {
                            self.input_buffer.pop();
                            self.base.mark_dirty();
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        InputResult { handled: EventHandled::No, request_focus: false }
    }
    
    fn render(&self, theme: &Theme) -> UiResult<RenderData> {
        let mut data = RenderData::new();
        
        // Render background
        data.add_quad(
            self.base.bounds.position,
            self.base.bounds.size,
            theme.colors.background,
        );
        
        // Render visible messages
        let mut y_offset = 10.0 - self.scroll_offset;
        for message in self.get_visible_messages() {
            // Skip messages above viewport
            if y_offset + 100.0 < 0.0 {
                y_offset += 100.0 + 15.0;
                continue;
            }
            
            // Stop rendering messages below viewport
            if y_offset > self.base.bounds.size.y {
                break;
            }
            
            // Render message bubble
            let bubble_color = match message.message_type {
                MessageType::User => theme.colors.primary,
                MessageType::Assistant => theme.colors.secondary,
                MessageType::System => theme.colors.surface,
            };
            
            let bubble_width = self.base.bounds.size.x * 0.8;
            let bubble_x = if matches!(message.message_type, MessageType::User) {
                self.base.bounds.size.x - bubble_width - 10.0
            } else {
                10.0
            };
            
            data.add_rounded_rect(
                Vector2::new(
                    self.base.bounds.position.x + bubble_x,
                    self.base.bounds.position.y + y_offset,
                ),
                Vector2::new(bubble_width, 100.0),
                8.0,
                bubble_color,
            );
            
            y_offset += 100.0 + 15.0;
        }
        
        Ok(data)
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Update animations if any
    }
    
    fn children(&self) -> &[ElementId] {
        &self.message_elements
    }
    
    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        &mut self.message_elements
    }
    
    fn is_dirty(&self) -> bool {
        self.base.dirty
    }
    
    fn mark_clean(&mut self) {
        self.base.dirty = false;
    }
    
    fn mark_dirty(&mut self) {
        self.base.dirty = true;
    }
    
    fn bounds(&self) -> ElementBounds {
        self.base.bounds
    }
    
    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.base.bounds = bounds;
        self.base.mark_dirty();
    }
    
    fn is_visible(&self) -> bool {
        self.base.visible
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.base.visible = visible;
        self.base.mark_dirty();
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ChatInterface {
    fn calculate_message_height(&self, message: &ChatMessage) -> f32 {
        let base_height = 60.0;
        let code_block_height = message.code_blocks.len() as f32 * 150.0;
        base_height + code_block_height
    }
}