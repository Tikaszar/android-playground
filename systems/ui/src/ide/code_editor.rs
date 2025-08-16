//! Code editor component

use crate::element::{Element, ElementBase, ElementBounds, ElementId};
use crate::input::{InputEvent, InputResult, EventHandled};
use crate::layout::{LayoutConstraints, LayoutResult};
use crate::rendering::RenderData;
use crate::theme::Theme;
use crate::error::UiResult;
use nalgebra::{Vector2, Vector4};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::ops::Range;
use uuid::Uuid;

/// Cursor position in the editor
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

/// Selection range in the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

/// Syntax highlighting token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxToken {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TokenType {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Punctuation,
    Plain,
}

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
    Command,
}

/// Code editor with syntax highlighting
pub struct CodeEditor {
    base: ElementBase,
    content: Vec<String>,
    language: String,
    editable: bool,
    cursor: CursorPosition,
    selections: Vec<Selection>,
    mode: EditorMode,
    vim_enabled: bool,
    line_numbers: bool,
    focused_lines: Option<Range<usize>>,
    syntax_tokens: HashMap<usize, Vec<SyntaxToken>>,
    scroll_offset: Vector2<f32>,
    tab_size: usize,
    word_wrap: bool,
    diagnostics: Vec<Diagnostic>,
}

/// Diagnostic message (error, warning, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl CodeEditor {
    pub fn new() -> Self {
        Self {
            base: ElementBase::new(),
            content: vec![String::new()],
            language: "plain".to_string(),
            editable: true,
            cursor: CursorPosition { line: 0, column: 0 },
            selections: Vec::new(),
            mode: EditorMode::Normal,
            vim_enabled: false,
            line_numbers: true,
            focused_lines: None,
            syntax_tokens: HashMap::new(),
            scroll_offset: Vector2::zeros(),
            tab_size: 4,
            word_wrap: false,
            diagnostics: Vec::new(),
        }
    }
    
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content.lines().map(String::from).collect();
        if self.content.is_empty() {
            self.content.push(String::new());
        }
        self.update_syntax_highlighting();
        self
    }
    
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self.update_syntax_highlighting();
        self
    }
    
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }
    
    pub fn with_focused_lines(mut self, range: Range<usize>) -> Self {
        self.focused_lines = Some(range);
        self
    }
    
    pub fn enable_vim_mode(mut self) -> Self {
        self.vim_enabled = true;
        self.mode = EditorMode::Normal;
        self
    }
    
    pub fn get_content(&self) -> String {
        self.content.join("\n")
    }
    
    pub fn set_content(&mut self, content: String) {
        self.content = content.lines().map(String::from).collect();
        if self.content.is_empty() {
            self.content.push(String::new());
        }
        self.update_syntax_highlighting();
        self.base.mark_dirty();
    }
    
    pub fn insert_text(&mut self, text: &str) {
        if !self.editable {
            return;
        }
        
        let line = &mut self.content[self.cursor.line];
        line.insert_str(self.cursor.column, text);
        self.cursor.column += text.len();
        self.update_syntax_highlighting();
        self.base.mark_dirty();
    }
    
    pub fn delete_char(&mut self) {
        if !self.editable || self.cursor.column == 0 {
            return;
        }
        
        let line = &mut self.content[self.cursor.line];
        line.remove(self.cursor.column - 1);
        self.cursor.column -= 1;
        self.update_syntax_highlighting();
        self.base.mark_dirty();
    }
    
    pub fn new_line(&mut self) {
        if !self.editable {
            return;
        }
        
        let current_line = &self.content[self.cursor.line];
        let new_line = current_line[self.cursor.column..].to_string();
        self.content[self.cursor.line].truncate(self.cursor.column);
        self.content.insert(self.cursor.line + 1, new_line);
        self.cursor.line += 1;
        self.cursor.column = 0;
        self.update_syntax_highlighting();
        self.base.mark_dirty();
    }
    
    pub fn move_cursor(&mut self, line: isize, column: isize) {
        let new_line = (self.cursor.line as isize + line)
            .max(0)
            .min(self.content.len() as isize - 1) as usize;
        
        self.cursor.line = new_line;
        
        let line_len = self.content[new_line].len();
        let new_column = (self.cursor.column as isize + column)
            .max(0)
            .min(line_len as isize) as usize;
        
        self.cursor.column = new_column;
        self.base.mark_dirty();
    }
    
    pub fn add_selection(&mut self, start: CursorPosition, end: CursorPosition) {
        self.selections.push(Selection { start, end });
        self.base.mark_dirty();
    }
    
    pub fn clear_selections(&mut self) {
        self.selections.clear();
        self.base.mark_dirty();
    }
    
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
        self.base.mark_dirty();
    }
    
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
        self.base.mark_dirty();
    }
    
    fn update_syntax_highlighting(&mut self) {
        // TODO: Implement actual syntax highlighting based on language
        // For now, just mark keywords for Rust
        if self.language == "rust" {
            for (line_idx, line) in self.content.iter().enumerate() {
                let mut tokens = Vec::new();
                let keywords = ["fn", "let", "mut", "impl", "struct", "enum", "trait", 
                               "pub", "use", "mod", "if", "else", "match", "for", "while"];
                
                for keyword in &keywords {
                    let mut start = 0;
                    while let Some(pos) = line[start..].find(keyword) {
                        let abs_pos = start + pos;
                        tokens.push(SyntaxToken {
                            start: abs_pos,
                            end: abs_pos + keyword.len(),
                            token_type: TokenType::Keyword,
                        });
                        start = abs_pos + keyword.len();
                    }
                }
                
                self.syntax_tokens.insert(line_idx, tokens);
            }
        }
    }
    
}

