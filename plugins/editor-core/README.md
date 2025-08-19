# Editor Core Plugin

A powerful, Vim-enabled code editor plugin that provides the foundational text editing capabilities for the Android Playground IDE, featuring multi-cursor support, syntax highlighting, and advanced buffer management.

## Overview

The Editor Core Plugin delivers a professional-grade text editing experience optimized for mobile development. It combines the power of Vim modal editing with modern IDE features like syntax highlighting, multi-cursor editing, and efficient text buffer management. The plugin integrates seamlessly with the UI system to provide visual editing components while maintaining a clean separation between logic and rendering.

## Architecture

```
editor-core/
├── plugin.rs      # Main plugin implementation and file management
├── state.rs       # Editor state persistence and management
├── buffer.rs      # Text buffer implementation with efficient operations
├── vim.rs         # Complete Vim emulation system
└── editor_view.rs # Visual representation and UI integration
```

## Core Features

### 1. Advanced Text Buffer Management

The `TextBuffer` provides efficient text manipulation with full undo/redo support:

```rust
use playground_plugins_editor_core::TextBuffer;

// Create a new buffer with content
let mut buffer = TextBuffer::with_path(
    "/src/main.rs".to_string(),
    "fn main() {\n    println!(\"Hello, world!\");\n}".to_string()
);

// Language auto-detection from file extension
assert_eq!(buffer.language, "rust");

// Line-based operations
buffer.insert_line(1, "    // This is a comment".to_string());
buffer.delete_line(3);

// Character operations
buffer.insert_char(0, 3, 'a');  // Insert 'a' at line 0, column 3
buffer.delete_char(0, 4);       // Delete character at line 0, column 4

// Range operations
buffer.insert(0, 0, "// File: main.rs\n");
buffer.delete(1, 0, 2, 10);  // Delete from line 1 col 0 to line 2 col 10

// Split and merge lines
buffer.split_line(1, 5);  // Split line 1 at column 5

// Get content
let full_text = buffer.get_text();
let line = buffer.get_line(0);
let line_count = buffer.line_count();
```

### 2. Complete Vim Mode Implementation

Full Vim emulation with support for all major modes and commands:

```rust
use playground_plugins_editor_core::{VimState, VimMode, VimCommand};

let mut vim_state = VimState::new();

// Mode transitions
vim_state.set_mode(VimMode::Normal);
vim_state.set_mode(VimMode::Insert);
vim_state.set_mode(VimMode::Visual);
vim_state.set_mode(VimMode::Command);
vim_state.set_mode(VimMode::Replace);

// Process key inputs
let command = vim_state.process_key('i');  // Enter insert mode
let command = vim_state.process_key('h');  // Move left in normal mode
let command = vim_state.process_key(':');  // Enter command mode

// Vim registers for copy/paste
vim_state.set_register('a', "copied text".to_string());
let content = vim_state.get_register('a');

// Marks for navigation
vim_state.marks.insert('a', (10, 5));  // Set mark 'a' at line 10, col 5
```

#### Supported Vim Commands

**Movement Commands:**
- `h/j/k/l` - Basic cursor movement
- `w/b/e` - Word-based movement (forward/backward/end)
- `0/$` - Line start/end
- `gg/G` - Document start/end
- `[count]G` - Go to line number

**Mode Changes:**
- `i` - Insert mode
- `a` - Append mode
- `o/O` - Open line below/above
- `v/V` - Visual/Visual line mode
- `R` - Replace mode
- `:` - Command mode
- `ESC` - Return to normal mode

**Editing Commands:**
- `x` - Delete character
- `dd` - Delete line
- `dw` - Delete word
- `d$` - Delete to line end
- `d0` - Delete to line start
- `yy` - Yank (copy) line
- `yw` - Yank word
- `p/P` - Paste after/before
- `cc` - Change line
- `cw` - Change word
- `u` - Undo
- `Ctrl-R` - Redo

**Visual Mode Operations:**
- `d` - Delete selection
- `y` - Yank selection
- `c` - Change selection
- Movement extends selection

**Command Mode:**
- `:w` - Save file
- `:q` - Quit
- `:wq` - Save and quit
- `:e <file>` - Open file
- `:[line]` - Go to line

### 3. Multi-Cursor Support

Edit multiple locations simultaneously:

