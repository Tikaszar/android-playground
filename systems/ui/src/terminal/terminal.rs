//! Terminal implementation for Termux integration

use crate::element::{Element, ElementBase, ElementBounds, ElementId};
use crate::input::{InputEvent, InputResult, EventHandled, Key};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use crate::theme::Theme;
use crate::error::{UiError, UiResult};
use crate::terminal::connection::{TerminalConnection, TerminalState as ConnectionState};
use crate::system::UiSystem;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Terminal line with styling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub text: String,
    pub is_input: bool,
    pub timestamp: u64,
}

/// ANSI color codes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

/// Terminal state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TerminalState {
    Ready,
    Running,
    Blocked,
    Disconnected,
}

// Terminal connection is now handled via the connection module
// which uses core/server channels instead of direct WebSocket

/// Terminal for interacting with Termux
pub struct Terminal {
    base: ElementBase,
    lines: VecDeque<TerminalLine>,
    max_lines: usize,
    input_buffer: String,
    cursor_position: usize,
    scroll_offset: f32,
    history: Vec<String>,
    history_index: Option<usize>,
    connection: Option<Arc<TerminalConnection>>,
    prompt: String,
    is_focused: bool,
    show_cursor: bool,
    cursor_blink_timer: f32,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            base: ElementBase::new(),
            lines: VecDeque::new(),
            max_lines: 1000,
            input_buffer: String::new(),
            cursor_position: 0,
            scroll_offset: 0.0,
            history: Vec::new(),
            history_index: None,
            connection: None,
            prompt: "$ ".to_string(),
            is_focused: false,
            show_cursor: true,
            cursor_blink_timer: 0.0,
        }
    }
    
    pub async fn connect_with_system(&mut self, ui_system: Arc<RwLock<UiSystem>>) -> UiResult<()> {
        let connection = Arc::new(TerminalConnection::new(ui_system));
        connection.connect(None, None).await?;
        self.connection = Some(connection);
        
        // Add connection message
        self.add_line(TerminalLine {
            text: "Connected to Termux terminal via core/server channels".to_string(),
            is_input: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        
        Ok(())
    }
    
    pub fn add_line(&mut self, line: TerminalLine) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
        self.base.mark_dirty();
        
        // Auto-scroll to bottom
        self.scroll_to_bottom();
    }
    
    pub fn add_output(&mut self, text: String) {
        // Split text by newlines and add each as a separate line
        for line in text.lines() {
            self.add_line(TerminalLine {
                text: line.to_string(),
                is_input: false,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
    }
    
    pub async fn execute_command(&mut self, command: String) -> UiResult<()> {
        // Add command to history
        self.history.push(command.clone());
        self.history_index = None;
        
        // Display command in terminal
        self.add_line(TerminalLine {
            text: format!("{}{}", self.prompt, command),
            is_input: true,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
        
        // Send to Termux if connected
        if let Some(connection) = &self.connection {
            connection.send_input(format!("{}\n", command)).await?;
        } else {
            // Fallback for when not connected
            self.add_output("Terminal not connected. Use 'connect' to establish connection.".to_string());
        }
        
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0.0;
        self.base.mark_dirty();
    }
    
    pub fn scroll_to_bottom(&mut self) {
        let line_height = 20.0;
        let total_height = self.lines.len() as f32 * line_height;
        let viewport_height = self.base.bounds.size.y;
        self.scroll_offset = (total_height - viewport_height).max(0.0);
    }
    
    fn handle_key_input(&mut self, key: Key) -> bool {
        match key {
            Key::Enter => {
                if !self.input_buffer.is_empty() {
                    let command = self.input_buffer.clone();
                    self.input_buffer.clear();
                    self.cursor_position = 0;
                    
                    // Execute command asynchronously
                    // TODO: Handle async execution properly
                    tokio::spawn(async move {
                        // Execute command
                    });
                }
                true
            }
            Key::Backspace => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
                true
            }
            Key::Delete => {
                if self.cursor_position < self.input_buffer.len() {
                    self.input_buffer.remove(self.cursor_position);
                }
                true
            }
            Key::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                true
            }
            Key::Right => {
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                }
                true
            }
            Key::Up => {
                // Navigate history backwards
                if !self.history.is_empty() {
                    if let Some(idx) = self.history_index {
                        if idx > 0 {
                            self.history_index = Some(idx - 1);
                            self.input_buffer = self.history[idx - 1].clone();
                            self.cursor_position = self.input_buffer.len();
                        }
                    } else {
                        let idx = self.history.len() - 1;
                        self.history_index = Some(idx);
                        self.input_buffer = self.history[idx].clone();
                        self.cursor_position = self.input_buffer.len();
                    }
                }
                true
            }
            Key::Down => {
                // Navigate history forwards
                if let Some(idx) = self.history_index {
                    if idx < self.history.len() - 1 {
                        self.history_index = Some(idx + 1);
                        self.input_buffer = self.history[idx + 1].clone();
                        self.cursor_position = self.input_buffer.len();
                    } else {
                        self.history_index = None;
                        self.input_buffer.clear();
                        self.cursor_position = 0;
                    }
                }
                true
            }
            Key::Home => {
                self.cursor_position = 0;
                true
            }
            Key::End => {
                self.cursor_position = self.input_buffer.len();
                true
            }
            _ => false,
        }
    }
    
    fn parse_ansi_color(&self, text: &str) -> Vec<(String, AnsiColor)> {
        // TODO: Implement ANSI color parsing
        vec![(text.to_string(), AnsiColor::White)]
    }
}

impl Element for Terminal {
    fn id(&self) -> Uuid {
        self.base.id
    }
    
    fn type_name(&self) -> &str {
        "Terminal"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        Ok(LayoutResult::new(
            constraints.available_size,
            Vector2::zeros(),
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
            InputEvent::KeyDown { key, modifiers } => {
                if self.is_focused {
                    // Handle Ctrl+C for interrupt
                    if modifiers.control && *key == Key::C {
                        self.add_output("^C".to_string());
                        self.input_buffer.clear();
                        self.cursor_position = 0;
                        return InputResult { handled: EventHandled::Yes, request_focus: false };
                    }
                    
                    // Handle Ctrl+L for clear
                    if modifiers.control && *key == Key::L {
                        self.clear();
                        return InputResult { handled: EventHandled::Yes, request_focus: false };
                    }
                    
                    if self.handle_key_input(*key) {
                        self.base.mark_dirty();
                        return InputResult { handled: EventHandled::Yes, request_focus: false };
                    }
                }
            }
            InputEvent::TextInput { text } => {
                if self.is_focused {
                    self.input_buffer.insert_str(self.cursor_position, text);
                    self.cursor_position += text.len();
                    self.base.mark_dirty();
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
            InputEvent::Scroll { delta, .. } => {
                if self.is_focused {
                    self.scroll_offset = (self.scroll_offset - delta.y * 20.0).max(0.0);
                    self.base.mark_dirty();
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
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
            nalgebra::Vector4::new(0.05, 0.05, 0.05, 1.0), // Dark terminal background
        );
        
        let line_height = 20.0;
        let char_width = 8.0;
        let padding = 5.0;
        
        // Calculate visible lines
        let visible_start = (self.scroll_offset / line_height) as usize;
        let visible_count = (self.base.bounds.size.y / line_height).ceil() as usize;
        let visible_end = (visible_start + visible_count).min(self.lines.len());
        
        // Render visible lines
        for (idx, line_idx) in (visible_start..visible_end).enumerate() {
            let line = &self.lines[line_idx];
            let y_pos = self.base.bounds.position.y + (idx as f32 * line_height) + padding;
            
            let text_color = if line.is_input {
                nalgebra::Vector4::new(0.5, 1.0, 0.5, 1.0) // Green for input
            } else {
                nalgebra::Vector4::new(0.9, 0.9, 0.9, 1.0) // White for output
            };
            
            data.add_text(
                Vector2::new(self.base.bounds.position.x + padding, y_pos),
                &line.text,
                14.0,
                text_color,
            );
        }
        
        // Render current input line
        let input_y = self.base.bounds.position.y + self.base.bounds.size.y - line_height - padding;
        data.add_text(
            Vector2::new(self.base.bounds.position.x + padding, input_y),
            &self.prompt,
            14.0,
            nalgebra::Vector4::new(0.5, 1.0, 0.5, 1.0),
        );
        
        data.add_text(
            Vector2::new(self.base.bounds.position.x + padding + (self.prompt.len() as f32 * char_width), input_y),
            &self.input_buffer,
            14.0,
            nalgebra::Vector4::new(1.0, 1.0, 1.0, 1.0),
        );
        
        // Render cursor
        if self.is_focused && self.show_cursor {
            let cursor_x = self.base.bounds.position.x + padding + 
                          ((self.prompt.len() + self.cursor_position) as f32 * char_width);
            data.add_quad(
                Vector2::new(cursor_x, input_y - 2.0),
                Vector2::new(char_width, line_height),
                nalgebra::Vector4::new(1.0, 1.0, 1.0, 0.5),
            );
        }
        
        Ok(data)
    }
    
    fn update(&mut self, delta_time: f32) {
        // Cursor blink animation
        self.cursor_blink_timer += delta_time;
        if self.cursor_blink_timer > 0.5 {
            self.show_cursor = !self.show_cursor;
            self.cursor_blink_timer = 0.0;
            self.base.mark_dirty();
        }
        
        // Check for terminal output
        if let Some(conn) = &self.connection {
            // TODO: Poll for output asynchronously
        }
    }
    
    fn children(&self) -> &[ElementId] {
        &self.base.children
    }
    
    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        &mut self.base.children
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