# Playground Editor Plugin

In-browser code editor and development tools for Android Playground.

## Overview

This plugin provides a full-featured code editor that runs in the browser, designed specifically for touch-friendly development on Android devices. It integrates with the playground-server to enable editing, building, and hot-reloading plugins directly from a mobile browser.

## Features

- **Code Editing** - Syntax highlighting, auto-completion, and code folding
- **File Browser** - Navigate and manage project files
- **Multi-file Support** - Work with multiple files simultaneously
- **Touch Gestures** - Optimized for touch input and mobile screens
- **Live Preview** - See changes reflected in real-time
- **Build Integration** - Compile and reload plugins from the editor
- **State Persistence** - Remember open files and cursor positions

## Editor Components

### File Management
- Tree view file browser
- Create, rename, and delete files
- Quick file switching
- Recent files list

### Code Editor
- Syntax highlighting for Rust, TOML, and more
- Auto-indentation
- Bracket matching
- Code folding
- Find and replace

### Build Tools
- Run cargo commands
- View build output
- Error highlighting
- Quick-fix suggestions

## Plugin Structure

```
src/
├── lib.rs       # Plugin entry point and exports
├── plugin.rs    # Main editor implementation
├── state.rs     # Editor state management
└── handlers.rs  # Event handlers for file operations
```

## Integration

The editor plugin depends on:
- `ui` system - For rendering the editor interface
- `networking` system - For WebSocket communication with the server

## Keyboard Shortcuts

- `Ctrl+S` - Save current file
- `Ctrl+O` - Open file
- `Ctrl+N` - New file
- `Ctrl+F` - Find
- `Ctrl+H` - Find and replace
- `Ctrl+Tab` - Switch between open files

## Touch Gestures

- **Tap** - Position cursor
- **Double-tap** - Select word
- **Long press** - Context menu
- **Pinch** - Zoom in/out
- **Two-finger scroll** - Navigate code

## Configuration

Editor settings are persisted in the plugin state:
- Theme (light/dark)
- Font size
- Tab size
- Auto-save interval
- Syntax highlighting preferences

## Building

```bash
# Build the editor plugin
cargo build -p playground-editor --release

# The plugin will be available at:
# target/release/libplayground_editor.so
```

## Dependencies

- `playground-plugin` - Plugin trait definitions
- `playground-types` - Core type system
- `serde` - State serialization
- `tracing` - Logging support