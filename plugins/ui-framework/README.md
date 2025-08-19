# UI Framework Plugin

A comprehensive UI orchestration system that manages all visual components, chat channels, inline editors, and agent coordination for the Android Playground IDE.

## Overview

The UI Framework Plugin serves as the central hub for all UI interactions in the Android Playground ecosystem. It provides a sophisticated multi-agent chat interface with inline components, task management, and real-time collaboration features. The plugin bridges server-side logic with browser-based rendering through WebSocket communication and MCP (Model Context Protocol) tool calls.

## Architecture

```
ui-framework/
├── plugin.rs           # Main plugin implementation and lifecycle
├── components.rs       # Core UI component definitions
├── channel_manager.rs  # Chat channel and agent management
├── message_system.rs   # Message creation and formatting
├── orchestrator.rs     # Agent coordination and task assignment
├── mcp_handler.rs      # MCP tool call processing
├── browser_bridge.rs   # Browser communication layer
├── websocket_handler.rs # WebSocket protocol handling
├── panel_manager.rs    # Panel layout management
└── ui_state.rs        # Global UI state management
```

## Core Features

### 1. Multi-Agent Chat System

#### Channel Management
```rust
use playground_plugins_ui_framework::{ChannelManager, ChannelType, AgentId};

// Create a new channel
let channel_id = channel_manager.create_channel(
    "Development Team".to_string(),
    ChannelType::Group,
    vec![orchestrator_id, worker1_id, worker2_id]
).await?;

// List channels for an agent
let agent_channels = channel_manager.list_channels_for_agent(&agent_id);

// Add/remove participants dynamically
channel_manager.add_participant(channel_id, new_agent_id).await?;
channel_manager.remove_participant(channel_id, old_agent_id).await?;
```

#### Message Types
- **Text Messages**: Plain text communication
- **Code Blocks**: Syntax-highlighted code snippets
- **Inline Editors**: Full-featured code editors embedded in messages
- **Inline File Browsers**: Interactive file system navigation
- **Inline Terminals**: Live terminal output with scrolling
- **Inline Diffs**: Visual diff viewers for code changes
- **System Notifications**: Automated status updates

### 2. Inline Components

#### Inline Editor Component
```rust
use playground_plugins_ui_framework::{MessageSystem, VimMode};

// Send an inline editor to a channel
let message_id = message_system.send_inline_editor(
    channel_id,
    sender_id,
    "/src/main.rs".to_string(),
    file_content,
    "rust".to_string()
).await?;

// Editor features:
// - Vim mode support (Normal, Insert, Visual, Command)
// - Syntax highlighting for multiple languages
// - Cursor position tracking
// - Selection support
// - Dirty state tracking
// - Original content for diff generation
```

#### Inline File Browser
```rust
// Create file browser component
let entries = vec![
    FileEntry {
        name: "src".to_string(),
        path: PathBuf::from("/project/src"),
        is_directory: true,
        size: None,
        git_status: Some(GitStatus::Modified),
    },
    // ... more entries
];

let message_id = message_system.send_inline_file_browser(
    channel_id,
    sender_id,
    PathBuf::from("/project"),
    entries
).await?;
```

#### Inline Terminal
```rust
// Stream terminal output to chat
let message_id = message_system.send_inline_terminal(
    channel_id,
    sender_id,
    session_id,
    vec!["$ cargo build".to_string(), "   Compiling...".to_string()]
).await?;

// Append output dynamically
message_system.append_terminal_output(
    channel_id,
    message_id,
    vec!["Build complete!".to_string()]
).await?;
```

### 3. Agent System

#### Agent Types
- **Orchestrator**: Manages tasks and assigns work to workers
- **Worker**: Executes assigned tasks with specific permissions
- **Human**: Human users interacting with the system

#### Agent Management
```rust
use playground_plugins_ui_framework::{AgentComponent, AgentType, AgentStatus};

// Register a new worker agent
let worker = AgentComponent {
    id: AgentId::new(),
    name: "Rust Worker".to_string(),
    agent_type: AgentType::Worker,
    status: AgentStatus::Idle,
    worktree_path: Some(PathBuf::from("/workspace/rust")),
    permissions: AgentPermissions {
        can_execute_commands: true,
        can_modify_files: true,
        can_create_worktrees: false,
        can_assign_tasks: false,
    },
    current_task: None,
    last_active: Utc::now(),
};

channel_manager.register_agent(worker).await?;
```

