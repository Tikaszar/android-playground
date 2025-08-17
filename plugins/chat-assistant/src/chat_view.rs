use playground_ui::{
    element::{Element, ElementId, ElementBounds},
    input::{InputEvent, InputResult, EventHandled, Key},
    layout::{LayoutConstraints, LayoutResult},
    rendering::RenderData,
    theme::Theme,
    UiResult,
};
use nalgebra::{Vector2, Vector4};
use uuid::Uuid;
use std::collections::VecDeque;

/// A chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: Uuid,
    pub sender: MessageSender,
    pub content: String,
    pub timestamp: std::time::SystemTime,
    pub code_blocks: Vec<CodeBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSender {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Chat interface for conversational IDE
pub struct ChatView {
    id: Uuid,
    position: Vector2<f32>,
    size: Vector2<f32>,
    messages: VecDeque<ChatMessage>,
    input_buffer: String,
    scroll_offset: f32,
    message_spacing: f32,
    bubble_padding: f32,
    max_bubble_width: f32,
    theme: Theme,
    dirty: bool,
    visible: bool,
    children: Vec<ElementId>,
    is_typing: bool,
    cursor_position: usize,
}

impl ChatView {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            position: Vector2::zeros(),
            size: Vector2::zeros(),
            messages: VecDeque::new(),
            input_buffer: String::new(),
            scroll_offset: 0.0,
            message_spacing: 10.0,
            bubble_padding: 12.0,
            max_bubble_width: 600.0,
            theme: Theme::dark(),
            dirty: true,
            visible: true,
            children: Vec::new(),
            is_typing: false,
            cursor_position: 0,
        }
    }

    pub fn add_message(&mut self, sender: MessageSender, content: String) {
        let code_blocks = self.extract_code_blocks(&content);
        
        let message = ChatMessage {
            id: Uuid::new_v4(),
            sender,
            content,
            timestamp: std::time::SystemTime::now(),
            code_blocks,
        };
        
        self.messages.push_back(message);
        self.scroll_to_bottom();
        self.dirty = true;
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0.0;
        self.dirty = true;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.dirty = true;
    }

    fn extract_code_blocks(&self, content: &str) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            if lines[i].starts_with("```") {
                let language = lines[i].trim_start_matches("```").trim().to_string();
                let start_line = i;
                i += 1;
                
                let mut code_content = String::new();
                while i < lines.len() && !lines[i].starts_with("```") {
                    if !code_content.is_empty() {
                        code_content.push('\n');
                    }
                    code_content.push_str(lines[i]);
                    i += 1;
                }
                
                if i < lines.len() {
                    blocks.push(CodeBlock {
                        language: if language.is_empty() { "text".to_string() } else { language },
                        content: code_content,
                        start_line,
                        end_line: i,
                    });
                }
            }
            i += 1;
        }
        
        blocks
    }

    fn scroll_to_bottom(&mut self) {
        let content_height = self.calculate_content_height();
        self.scroll_offset = (content_height - self.size.y).max(0.0);
    }

    fn calculate_content_height(&self) -> f32 {
        let mut height = 0.0;
        
        for message in &self.messages {
            height += self.calculate_message_height(message);
            height += self.message_spacing;
        }
        
        height += 60.0; // Input area height
        height
    }

    fn calculate_message_height(&self, message: &ChatMessage) -> f32 {
        // Simplified height calculation
        let lines = message.content.lines().count();
        let line_height = 20.0;
        let bubble_height = lines as f32 * line_height + self.bubble_padding * 2.0;
        bubble_height.max(40.0)
    }

    fn handle_key_input(&mut self, key: Key) -> InputResult {
        match key {
            Key::Enter => {
                if !self.input_buffer.is_empty() {
                    let content = self.input_buffer.clone();
                    self.input_buffer.clear();
                    self.cursor_position = 0;
                    self.add_message(MessageSender::User, content);
                    self.dirty = true;
                }
            }
            Key::Backspace => {
                if self.cursor_position > 0 {
                    self.input_buffer.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.dirty = true;
                }
            }
            Key::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.dirty = true;
                }
            }
            Key::Right => {
                if self.cursor_position < self.input_buffer.len() {
                    self.cursor_position += 1;
                    self.dirty = true;
                }
            }
            _ => {
                if let Some(ch) = key_to_char(key) {
                    self.input_buffer.insert(self.cursor_position, ch);
                    self.cursor_position += 1;
                    self.dirty = true;
                }
            }
        }
        
        InputResult { handled: EventHandled::Yes, request_focus: true }
    }

    fn render_messages(&self, data: &mut RenderData) {
        let mut y = -self.scroll_offset;
        
        for message in &self.messages {
            if y + self.calculate_message_height(message) < 0.0 {
                y += self.calculate_message_height(message) + self.message_spacing;
                continue;
            }
            
            if y > self.size.y {
                break;
            }
            
            self.render_message(data, message, y);
            y += self.calculate_message_height(message) + self.message_spacing;
        }
    }

    fn render_message(&self, data: &mut RenderData, message: &ChatMessage, y: f32) {
        let is_user = message.sender == MessageSender::User;
        let bubble_width = self.max_bubble_width.min(self.size.x - 40.0);
        let bubble_height = self.calculate_message_height(message);
        
        let x = if is_user {
            self.size.x - bubble_width - 20.0
        } else {
            20.0
        };
        
        let bubble_color = if is_user {
            Vector4::new(0.2, 0.3, 0.5, 1.0)
        } else {
            Vector4::new(0.25, 0.25, 0.25, 1.0)
        };
        
        // Render bubble background
        data.add_quad(
            Vector2::new(self.position.x + x, self.position.y + y),
            Vector2::new(bubble_width, bubble_height),
            bubble_color,
        );
        
        // Render code blocks with different background
        for block in &message.code_blocks {
            let block_y = y + (block.start_line as f32 * 20.0);
            let block_height = ((block.end_line - block.start_line) as f32 * 20.0);
            
            data.add_quad(
                Vector2::new(self.position.x + x + 10.0, self.position.y + block_y),
                Vector2::new(bubble_width - 20.0, block_height),
                Vector4::new(0.1, 0.1, 0.1, 1.0),
            );
        }
    }

    fn render_input_area(&self, data: &mut RenderData) {
        let input_y = self.size.y - 60.0;
        
        // Input background
        data.add_quad(
            Vector2::new(self.position.x, self.position.y + input_y),
            Vector2::new(self.size.x, 60.0),
            Vector4::new(0.15, 0.15, 0.15, 1.0),
        );
        
        // Input field
        data.add_quad(
            Vector2::new(self.position.x + 10.0, self.position.y + input_y + 10.0),
            Vector2::new(self.size.x - 20.0, 40.0),
            Vector4::new(0.2, 0.2, 0.2, 1.0),
        );
        
        // Cursor
        if self.is_typing {
            let cursor_x = 15.0 + (self.cursor_position as f32 * 8.0);
            data.add_quad(
                Vector2::new(self.position.x + cursor_x, self.position.y + input_y + 15.0),
                Vector2::new(2.0, 30.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
            );
        }
    }
}

