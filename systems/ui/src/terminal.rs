use playground_core_ecs::EntityId;
use crate::error::{UiError, UiResult};
use std::collections::HashMap;
use uuid::Uuid;

pub struct TerminalManager {
    terminals: HashMap<Uuid, Terminal>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminals: HashMap::new(),
        }
    }
    
    pub async fn create_terminal(&mut self, id: Uuid) -> UiResult<()> {
        let terminal = Terminal::new(id);
        self.terminals.insert(id, terminal);
        Ok(())
    }
    
    pub async fn destroy_terminal(&mut self, id: Uuid) -> UiResult<()> {
        self.terminals.remove(&id);
        Ok(())
    }
    
    pub async fn write_to_terminal(&mut self, id: Uuid, text: String) -> UiResult<()> {
        if let Some(terminal) = self.terminals.get_mut(&id) {
            terminal.write(text);
            Ok(())
        } else {
            Err(UiError::TerminalError(format!("Terminal {} not found", id)))
        }
    }
    
    pub async fn read_from_terminal(&self, id: Uuid) -> UiResult<Vec<String>> {
        if let Some(terminal) = self.terminals.get(&id) {
            Ok(terminal.read())
        } else {
            Err(UiError::TerminalError(format!("Terminal {} not found", id)))
        }
    }
}

struct Terminal {
    id: Uuid,
    lines: Vec<String>,
    cursor_position: (usize, usize),
    max_lines: usize,
}

impl Terminal {
    fn new(id: Uuid) -> Self {
        Self {
            id,
            lines: vec![String::new()],
            cursor_position: (0, 0),
            max_lines: 1000,
        }
    }
    
    fn write(&mut self, text: String) {
        // Simple terminal emulation
        for ch in text.chars() {
            match ch {
                '\n' => {
                    self.lines.push(String::new());
                    self.cursor_position.0 += 1;
                    self.cursor_position.1 = 0;
                    
                    // Limit lines
                    if self.lines.len() > self.max_lines {
                        self.lines.remove(0);
                        if self.cursor_position.0 > 0 {
                            self.cursor_position.0 -= 1;
                        }
                    }
                }
                '\r' => {
                    self.cursor_position.1 = 0;
                }
                '\t' => {
                    // Tab to next 4-space boundary
                    let spaces = 4 - (self.cursor_position.1 % 4);
                    for _ in 0..spaces {
                        if let Some(line) = self.lines.get_mut(self.cursor_position.0) {
                            line.push(' ');
                        }
                        self.cursor_position.1 += 1;
                    }
                }
                ch => {
                    if let Some(line) = self.lines.get_mut(self.cursor_position.0) {
                        if self.cursor_position.1 >= line.len() {
                            line.push(ch);
                        } else {
                            line.insert(self.cursor_position.1, ch);
                        }
                        self.cursor_position.1 += 1;
                    }
                }
            }
        }
    }
    
    fn read(&self) -> Vec<String> {
        self.lines.clone()
    }
}