### 4. Task Queue System

#### Task Management
```rust
use playground_plugins_ui_framework::{Task, TaskPriority, TaskStatus};

// Create a new task
let task_id = orchestrator.create_task(
    "Implement authentication".to_string(),
    "Add JWT-based auth to the API endpoints".to_string(),
    TaskPriority::High,
    vec![
        PathBuf::from("/src/auth.rs"),
        PathBuf::from("/src/middleware.rs"),
    ]
).await?;

// Task assignment happens automatically
orchestrator.run_assignment_loop().await;

// Complete a task
orchestrator.complete_task(
    agent_id,
    TaskResult {
        task_id,
        success: true,
        output: "Authentication implemented successfully".to_string(),
        files_modified: vec![PathBuf::from("/src/auth.rs")],
        duration_seconds: 3600,
    }
).await?;
```

### 5. MCP Tool Integration

The plugin exposes numerous MCP tools for external systems:

```rust
// Available MCP tools:
- show_file           // Display file with syntax highlighting
- update_editor       // Modify editor content
- show_terminal_output // Display terminal output
- update_file_tree    // Refresh file browser
- show_diff          // Display code differences
- show_error         // Show error messages
- update_status_bar  // Update status information
- show_notification  // Display notifications
- show_chat_message  // Send chat messages
- execute_command    // Run shell commands
- save_file         // Persist file changes
- read_file         // Load file content
- create_task       // Create new tasks
- create_worker     // Spawn worker agents
- assign_task       // Assign tasks to agents
- complete_task     // Mark tasks as complete
```

Example MCP tool call:
```json
{
  "tool": "show_file",
  "arguments": {
    "path": "/src/main.rs",
    "language": "rust"
  }
}
```

### 6. Bubble State Management

Messages support three bubble states for optimal screen usage:

```rust
pub enum BubbleState {
    Collapsed,   // Title + timestamp only
    Compressed,  // Key lines/content (MCP-specified)
    Expanded,    // Full content with scrolling
}

// Toggle bubble state
message_system.toggle_bubble_state(channel_id, message_id).await?;
```

### 7. Persistence System

Conversations and state are automatically persisted:

```rust
// Initialize with persistence
let ui_state = UiState::with_persistence(
    PathBuf::from("/data/data/com.termux/files/home/.android-playground/conversations")
);

// Manual save/load
channel_manager.save_to_disk().await?;
channel_manager.load_from_disk().await?;
```

## WebSocket Protocol

### Binary Message Format
```
[channel_id: u16][packet_type: u16][payload: bytes]
```

### Channel Allocation
- **1200-1209**: UI Framework channels
- **1200**: Main UI control channel
- **1201**: Tool result responses
- **1202-1209**: Component-specific channels

### Message Types
```rust
// Browser → Server
GetState               // Request current UI state
SendMessage           // Send chat message
BubbleStateChange     // Toggle message bubble
SaveFile             // Save file to disk
OpenFile            // Open file in editor
ToggleDirectory     // Expand/collapse directory

// Server → Browser
StateUpdate         // Full state synchronization
Message            // New chat message
ChannelUpdate      // Channel modifications
InlineComponent    // Component updates
AgentStatus       // Agent state changes
Error            // Error notifications
```

## Integration with Systems

### Networking System
```rust
// Register with networking system for channel allocation
let channel_id = networking.register_plugin("ui-framework").await?;

// Send packets
networking.send_packet(
    1200,                    // Channel
    PacketType::ToolCall,    // Type
    data,                    // Payload
    Priority::High          // Priority
).await?;

// Receive packets
let packets = networking.receive_packets(channel_id).await?;
```

### UI System
```rust
// Coordinate with UI system for rendering
ui_system.register_panel(panel_id, panel_type).await?;
ui_system.update_panel_content(panel_id, content).await?;
```

## Configuration

### Plugin Metadata
```rust
PluginMetadata {
    id: PluginId("ui-framework"),
    name: "UI Framework",
    version: Version { major: 0, minor: 1, patch: 0 },
}
```