impl Element for CodeEditor {
    fn id(&self) -> Uuid {
        self.base.id
    }
    
    fn type_name(&self) -> &str {
        "CodeEditor"
    }
    
    fn layout(&mut self, constraints: &LayoutConstraints) -> UiResult<LayoutResult> {
        let line_height = 20.0;
        let content_height = self.content.len() as f32 * line_height;
        let gutter_width = if self.line_numbers { 50.0 } else { 0.0 };
        
        Ok(LayoutResult::new(
            constraints.available_size,
            Vector2::new(gutter_width + 500.0, content_height),
        ))
    }
    
    fn handle_input(&mut self, event: &InputEvent) -> InputResult {
        use crate::input::Key;
        
        match event {
            InputEvent::PointerDown { position, .. } => {
                if self.base.bounds.contains(*position) {
                    // Calculate clicked line and column
                    let relative_pos = *position - self.base.bounds.position - self.scroll_offset;
                    let line = (relative_pos.y / 20.0) as usize;
                    if line < self.content.len() {
                        self.cursor.line = line;
                        // TODO: Calculate column based on text metrics
                        self.base.mark_dirty();
                    }
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
            InputEvent::KeyDown { key, .. } => {
                if self.vim_enabled {
                    // Handle vim mode keys
                    match self.mode {
                        EditorMode::Normal => match key {
                            Key::I => {
                                self.mode = EditorMode::Insert;
                                return InputResult { handled: EventHandled::Yes, request_focus: false };
                            }
                            Key::H => {
                                self.move_cursor(0, -1);
                                return InputResult { handled: EventHandled::Yes, request_focus: false };
                            }
                            Key::J => {
                                self.move_cursor(1, 0);
                                return InputResult { handled: EventHandled::Yes, request_focus: false };
                            }
                            Key::K => {
                                self.move_cursor(-1, 0);
                                return InputResult { handled: EventHandled::Yes, request_focus: false };
                            }
                            Key::L => {
                                self.move_cursor(0, 1);
                                return InputResult { handled: EventHandled::Yes, request_focus: false };
                            }
                            _ => {}
                        },
                        EditorMode::Insert => {
                            if *key == Key::Escape {
                                self.mode = EditorMode::Normal;
                                return InputResult { handled: EventHandled::Yes, request_focus: false };
                            }
                        }
                        _ => {}
                    }
                }
                
                if self.editable && (self.mode == EditorMode::Insert || !self.vim_enabled) {
                    match key {
                        Key::Enter => {
                            self.new_line();
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        Key::Backspace => {
                            self.delete_char();
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        Key::Left => {
                            self.move_cursor(0, -1);
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        Key::Right => {
                            self.move_cursor(0, 1);
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        Key::Up => {
                            self.move_cursor(-1, 0);
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        Key::Down => {
                            self.move_cursor(1, 0);
                            return InputResult { handled: EventHandled::Yes, request_focus: false };
                        }
                        _ => {}
                    }
                }
            }
            InputEvent::TextInput { text } => {
                if self.editable && (self.mode == EditorMode::Insert || !self.vim_enabled) {
                    self.insert_text(text);
                    return InputResult { handled: EventHandled::Yes, request_focus: false };
                }
            }
            InputEvent::Scroll { delta, .. } => {
                self.scroll_offset -= *delta * 20.0;
                self.scroll_offset.x = self.scroll_offset.x.max(0.0);
                self.scroll_offset.y = self.scroll_offset.y.max(0.0);
                self.base.mark_dirty();
                return InputResult { handled: EventHandled::Yes, request_focus: false };
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
            theme.colors.editor_background,
        );
        
        let line_height = 20.0;
        let char_width = 8.0;
        let gutter_width = if self.line_numbers { 50.0 } else { 0.0 };
        
        // Render visible lines
        let visible_start = (self.scroll_offset.y / line_height) as usize;
        let visible_end = ((self.scroll_offset.y + self.base.bounds.size.y) / line_height) as usize + 1;
        let visible_end = visible_end.min(self.content.len());
        
        for (idx, line_idx) in (visible_start..visible_end).enumerate() {
            let y_pos = self.base.bounds.position.y + (idx as f32 * line_height) - (self.scroll_offset.y % line_height);
            
            // Highlight focused lines
            if let Some(ref range) = self.focused_lines {
                if range.contains(&line_idx) {
                    data.add_quad(
                        Vector2::new(self.base.bounds.position.x, y_pos),
                        Vector2::new(self.base.bounds.size.x, line_height),
                        theme.colors.highlight,
                    );
                }
            }
            
            // Render line numbers
            if self.line_numbers {
                data.add_text(
                    Vector2::new(self.base.bounds.position.x + 5.0, y_pos + 15.0),
                    &format!("{:4}", line_idx + 1),
                    12.0,
                    theme.colors.line_number,
                );
            }
            
            // Render line content with syntax highlighting
            let line = &self.content[line_idx];
            let tokens = self.syntax_tokens.get(&line_idx);
            
            if let Some(tokens) = tokens {
                let mut last_end = 0;
                for token in tokens {
                    // Render plain text before token
                    if token.start > last_end {
                        let text = &line[last_end..token.start];
                        data.add_text(
                            Vector2::new(
                                self.base.bounds.position.x + gutter_width + (last_end as f32 * char_width),
                                y_pos + 15.0,
                            ),
                            text,
                            14.0,
                            theme.colors.text,
                        );
                    }
                    
                    // Render token with color
                    let token_color = match token.token_type {
                        TokenType::Keyword => theme.colors.keyword,
                        TokenType::String => theme.colors.string,
                        TokenType::Number => theme.colors.number,
                        TokenType::Comment => theme.colors.comment,
                        TokenType::Function => theme.colors.function,
                        TokenType::Type => theme.colors.type_color,
                        _ => theme.colors.text,
                    };
                    
                    let token_text = &line[token.start..token.end];
                    data.add_text(
                        Vector2::new(
                            self.base.bounds.position.x + gutter_width + (token.start as f32 * char_width),
                            y_pos + 15.0,
                        ),
                        token_text,
                        14.0,
                        token_color,
                    );
                    
                    last_end = token.end;
                }
                
                // Render remaining text
                if last_end < line.len() {
                    let text = &line[last_end..];
                    data.add_text(
                        Vector2::new(
                            self.base.bounds.position.x + gutter_width + (last_end as f32 * char_width),
                            y_pos + 15.0,
                        ),
                        text,
                        14.0,
                        theme.colors.text,
                    );
                }
            } else {
                // No syntax highlighting, render plain text
                data.add_text(
                    Vector2::new(self.base.bounds.position.x + gutter_width, y_pos + 15.0),
                    line,
                    14.0,
                    theme.colors.text,
                );
            }
            
            // Render diagnostics
            for diagnostic in &self.diagnostics {
                if diagnostic.line == line_idx {
                    let color = match diagnostic.severity {
                        DiagnosticSeverity::Error => theme.colors.error,
                        DiagnosticSeverity::Warning => theme.colors.warning,
                        _ => theme.colors.info,
                    };
                    
                    // Underline the problematic text
                    data.add_line(
                        Vector2::new(
                            self.base.bounds.position.x + gutter_width + (diagnostic.column as f32 * char_width),
                            y_pos + line_height - 2.0,
                        ),
                        Vector2::new(
                            self.base.bounds.position.x + gutter_width + ((diagnostic.column + 10) as f32 * char_width),
                            y_pos + line_height - 2.0,
                        ),
                        2.0,
                        color,
                    );
                }
            }
        }
        
        // Render cursor
        if self.editable {
            let cursor_y = self.base.bounds.position.y + (self.cursor.line as f32 * line_height) - self.scroll_offset.y;
            let cursor_x = self.base.bounds.position.x + gutter_width + (self.cursor.column as f32 * char_width) - self.scroll_offset.x;
            
            data.add_quad(
                Vector2::new(cursor_x, cursor_y),
                Vector2::new(2.0, line_height),
                theme.colors.cursor,
            );
        }
        
        // Render selections
        for selection in &self.selections {
            // TODO: Render selection rectangles
        }
        
        Ok(data)
    }
    
    fn update(&mut self, _delta_time: f32) {
        // Cursor blink animation could go here
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