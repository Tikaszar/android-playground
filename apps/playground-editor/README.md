# Playground Editor

The main IDE application that orchestrates all plugins and systems to create a complete Conversational IDE experience.

## Overview

Playground Editor is the primary application of the Android Playground ecosystem. It initializes the core server, manages all systems through the logic layer, loads IDE plugins, and provides the MCP endpoint for LLM integration. This is the entry point for the entire Conversational IDE.

## Architecture

```
playground-editor/
├── main.rs         # Application entry point and orchestration
├── layout.rs       # UI layout management
├── messages.rs     # Inter-component messaging
├── message_bus.rs  # Event distribution system
└── static/         # Web assets
    ├── index.html  # Browser interface
    ├── app.js      # Client-side logic
    └── styles.css  # UI styling
```

## Core Responsibilities

### 1. System Initialization

The app initializes all systems through the logic layer:

```rust
// Initialize ECS from systems/logic
let mut ecs = ECS::new();

// Initialize ALL systems
let systems = ecs.initialize_systems().await?;
// This creates:
// - NetworkingSystem (WebSocket multiplexer)
// - UiSystem (UI management)
// - RenderingSystem (browser-side only)
```

### 2. Core Server Management

Starts the core server internally:

```rust
// Start core server on port 8080
tokio::spawn(async {
    run_core_server().await
});

// Server provides:
// - WebSocket endpoint at ws://localhost:8080/ws
// - MCP endpoint at http://localhost:8080/mcp
// - Static file serving at http://localhost:8080/
```

### 3. Plugin Orchestration

Loads and manages IDE plugins:

```rust
// Register plugin channels
systems.register_plugin_channels("ui-framework", 1200, 10).await?;

// Load UI Framework Plugin
let mut ui_plugin = UiFrameworkPlugin::new();

// Provide systems access to plugins
let mut context = Context::new();
context.resources.insert("networking", Box::new(systems.networking.clone()));
context.resources.insert("ui", Box::new(systems.ui.clone()));

// Initialize plugin
ui_plugin.on_load(&mut context).await?;
```

### 4. Update Loop

Runs the main update loop at 60 FPS:

```rust
// Update loop for plugins and systems
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_millis(16));
    loop {
        interval.tick().await;
        
        // Update plugin
        plugin.update(&mut context, 0.016).await;
        
        // Update systems
        systems.update(0.016).await;
    }
});
```

## Channel Architecture

The editor manages channel allocation for all components:

```
Channel Ranges:
- 0: Control channel
- 1-999: Systems
  - 10: UI System
  - 20: Networking System
  - 30: Rendering System
- 1000-1099: IDE Plugins
  - 1000-1009: Editor Core
  - 1010-1019: File Browser
  - 1020-1029: Terminal
  - 1030-1039: Chat
  - 1040-1049: Chat Assistant
  - 1050-1059: LSP Client
  - 1060-1069: Debugger
  - 1070-1079: Theme Manager
  - 1080-1089: Version Control
- 1100-1199: Game Plugins
- 1200-1209: UI Framework Plugin
- 2000-2999: LLM Sessions via MCP
```

## MCP Integration

The editor provides the MCP server for LLM connections:

### Available MCP Tools

```json
{
  "tools": [
    "show_file",
    "update_editor",
    "show_terminal_output",
    "update_file_tree",
    "show_diff",
    "execute_command",
    "save_file",
    "read_file",
    "create_task",
    "assign_task"
  ]
}
```

### Connecting LLMs

```bash
# Start the editor
cargo run -p playground-apps-editor

# Connect Claude Code
claude --mcp http://localhost:8080/mcp

# Connect via custom client
curl -X POST http://localhost:8080/mcp/sessions
```

## Web Interface

The browser interface at `http://localhost:8080/playground-editor/`:

### Features
- WebSocket connection to server
- Binary protocol message handling
- Real-time UI updates
- Touch-optimized for mobile
- WebGL rendering support

### Client-Server Protocol

```javascript
// Binary message format
[channel_id: u16][packet_type: u16][payload: bytes]

// Connect to server
const ws = new WebSocket('ws://localhost:8080/ws');

// Send messages
ws.send(encodeMessage(1200, 1, { type: 'get_state' }));

// Receive updates
ws.onmessage = (event) => {
  const { channel, type, data } = decodeMessage(event.data);
  updateUI(data);
};
```

## Running the Editor

### Development Mode

```bash
# Run with hot reload
cargo watch -x "run -p playground-apps-editor"

# With logging
RUST_LOG=debug cargo run -p playground-apps-editor

# With custom port
PORT=3000 cargo run -p playground-apps-editor
```

### Production Build

```bash
# Build optimized binary
cargo build -p playground-apps-editor --release

# Run production build
./target/release/playground-apps-editor
```

## Configuration

### Environment Variables

```bash
# Server configuration
PORT=8080                    # HTTP/WebSocket port
HOST=0.0.0.0                # Bind address
RUST_LOG=info               # Log level

# MCP configuration
MCP_ENABLED=true            # Enable MCP server
MCP_AUTH_TOKEN=secret       # Optional auth token

# Plugin configuration
PLUGINS_DIR=/plugins        # Plugin directory
AUTO_LOAD_PLUGINS=true      # Auto-load plugins
```

## Plugin Loading

The editor can load plugins dynamically:

```rust
// Load additional plugins at runtime
let plugin_path = PathBuf::from("./plugins/custom-plugin.so");
let plugin = load_plugin(&plugin_path)?;
plugin.on_load(&mut context).await?;

// Register plugin channels
systems.register_plugin_channels(&plugin.metadata().id, base_channel, count).await?;
```

## System Access Pattern

The editor follows strict architectural rules:

```rust
// CORRECT: App provides Systems to Plugins
context.resources.insert("networking", Box::new(systems.networking));
plugin.on_load(&mut context).await?;

// WRONG: App accessing Core directly
// core::server::start().await;  // Never do this!

// CORRECT: App uses Systems which use Core
let systems = ecs.initialize_systems().await?;
```

## Performance Metrics

- **Startup time**: < 2 seconds
- **WebSocket latency**: < 5ms local, < 50ms remote
- **Update loop**: Consistent 60 FPS
- **Memory usage**: ~50MB base + plugins
- **Channel throughput**: 10,000 messages/second
- **Concurrent connections**: 100+ WebSocket clients

## Testing

```bash
# Run all tests
cargo test -p playground-apps-editor

# Integration tests
cargo test -p playground-apps-editor --test integration

# Benchmark performance
cargo bench -p playground-apps-editor

# Test MCP endpoint
curl -X POST http://localhost:8080/mcp/sessions
```

## Dependencies

### Core Dependencies
- `playground-systems-logic`: System initialization and ECS
- `playground-systems-networking`: WebSocket communication
- `playground-systems-ui`: UI system management
- `playground-plugins-ui-framework`: Main UI orchestration

### Runtime Dependencies
- `tokio`: Async runtime
- `anyhow`: Error handling
- `tracing`: Logging and diagnostics
- `serde`/`serde_json`: Serialization

## License

See the main project LICENSE file for details.