```rust
use playground_plugins_editor_core::EditorCorePlugin;

let mut editor = EditorCorePlugin::new();

// Add multiple cursors
editor.add_cursor(5, 10).await;   // Line 5, column 10
editor.add_cursor(10, 10).await;  // Line 10, column 10
editor.add_cursor(15, 10).await;  // Line 15, column 10

// Operations apply to all cursors
// Type "const " at all cursor positions simultaneously

// Clear additional cursors
editor.clear_cursors().await;  // Keep only primary cursor
```

### 4. File Management

Efficient file handling with tab support:

```rust
// Open files
editor.open_file(
    "/src/main.rs".to_string(),
    file_content
).await;

// Switch between open files
let state = editor.save_state().await;
for (index, file) in state.open_files.iter().enumerate() {
    println!("{}: {} {}", 
        index, 
        file.path,
        if file.modified { "*" } else { "" }
    );
}

// Close files
editor.close_file("/src/old.rs").await;

// Check modifications
if state.open_files[0].modified {
    // Prompt to save
}
```

### 5. Syntax Highlighting

Language-aware syntax highlighting with automatic detection:

```rust
// Automatic language detection from file extension
let language = EditorCorePlugin::detect_language("main.rs");  // "rust"
let language = EditorCorePlugin::detect_language("app.js");   // "javascript"
let language = EditorCorePlugin::detect_language("config.json"); // "json"

// Supported languages
- Rust (.rs)
- JavaScript (.js)
- TypeScript (.ts)
- Python (.py)
- Go (.go)
- Java (.java)
- C/C++ (.c, .h, .cpp, .cc, .cxx)
- Markdown (.md)
- JSON (.json)
- TOML (.toml)
- YAML (.yaml, .yml)
```

### 6. Visual Editor Component

The `EditorView` provides a complete visual representation:

```rust
use playground_plugins_editor_core::EditorView;
use playground_systems_ui::theme::Theme;

let mut editor_view = EditorView::new();

// Configure appearance
editor_view.set_theme(Theme::dark());
editor_view.show_line_numbers = true;
editor_view.line_height = 20.0;
editor_view.char_width = 10.0;

// Set content
editor_view.set_content(file_content);

// Apply syntax highlighting
let highlights = generate_syntax_highlights(&content);
editor_view.set_syntax_highlights(highlights);

// Handle input events
editor_view.handle_input(&InputEvent::KeyDown { 
    key: Key::I, 
    modifiers: Modifiers::empty() 
});
```

### 7. State Persistence

Save and restore editor state across sessions:

```rust
use playground_plugins_editor_core::EditorState;

// Save current state
let state = editor.save_state().await;
let json = serde_json::to_string(&state)?;
std::fs::write("editor_state.json", json)?;

// Restore state
let json = std::fs::read_to_string("editor_state.json")?;
let state: EditorState = serde_json::from_str(&json)?;
editor.load_state(state);

// State includes:
// - Open files and their content
// - Active file index
// - Cursor positions for each file
// - Vim mode state
// - Modified flags
```

## Plugin Integration

### Channel Communication

The editor uses channel 1000 for IDE plugin communication:

```rust
// Register on channel 1000 (IDE plugins range)
let channel_id = networking.register_plugin("editor-core").await?;

// Message types
pub enum EditorMessage {
    OpenFile { path: String, content: String },
    SaveFile { path: String },
    CloseFile { path: String },
    GetContent { path: String },
    SetCursor { line: usize, column: usize },
    ToggleVimMode,
    Undo,
    Redo,
}
```

### Event Handling

```rust
#[async_trait]
impl Plugin for EditorCorePlugin {
    async fn on_event(&mut self, event: &Event) -> bool {
        match event.id.as_str() {
            "file:open" => {
                let path = event.data["path"].as_str().unwrap();
                let content = load_file(path).await;
                self.open_file(path.to_string(), content).await;
                true
            }
            "edit:save" => {
                self.save_current_file().await;
                true
            }
            "edit:vim_toggle" => {
                self.toggle_vim_mode().await;
                true
            }
            _ => false
        }
    }
}
```

## Usage Examples

### Complete Example: Creating a Code Editor

```rust
use playground_plugins_editor_core::{EditorCorePlugin, EditorView};
use playground_systems_ui::UiSystem;

// Initialize the editor plugin
let mut editor_plugin = EditorCorePlugin::new();
editor_plugin.on_load(&mut context).await?;

// Create visual component
let editor_view = EditorView::new();
ui_system.add_element(Box::new(editor_view)).await?;

// Open a file
let content = std::fs::read_to_string("/src/main.rs")?;
editor_plugin.open_file("/src/main.rs".to_string(), content).await;

// Enable Vim mode
editor_plugin.toggle_vim_mode().await;

// Handle user input (delegated from UI system)
loop {
    let event = ui_system.poll_event().await;
    if editor_plugin.on_event(&event).await {
        // Event was handled by editor
    }
}
```

