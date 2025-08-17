use playground_ui::{
    element::{Element, ElementId, ElementBounds},
    input::{InputEvent, InputResult, EventHandled, Key, Modifiers},
    layout::{LayoutConstraints, LayoutResult},
    rendering::RenderData,
    theme::Theme,
    UiResult,
};
use nalgebra::{Vector2, Vector4};
use uuid::Uuid;
use std::collections::HashMap;

use crate::buffer::TextBuffer;
use crate::vim::{VimMode, VimState, VimCommand};

/// Visual representation of a code editor
pub struct EditorView {
    id: Uuid,
    position: Vector2<f32>,
    size: Vector2<f32>,
    buffer: TextBuffer,
    vim_state: VimState,
    cursor_position: (usize, usize), // (line, column)
    selection: Option<Selection>,
    scroll_offset: Vector2<f32>,
    line_height: f32,
    char_width: f32,
    show_line_numbers: bool,
    line_number_width: f32,
    theme: Theme,
    syntax_highlights: HashMap<usize, Vec<Highlight>>,
    dirty: bool,
    visible: bool,
    children: Vec<ElementId>,
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct Highlight {
    pub start_col: usize,
    pub end_col: usize,
    pub color: Vector4<f32>,
    pub style: HighlightStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightStyle {
    Normal,
    Bold,
    Italic,
    Underline,
}

impl EditorView {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            position: Vector2::zeros(),
            size: Vector2::zeros(),
            buffer: TextBuffer::new(),
            vim_state: VimState::new(),
            cursor_position: (0, 0),
            selection: None,
            scroll_offset: Vector2::zeros(),
            line_height: 20.0,
            char_width: 10.0,
            show_line_numbers: true,
            line_number_width: 50.0,
            theme: Theme::dark(),
            syntax_highlights: HashMap::new(),
            dirty: true,
            visible: true,
            children: Vec::new(),
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.buffer = TextBuffer::from_string(content);
        self.cursor_position = (0, 0);
        self.selection = None;
        self.scroll_offset = Vector2::zeros();
        self.dirty = true;
    }

    pub fn get_content(&self) -> String {
        self.buffer.to_string()
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.dirty = true;
    }

    pub fn set_syntax_highlights(&mut self, highlights: HashMap<usize, Vec<Highlight>>) {
        self.syntax_highlights = highlights;
        self.dirty = true;
    }

    fn handle_vim_command(&mut self, command: VimCommand) {
        match command {
            VimCommand::Move(direction) => {
                self.move_cursor(direction);
            }
            VimCommand::Insert => {
                self.vim_state.set_mode(VimMode::Insert);
            }
            VimCommand::Normal => {
                self.vim_state.set_mode(VimMode::Normal);
            }
            VimCommand::Visual => {
                self.vim_state.set_mode(VimMode::Visual);
                self.start_selection();
            }
            VimCommand::Delete(motion) => {
                self.delete_with_motion(motion);
            }
            VimCommand::Yank(motion) => {
                self.yank_with_motion(motion);
            }
            VimCommand::Paste => {
                self.paste();
            }
            // Handle all other command variants
            VimCommand::None | 
            VimCommand::MoveLeft(_) | VimCommand::MoveRight(_) |
            VimCommand::MoveUp(_) | VimCommand::MoveDown(_) |
            VimCommand::MoveWordForward(_) | VimCommand::MoveWordBackward(_) |
            VimCommand::MoveWordEnd(_) | VimCommand::MoveLineStart |
            VimCommand::MoveLineEnd | VimCommand::GoToLine(_) |
            VimCommand::GoToFirstLine | VimCommand::EnterInsertMode |
            VimCommand::ExitInsertMode | VimCommand::AppendMode |
            VimCommand::OpenLineBelow | VimCommand::OpenLineAbove |
            VimCommand::EnterVisualMode | VimCommand::EnterVisualLineMode |
            VimCommand::EnterReplaceMode | VimCommand::ExitVisualMode |
            VimCommand::ExitReplaceMode | VimCommand::EnterCommandMode |
            VimCommand::ExitCommandMode | VimCommand::DeleteChar(_) |
            VimCommand::DeleteLine(_) | VimCommand::DeleteWord(_) |
            VimCommand::DeleteToLineEnd | VimCommand::DeleteToLineStart |
            VimCommand::DeleteSelection | VimCommand::YankLine(_) |
            VimCommand::YankWord(_) | VimCommand::YankSelection |
            VimCommand::ChangeLine(_) | VimCommand::ChangeWord(_) |
            VimCommand::ChangeSelection | VimCommand::PasteAfter |
            VimCommand::PasteBefore | VimCommand::InsertChar(_) |
            VimCommand::ReplaceChar(_) | VimCommand::ExtendSelectionLeft |
            VimCommand::ExtendSelectionRight | VimCommand::ExtendSelectionUp |
            VimCommand::ExtendSelectionDown | VimCommand::ExecuteCommand(_) |
            VimCommand::Undo | VimCommand::Redo => {}
        }
        self.dirty = true;
    }

    fn move_cursor(&mut self, direction: crate::vim::Direction) {
        let (mut line, mut col) = self.cursor_position;
        let line_count = self.buffer.line_count();
        
        match direction {
            crate::vim::Direction::Up => {
                if line > 0 {
                    line -= 1;
                    col = col.min(self.buffer.line_length(line));
                }
            }
            crate::vim::Direction::Down => {
                if line < line_count - 1 {
                    line += 1;
                    col = col.min(self.buffer.line_length(line));
                }
            }
            crate::vim::Direction::Left => {
                if col > 0 {
                    col -= 1;
                } else if line > 0 {
                    line -= 1;
                    col = self.buffer.line_length(line);
                }
            }
            crate::vim::Direction::Right => {
                let line_len = self.buffer.line_length(line);
                if col < line_len {
                    col += 1;
                } else if line < line_count - 1 {
                    line += 1;
                    col = 0;
                }
            }
        }
        
        self.cursor_position = (line, col);
        self.ensure_cursor_visible();
        
        if self.vim_state.mode() == VimMode::Visual {
            self.update_selection();
        }
    }

    fn start_selection(&mut self) {
        self.selection = Some(Selection {
            start: self.cursor_position,
            end: self.cursor_position,
        });
    }

    fn update_selection(&mut self) {
        if let Some(ref mut selection) = self.selection {
            selection.end = self.cursor_position;
        }
    }

    fn delete_with_motion(&mut self, _motion: crate::vim::Motion) {
        // Simplified delete - just delete current line
        if self.buffer.line_count() > 1 {
            self.buffer.delete_line(self.cursor_position.0);
            if self.cursor_position.0 >= self.buffer.line_count() {
                self.cursor_position.0 = self.buffer.line_count() - 1;
            }
            self.cursor_position.1 = 0;
        }
    }

    fn yank_with_motion(&mut self, _motion: crate::vim::Motion) {
        // Simplified yank - just copy current line
        let line = self.buffer.get_line(self.cursor_position.0).to_string();
        self.vim_state.set_register('0', line);
    }

    fn paste(&mut self) {
        if let Some(content) = self.vim_state.get_register('0') {
            self.buffer.insert_line(self.cursor_position.0 + 1, content);
            self.cursor_position.0 += 1;
            self.cursor_position.1 = 0;
        }
    }

    fn ensure_cursor_visible(&mut self) {
        let cursor_y = self.cursor_position.0 as f32 * self.line_height;
        let cursor_x = self.cursor_position.1 as f32 * self.char_width + self.line_number_width;
        
        // Vertical scrolling
        if cursor_y < self.scroll_offset.y {
            self.scroll_offset.y = cursor_y;
        } else if cursor_y + self.line_height > self.scroll_offset.y + self.size.y {
            self.scroll_offset.y = cursor_y + self.line_height - self.size.y;
        }
        
        // Horizontal scrolling
        if cursor_x < self.scroll_offset.x + self.line_number_width {
            self.scroll_offset.x = cursor_x - self.line_number_width;
        } else if cursor_x > self.scroll_offset.x + self.size.x {
            self.scroll_offset.x = cursor_x - self.size.x;
        }
    }

    fn handle_key_input(&mut self, key: Key, modifiers: Modifiers) -> InputResult {
        match self.vim_state.mode() {
            VimMode::Normal => {
                if let Some(command) = self.vim_state.handle_normal_key(key) {
                    self.handle_vim_command(command);
                }
            }
            VimMode::Insert => {
                if key == Key::Escape {
                    self.vim_state.set_mode(VimMode::Normal);
                } else if let Some(ch) = key_to_char(key) {
                    self.buffer.insert_char(self.cursor_position.0, self.cursor_position.1, ch);
                    self.cursor_position.1 += 1;
                } else if key == Key::Backspace {
                    if self.cursor_position.1 > 0 {
                        self.cursor_position.1 -= 1;
                        self.buffer.delete_char(self.cursor_position.0, self.cursor_position.1);
                    }
                } else if key == Key::Enter {
                    self.buffer.split_line(self.cursor_position.0, self.cursor_position.1);
                    self.cursor_position.0 += 1;
                    self.cursor_position.1 = 0;
                }
                self.dirty = true;
            }
            VimMode::Visual => {
                if key == Key::Escape {
                    self.vim_state.set_mode(VimMode::Normal);
                    self.selection = None;
                } else if let Some(command) = self.vim_state.handle_visual_key(key) {
                    self.handle_vim_command(command);
                }
            }
            _ => {}
        }
        
        InputResult { handled: EventHandled::Yes, request_focus: true }
    }

    fn render_line_numbers(&self, data: &mut RenderData) {
        if !self.show_line_numbers {
            return;
        }
        
        // Background for line numbers
        data.add_quad(
            self.position,
            Vector2::new(self.line_number_width, self.size.y),
            Vector4::new(0.15, 0.15, 0.15, 1.0),
        );
        
        // Render line numbers (simplified - just colored rectangles for now)
        let start_line = (self.scroll_offset.y / self.line_height) as usize;
        let end_line = ((self.scroll_offset.y + self.size.y) / self.line_height) as usize + 1;
        
        for line in start_line..end_line.min(self.buffer.line_count()) {
            let y = line as f32 * self.line_height - self.scroll_offset.y;
            
            // Highlight current line number
            if line == self.cursor_position.0 {
                data.add_quad(
                    Vector2::new(self.position.x, self.position.y + y),
                    Vector2::new(self.line_number_width, self.line_height),
                    Vector4::new(0.2, 0.2, 0.2, 1.0),
                );
            }
        }
    }

    fn render_text(&self, data: &mut RenderData) {
        let start_line = (self.scroll_offset.y / self.line_height) as usize;
        let end_line = ((self.scroll_offset.y + self.size.y) / self.line_height) as usize + 1;
        
        for line in start_line..end_line.min(self.buffer.line_count()) {
            let y = line as f32 * self.line_height - self.scroll_offset.y;
            
            // Render selection if applicable
            if let Some(ref selection) = self.selection {
                if line >= selection.start.0.min(selection.end.0) &&
                   line <= selection.start.0.max(selection.end.0) {
                    data.add_quad(
                        Vector2::new(
                            self.position.x + self.line_number_width,
                            self.position.y + y
                        ),
                        Vector2::new(self.size.x - self.line_number_width, self.line_height),
                        Vector4::new(0.3, 0.3, 0.5, 0.5),
                    );
                }
            }
            
            // Text would be rendered here with proper font rendering
            // For now, we'll just skip actual text rendering
        }
    }

    fn render_cursor(&self, data: &mut RenderData) {
        let cursor_x = self.position.x + self.line_number_width + 
                      (self.cursor_position.1 as f32 * self.char_width) - self.scroll_offset.x;
        let cursor_y = self.position.y + 
                      (self.cursor_position.0 as f32 * self.line_height) - self.scroll_offset.y;
        
        let cursor_color = match self.vim_state.mode() {
            VimMode::Normal => Vector4::new(1.0, 1.0, 1.0, 1.0),
            VimMode::Insert => Vector4::new(0.0, 1.0, 0.0, 1.0),
            VimMode::Visual => Vector4::new(0.0, 0.5, 1.0, 1.0),
            _ => Vector4::new(0.5, 0.5, 0.5, 1.0),
        };
        
        let cursor_width = match self.vim_state.mode() {
            VimMode::Insert => 2.0,
            _ => self.char_width,
        };
        
        data.add_quad(
            Vector2::new(cursor_x, cursor_y),
            Vector2::new(cursor_width, self.line_height),
            cursor_color,
        );
    }
}

impl Element for EditorView {
    fn id(&self) -> Uuid {
        self.id
    }

    fn type_name(&self) -> &str {
        "EditorView"
    }

    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        self.size = constraints.available_size;
        Ok(LayoutResult::new(self.size, self.position))
    }

    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        match event {
            InputEvent::KeyDown { key, modifiers } => {
                self.handle_key_input(*key, *modifiers)
            }
            InputEvent::Scroll { delta, .. } => {
                self.scroll_offset.y = (self.scroll_offset.y - delta.y * 20.0)
                    .max(0.0)
                    .min((self.buffer.line_count() as f32 * self.line_height - self.size.y).max(0.0));
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
        
        // Render components
        self.render_line_numbers(&mut data);
        self.render_text(&mut data);
        self.render_cursor(&mut data);
        
        Ok(data)
    }

    fn update(&mut self, _delta_time: f32) {
        // Could add cursor blinking animation here
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