### Environment Variables
```bash
# Persistence directory
UI_FRAMEWORK_PERSISTENCE_DIR=/data/data/com.termux/files/home/.android-playground

# WebSocket configuration
UI_FRAMEWORK_WS_PORT=8080
UI_FRAMEWORK_WS_PATH=/ws/ui

# Agent limits
UI_FRAMEWORK_MAX_WORKERS=10
UI_FRAMEWORK_MAX_TASKS_PER_WORKER=3
```

## Usage Examples

### Complete Example: Multi-Agent Development Session
```rust
use playground_plugins_ui_framework::*;

// Initialize UI Framework
let mut ui_plugin = UiFrameworkPlugin::new();
ui_plugin.on_load(&mut context).await?;

// Create development channel
let channel_id = {
    let ui_state = ui_plugin.ui_state.read().await;
    ui_state.channel_manager.write().await.create_channel(
        "Feature Development".to_string(),
        ChannelType::Group,
        vec![]  // Will add agents
    ).await?
};

// Spawn orchestrator
let orchestrator_id = {
    let mut orchestrator = ui_plugin.orchestrator.write().await;
    orchestrator.initialize().await?;
    orchestrator.orchestrator_id
};

// Create worker agents
let rust_worker = create_worker_agent("Rust Worker", "/workspace/rust").await?;
let js_worker = create_worker_agent("JS Worker", "/workspace/js").await?;

// Add agents to channel
add_agents_to_channel(channel_id, vec![orchestrator_id, rust_worker, js_worker]).await?;

// Create tasks
let backend_task = create_task(
    "Implement REST API",
    "Create CRUD endpoints for user management",
    TaskPriority::High,
    vec!["/src/api/users.rs"]
).await?;

let frontend_task = create_task(
    "Build UI components",
    "Create user list and edit forms",
    TaskPriority::Medium,
    vec!["/src/components/UserList.tsx"]
).await?;

// Tasks are automatically assigned to available workers
// Monitor progress through chat messages and inline components
```

### MCP Tool Call Example
```rust
// Handle incoming MCP tool call
let tool_call = serde_json::json!({
    "tool": "show_file",
    "arguments": {
        "path": "/src/main.rs",
        "language": "rust"
    }
});

let result = ui_plugin.mcp_handler.handle_tool_call(
    "show_file",
    tool_call["arguments"].clone()
).await?;

// Result is automatically displayed as inline editor in active channel
```

## Performance Considerations

### Optimization Strategies
1. **Message Batching**: Groups multiple updates into single WebSocket frames
2. **Lazy Loading**: Components load content on-demand
3. **Virtual Scrolling**: Large message lists use virtual rendering
4. **Compression**: Binary protocol with optional zstd compression
5. **Caching**: Frequently accessed files cached in memory

### Resource Limits
- Maximum channels: 100
- Maximum messages per channel: 10,000
- Maximum agents: 50
- Maximum concurrent tasks: 100
- Terminal output buffer: 1,000 lines
- File browser cache: 1,000 entries

## Error Handling

All operations return `Result<T, anyhow::Error>` with detailed error context:

```rust
// Comprehensive error handling
match channel_manager.create_channel(name, channel_type, participants).await {
    Ok(channel_id) => {
        info!("Channel created: {}", channel_id);
    }
    Err(e) => {
        error!("Failed to create channel: {}", e);
        // Errors include context like:
        // - Agent not found
        // - Persistence failure
        // - Network errors
        // - Invalid parameters
    }
}
```

## Testing

```bash
# Run unit tests
cargo test -p playground-plugins-ui-framework

# Run integration tests
cargo test -p playground-plugins-ui-framework --test integration

# Test with mock browser
cargo run -p playground-plugins-ui-framework --example mock_browser

# Benchmark performance
cargo bench -p playground-plugins-ui-framework
```

## Dependencies

- `playground-core-plugin`: Plugin trait and lifecycle
- `playground-core-types`: Core type definitions
- `playground-systems-ui`: UI system integration
- `playground-systems-networking`: WebSocket communication
- `playground-systems-logic`: Game logic coordination
- `async-trait`: Async trait support
- `tokio`: Async runtime
- `serde`/`serde_json`: Serialization
- `uuid`: Unique identifiers
- `chrono`: Timestamp management
- `anyhow`: Error handling
- `tracing`: Logging and diagnostics
- `bytes`: Binary message handling
- `dashmap`: Concurrent hash maps

## License

See the main project LICENSE file for details.