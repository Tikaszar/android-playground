use serde::{Deserialize, Serialize};
use playground_ui::input::Key;

/// Vim commands that can be executed
#[derive(Debug, Clone, PartialEq)]
pub enum VimCommand {
    None,
    Move(Direction),
    Insert,
    Normal,
    Visual,
    Delete(Motion),
    Yank(Motion),
    Paste,
    MoveLeft(u32),
    MoveRight(u32),
    MoveUp(u32),
    MoveDown(u32),
    MoveWordForward(u32),
    MoveWordBackward(u32),
    MoveWordEnd(u32),
    MoveLineStart,
    MoveLineEnd,
    GoToLine(usize),
    GoToFirstLine,
    EnterInsertMode,
    ExitInsertMode,
    AppendMode,
    OpenLineBelow,
    OpenLineAbove,
    EnterVisualMode,
    EnterVisualLineMode,
    EnterReplaceMode,
    ExitVisualMode,
    ExitReplaceMode,
    EnterCommandMode,
    ExitCommandMode,
    DeleteChar(u32),
    DeleteLine(u32),
    DeleteWord(u32),
    DeleteToLineEnd,
    DeleteToLineStart,
    DeleteSelection,
    YankLine(u32),
    YankWord(u32),
    YankSelection,
    ChangeLine(u32),
    ChangeWord(u32),
    ChangeSelection,
    PasteAfter,
    PasteBefore,
    InsertChar(char),
    ReplaceChar(char),
    ExtendSelectionLeft,
    ExtendSelectionRight,
    ExtendSelectionUp,
    ExtendSelectionDown,
    ExecuteCommand(String),
    Undo,
    Redo,
}

/// Movement directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Motion types for operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    Line,
    Word,
    Character,
    ToLineEnd,
    ToLineStart,
}

/// Vim mode states
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    VisualBlock,
    Command,
    Replace,
}

/// Vim state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimState {
    pub mode: VimMode,
    pub command_buffer: String,
    pub repeat_count: Option<u32>,
    pub pending_operator: Option<String>,
    pub last_command: Option<String>,
    pub registers: std::collections::HashMap<char, String>,
    pub marks: std::collections::HashMap<char, (usize, usize)>, // line, column
}

impl Default for VimState {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            command_buffer: String::new(),
            repeat_count: None,
            pending_operator: None,
            last_command: None,
            registers: std::collections::HashMap::new(),
            marks: std::collections::HashMap::new(),
        }
    }
}

