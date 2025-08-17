use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Position in a text document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// Text change in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChange {
    pub range: (Position, Position),
    pub text: String,
}

/// Diagnostic severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: (Position, Position),
    pub severity: Severity,
    pub message: String,
    pub source: String,
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
    pub insert_text: String,
}

/// Theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: std::collections::HashMap<String, String>,
}

/// Messages that plugins can send to each other
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginMessage {
    // Editor → File Browser
    FileOpened { path: String },
    FileClosed { path: String },
    FileSaved { path: String },
    
    // File Browser → Editor
    OpenFile { path: String, content: String },
    CreateFile { path: String },
    DeleteFile { path: String },
    RenameFile { old_path: String, new_path: String },
    
    // Editor → LSP
    TextChanged { path: String, version: u32, changes: Vec<TextChange> },
    RequestCompletion { path: String, position: Position },
    RequestHover { path: String, position: Position },
    RequestDefinition { path: String, position: Position },
    RequestReferences { path: String, position: Position },
    
    // LSP → Editor
    PublishDiagnostics { path: String, diagnostics: Vec<Diagnostic> },
    CompletionResult { request_id: Uuid, items: Vec<CompletionItem> },
    HoverResult { request_id: Uuid, content: String },
    DefinitionResult { request_id: Uuid, locations: Vec<(String, Position)> },
    ReferencesResult { request_id: Uuid, locations: Vec<(String, Position)> },
    
    // Editor → Debugger
    ToggleBreakpoint { path: String, line: u32 },
    StartDebugging { program: String, args: Vec<String> },
    StepOver,
    StepInto,
    StepOut,
    Continue,
    Pause,
    Stop,
    
    // Debugger → Editor
    BreakpointHit { path: String, line: u32 },
    StackTrace { frames: Vec<String> },
    Variables { locals: Vec<(String, String)> },
    
    // Terminal → Editor
    OpenInEditor { path: String, line: Option<u32> },
    RunCommand { command: String },
    
    // Editor → Terminal
    RunFile { path: String },
    RunSelection { code: String },
    
    // Theme → All
    ThemeChanged { theme: Theme },
    FontChanged { font_family: String, font_size: f32 },
    
    // Chat → Editor
    InsertCode { code: String, position: Option<Position> },
    ReplaceSelection { code: String },
    ExplainSelection { selection: String },
    
    // Editor → Chat
    SelectionChanged { selection: Option<String> },
    FileContextChanged { path: String, content: String },
    
    // Version Control → Editor
    FileModified { path: String, status: String },
    ShowDiff { path: String, diff: String },
    
    // Editor → Version Control
    CommitFile { path: String, message: String },
    RevertFile { path: String },
    
    // Generic
    PluginReady { plugin_id: Uuid, name: String },
    PluginError { plugin_id: Uuid, error: String },
    RequestFocus { plugin_id: Uuid },
}

/// Message envelope with routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Option<Uuid>, // None = broadcast
    pub message: PluginMessage,
    pub timestamp: u64,
}