impl Element for ChatView {
    fn id(&self) -> Uuid {
        self.id
    }

    fn type_name(&self) -> &str {
        "ChatView"
    }

    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        self.size = constraints.available_size;
        Ok(LayoutResult::new(self.size, self.position))
    }

    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::KeyDown { key, .. } => {
                self.is_typing = true;
                self.handle_key_input(*key)
            }
            InputEvent::PointerDown { position, .. } => {
                let input_y = self.size.y - 60.0;
                if position.y >= self.position.y + input_y {
                    self.is_typing = true;
                    InputResult { handled: EventHandled::Yes, request_focus: true }
                } else {
                    self.is_typing = false;
                    InputResult { handled: EventHandled::No, request_focus: false }
                }
            }
            InputEvent::Scroll { delta, .. } => {
                let content_height = self.calculate_content_height();
                self.scroll_offset = (self.scroll_offset - delta.y * 20.0)
                    .max(0.0)
                    .min((content_height - self.size.y).max(0.0));
                self.dirty = true;
                InputResult { handled: EventHandled::Yes, request_focus: false }
            }
            _ => InputResult { handled: EventHandled::No, request_focus: false },
        }
    }

    fn render(&self, _theme: &Theme) -> UiResult<RenderData> {
        let mut data = RenderData::new();
        
        // Background
        data.add_quad(
            self.position,
            self.size,
            self.theme.colors.background,
        );
        
        // Messages
        self.render_messages(&mut data);
        
        // Input area
        self.render_input_area(&mut data);
        
        Ok(data)
    }

    fn update(&mut self, _delta_time: f32) {
        // Could add typing indicator animation here
    }

    fn children(&self) -> &[ElementId] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<ElementId> {
        &mut self.children
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn bounds(&self) -> ElementBounds {
        ElementBounds {
            position: self.position,
            size: self.size,
        }
    }

    fn set_bounds(&mut self, bounds: ElementBounds) {
        self.position = bounds.position;
        self.size = bounds.size;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn key_to_char(key: Key) -> Option<char> {
    match key {
        Key::A => Some('a'),
        Key::B => Some('b'),
        Key::C => Some('c'),
        Key::D => Some('d'),
        Key::E => Some('e'),
        Key::F => Some('f'),
        Key::G => Some('g'),
        Key::H => Some('h'),
        Key::I => Some('i'),
        Key::J => Some('j'),
        Key::K => Some('k'),
        Key::L => Some('l'),
        Key::M => Some('m'),
        Key::N => Some('n'),
        Key::O => Some('o'),
        Key::P => Some('p'),
        Key::Q => Some('q'),
        Key::R => Some('r'),
        Key::S => Some('s'),
        Key::T => Some('t'),
        Key::U => Some('u'),
        Key::V => Some('v'),
        Key::W => Some('w'),
        Key::X => Some('x'),
        Key::Y => Some('y'),
        Key::Z => Some('z'),
        Key::Space => Some(' '),
        _ => None,
    }
}