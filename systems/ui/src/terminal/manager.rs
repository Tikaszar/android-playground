use crate::error::{UiError, UiResult};
use std::collections::HashMap;
use uuid::Uuid;
use super::emulator::Terminal;

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