impl VimState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn mode(&self) -> VimMode {
        self.mode
    }
    
    pub fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
    }
    
    pub fn set_register(&mut self, register: char, content: String) {
        self.registers.insert(register, content);
    }
    
    pub fn get_register(&self, register: char) -> Option<String> {
        self.registers.get(&register).cloned()
    }
    
    pub fn handle_normal_key(&mut self, key: Key) -> Option<VimCommand> {
        match key {
            Key::H => Some(VimCommand::Move(Direction::Left)),
            Key::J => Some(VimCommand::Move(Direction::Down)),
            Key::K => Some(VimCommand::Move(Direction::Up)),
            Key::L => Some(VimCommand::Move(Direction::Right)),
            Key::I => Some(VimCommand::Insert),
            Key::V => Some(VimCommand::Visual),
            Key::Escape => Some(VimCommand::Normal),
            _ => None,
        }
    }
    
    pub fn handle_visual_key(&mut self, key: Key) -> Option<VimCommand> {
        match key {
            Key::H => Some(VimCommand::Move(Direction::Left)),
            Key::J => Some(VimCommand::Move(Direction::Down)),
            Key::K => Some(VimCommand::Move(Direction::Up)),
            Key::L => Some(VimCommand::Move(Direction::Right)),
            Key::D => Some(VimCommand::Delete(Motion::Character)),
            Key::Y => Some(VimCommand::Yank(Motion::Character)),
            Key::Escape => Some(VimCommand::Normal),
            _ => None,
        }
    }
    /// Process a key in vim mode
    pub fn process_key(&mut self, key: char) -> VimCommand {
        match self.mode {
            VimMode::Normal => self.process_normal_key(key),
            VimMode::Insert => self.process_insert_key(key),
            VimMode::Visual | VimMode::VisualLine | VimMode::VisualBlock => {
                self.process_visual_key(key)
            }
            VimMode::Command => self.process_command_key(key),
            VimMode::Replace => self.process_replace_key(key),
        }
    }
    
    fn process_normal_key(&mut self, key: char) -> VimCommand {
        // Handle repeat count
        if key.is_ascii_digit() && key != '0' || (self.repeat_count.is_some() && key == '0') {
            let count = self.repeat_count.unwrap_or(0) * 10 + (key as u32 - '0' as u32);
            self.repeat_count = Some(count);
            return VimCommand::None;
        }
        
        // Handle operators
        if let Some(op) = self.pending_operator.clone() {
            let command = self.complete_operator(&op, key);
            self.pending_operator = None;
            self.repeat_count = None;
            return command;
        }
        
        // Process normal mode commands
        let count = self.repeat_count.unwrap_or(1);
        self.repeat_count = None;
        
        match key {
            // Movement
            'h' => VimCommand::MoveLeft(count),
            'j' => VimCommand::MoveDown(count),
            'k' => VimCommand::MoveUp(count),
            'l' => VimCommand::MoveRight(count),
            'w' => VimCommand::MoveWordForward(count),
            'b' => VimCommand::MoveWordBackward(count),
            'e' => VimCommand::MoveWordEnd(count),
            '0' => VimCommand::MoveLineStart,
            '$' => VimCommand::MoveLineEnd,
            'g' => {
                self.pending_operator = Some("g".to_string());
                VimCommand::None
            }
            'G' => VimCommand::GoToLine(count as usize),
            
            // Mode changes
            'i' => {
                self.mode = VimMode::Insert;
                VimCommand::EnterInsertMode
            }
            'a' => {
                self.mode = VimMode::Insert;
                VimCommand::AppendMode
            }
            'o' => {
                self.mode = VimMode::Insert;
                VimCommand::OpenLineBelow
            }
            'O' => {
                self.mode = VimMode::Insert;
                VimCommand::OpenLineAbove
            }
            'v' => {
                self.mode = VimMode::Visual;
                VimCommand::EnterVisualMode
            }
            'V' => {
                self.mode = VimMode::VisualLine;
                VimCommand::EnterVisualLineMode
            }
            'R' => {
                self.mode = VimMode::Replace;
                VimCommand::EnterReplaceMode
            }
            ':' => {
                self.mode = VimMode::Command;
                self.command_buffer.clear();
                VimCommand::EnterCommandMode
            }
            
            // Editing
            'x' => VimCommand::DeleteChar(count),
            'd' => {
                self.pending_operator = Some("d".to_string());
                VimCommand::None
            }
            'y' => {
                self.pending_operator = Some("y".to_string());
                VimCommand::None
            }
            'c' => {
                self.pending_operator = Some("c".to_string());
                VimCommand::None
            }
            'p' => VimCommand::PasteAfter,
            'P' => VimCommand::PasteBefore,
            'u' => VimCommand::Undo,
            '\x12' => VimCommand::Redo, // Ctrl-R
            
            _ => VimCommand::None,
        }
    }
    
    fn process_insert_key(&mut self, key: char) -> VimCommand {
        if key == '\x1b' {
            // ESC key
            self.mode = VimMode::Normal;
            VimCommand::ExitInsertMode
        } else {
            VimCommand::InsertChar(key)
        }
    }
    
    fn process_visual_key(&mut self, key: char) -> VimCommand {
        match key {
            '\x1b' => {
                // ESC key
                self.mode = VimMode::Normal;
                VimCommand::ExitVisualMode
            }
            // Visual mode operations
            'd' => {
                self.mode = VimMode::Normal;
                VimCommand::DeleteSelection
            }
            'y' => {
                self.mode = VimMode::Normal;
                VimCommand::YankSelection
            }
            'c' => {
                self.mode = VimMode::Insert;
                VimCommand::ChangeSelection
            }
            // Movement in visual mode
            'h' => VimCommand::ExtendSelectionLeft,
            'j' => VimCommand::ExtendSelectionDown,
            'k' => VimCommand::ExtendSelectionUp,
            'l' => VimCommand::ExtendSelectionRight,
            _ => VimCommand::None,
        }
    }
    
    fn process_command_key(&mut self, key: char) -> VimCommand {
        if key == '\n' {
            // Enter key - execute command
            let command = self.command_buffer.clone();
            self.command_buffer.clear();
            self.mode = VimMode::Normal;
            VimCommand::ExecuteCommand(command)
        } else if key == '\x1b' {
            // ESC key - cancel
            self.command_buffer.clear();
            self.mode = VimMode::Normal;
            VimCommand::ExitCommandMode
        } else if key == '\x08' {
            // Backspace
            self.command_buffer.pop();
            VimCommand::None
        } else {
            self.command_buffer.push(key);
            VimCommand::None
        }
    }
    
    fn process_replace_key(&mut self, key: char) -> VimCommand {
        if key == '\x1b' {
            self.mode = VimMode::Normal;
            VimCommand::ExitReplaceMode
        } else {
            VimCommand::ReplaceChar(key)
        }
    }
    
    fn complete_operator(&mut self, operator: &str, motion: char) -> VimCommand {
        let count = self.repeat_count.unwrap_or(1);
        
        match (operator, motion) {
            // Delete operations
            ("d", 'd') => VimCommand::DeleteLine(count),
            ("d", 'w') => VimCommand::DeleteWord(count),
            ("d", '$') => VimCommand::DeleteToLineEnd,
            ("d", '0') => VimCommand::DeleteToLineStart,
            
            // Yank operations
            ("y", 'y') => VimCommand::YankLine(count),
            ("y", 'w') => VimCommand::YankWord(count),
            
            // Change operations
            ("c", 'c') => {
                self.mode = VimMode::Insert;
                VimCommand::ChangeLine(count)
            }
            ("c", 'w') => {
                self.mode = VimMode::Insert;
                VimCommand::ChangeWord(count)
            }
            
            // Go commands
            ("g", 'g') => VimCommand::GoToFirstLine,
            
            _ => VimCommand::None,
        }
    }
}