### Example: Implementing Find and Replace

```rust
// Search for text
fn find_in_buffer(buffer: &TextBuffer, query: &str) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    for (line_num, line) in buffer.content.iter().enumerate() {
        for (col, _) in line.match_indices(query) {
            matches.push((line_num, col));
        }
    }
    matches
}

// Replace text
fn replace_in_buffer(
    buffer: &mut TextBuffer, 
    find: &str, 
    replace: &str
) {
    for line in &mut buffer.content {
        *line = line.replace(find, replace);
    }
    buffer.version += 1;
    buffer.modified = true;
}
```

### Example: Custom Vim Command

```rust
// Extend Vim command processing
fn handle_custom_command(command: &str, editor: &mut EditorCorePlugin) {
    match command {
        "format" => {
            // Format current file
            let content = editor.get_current_content();
            let formatted = format_code(&content, &editor.get_language());
            editor.set_current_content(formatted);
        }
        "lint" => {
            // Run linter
            let issues = lint_code(&editor.get_current_content());
            display_lint_issues(issues);
        }
        _ => {}
    }
}
```

## Performance Optimizations

### Efficient Text Operations
- **Rope data structure** consideration for very large files
- **Incremental parsing** for syntax highlighting
- **Lazy loading** of file content
- **Viewport-based rendering** for large documents

### Memory Management
- **Line-based storage** reduces memory fragmentation
- **Copy-on-write** for undo/redo operations
- **String interning** for repeated tokens
- **Bounded history** limits undo stack size

## Configuration

### Editor Settings
```rust
pub struct EditorConfig {
    pub tab_size: usize,           // Default: 4
    pub use_spaces: bool,           // Default: true
    pub auto_indent: bool,          // Default: true
    pub word_wrap: bool,            // Default: false
    pub show_whitespace: bool,      // Default: false
    pub highlight_current_line: bool, // Default: true
    pub vim_mode_default: bool,     // Default: false
    pub auto_save_interval: Option<u64>, // Seconds, None = disabled
}
```

### Theme Configuration
```rust
pub struct EditorTheme {
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,
    pub line_numbers: Color,
    pub current_line: Color,
    pub syntax: SyntaxColors,
}

pub struct SyntaxColors {
    pub keyword: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub function: Color,
    pub type_name: Color,
    pub variable: Color,
    pub operator: Color,
}
```

## Testing

```bash
# Run unit tests
cargo test -p playground-plugins-editor-core

# Test Vim mode implementation
cargo test -p playground-plugins-editor-core vim_

# Test buffer operations
cargo test -p playground-plugins-editor-core buffer_

# Benchmark text operations
cargo bench -p playground-plugins-editor-core
```

## Dependencies

- `playground-core-plugin`: Plugin trait and lifecycle
- `playground-core-types`: Core type definitions
- `playground-systems-ui`: UI system integration and rendering
- `playground-systems-logic`: Game logic and world management
- `playground-systems-networking`: Channel communication
- `tree-sitter`: Syntax highlighting (optional)
- `tree-sitter-rust`: Rust syntax support
- `nalgebra`: Vector math for rendering
- `async-trait`: Async plugin traits
- `serde`/`serde_json`: State serialization
- `tokio`: Async runtime
- `uuid`: Unique identifiers
- `tracing`: Logging and diagnostics

## Performance Metrics

- **Startup time**: < 50ms
- **File open (< 10MB)**: < 100ms
- **Keystroke latency**: < 16ms (60 FPS)
- **Syntax highlighting**: < 50ms for 1000 lines
- **Search in file**: < 20ms for 10,000 lines
- **Memory per file**: ~2x file size
- **Undo history**: 1000 operations default

## Future Enhancements

- [ ] Language Server Protocol (LSP) integration
- [ ] Code folding
- [ ] Multiple selections/cursors with Vim
- [ ] Snippet support
- [ ] Bracket matching and auto-closing
- [ ] Incremental search
- [ ] Split view editing
- [ ] Minimap
- [ ] Git diff indicators
- [ ] Collaborative editing support

## License

See the main project LICENSE file for details.