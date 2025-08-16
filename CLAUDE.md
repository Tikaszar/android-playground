# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Android Playground is a mobile-first, plugin-based game engine designed for development entirely on Android devices using Termux. The architecture prioritizes hot-reload capabilities, battery efficiency, and touch-friendly development.

## Architecture

### Crate Structure
```
core/           # Foundation layer (minimal dependencies)
├── types       # Shared types and traits (zero dependencies)
├── android     # Android JNI bindings
├── server      # Axum-based web server for browser editor
└── plugin      # Plugin trait and loading mechanism

systems/        # Engine components (depend on core)
├── ui          # Conversational-first IDE with persistent UI graph
├── networking  # WebSocket, WebRTC protocols
├── physics     # 2D/3D physics simulation
├── logic       # ECS, state machines
└── rendering   # Multi-backend renderer (WebGL, future Vulkan)

plugins/        # Games and applications
├── idle-game   # First production game
└── playground-editor  # In-browser development tools
```

### Plugin System

Plugins are compiled as `.so` files and loaded dynamically. The core `Plugin` trait (defined in `core/plugin`) requires:
- Unique ID, name, and version
- Lifecycle hooks: `on_load`, `on_unload`, `update`, `render`, `on_event`
- State preservation for hot-reload via `Stateful` trait
- Message passing through context for inter-plugin communication

Entry point for each plugin:
```rust
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin>
```

## Development Commands

Since this is a new project without established build infrastructure yet, here are the expected commands once the Cargo workspace is set up:

```bash
# Build all crates
cargo build --workspace

# Build specific plugin as dynamic library
cargo build -p idle-game --release

# Run the development server
cargo run -p playground-server

# Watch and rebuild plugins on change (once implemented)
cargo watch -x 'build -p idle-game'
```

## Development Environment Constraints

- All development happens in Termux on Android
- No access to traditional desktop IDEs
- Browser-based code editor served by `core/server`
- Limited system resources compared to desktop
- Touch input as primary interaction method

## Key Design Decisions

1. **Everything is a plugin** - Even core systems can be replaced/reloaded
2. **Message passing over direct calls** - Enables hot-reload without breaking references
3. **Shared state through core types** - All plugins depend on `core/types` for compatibility
4. **Battery-efficient builds** - Incremental compilation and minimal rebuilds
5. **APK packaging** - Final distribution through standard Android packages

## Rendering System Design

### BaseRenderer Architecture

The rendering system uses a stateless base trait that all renderer implementations (WebGL, Vulkan) must implement. Key features:

- **Single Draw Call Batching**: All geometry is batched into ONE optimized draw call per frame
- **Immutable Pipelines**: All render state (blend, depth, rasterizer) is baked into pipelines at creation
- **Hot-Reload Support**: Automatic shader recompilation on file changes
- **Compute Shader Ready**: Full compute API (stubbed in WebGL, implemented in Vulkan)
- **Texture Streaming**: Automatic LOD management and memory budget adjustment
- **Debug & Metrics**: Performance tracking, GPU markers, resource naming

### Buffer Types

Distinct buffer types for type safety:
- `VertexBuffer`: Vertex data with format
- `IndexBuffer`: Index data with type
- `UniformBuffer`: Shader uniforms
- `StorageBuffer`: Compute storage (read/write)
- `StagingBuffer`: CPU to GPU transfers

### Render Graph

- Persistent graphs with runtime modification
- Swappable graph templates
- Unified graph for render and compute passes
- Pass inheritance hierarchy:
  - Base `Pass` trait
  - `RenderPass`, `ComputePass`, `CopyPass`, `BlitPass`

### Resource Management

- Opaque handle system with recycling
- Runtime shader compilation from `.glsl` files
- Automatic device recovery on GPU lost
- Safe handle validation (never crashes)

### Coordinate System

- Right-handed: X=Right, Y=Up, Z=Forward (all positive)
- Units in meters

### File Organization

**One class per file principle**: Each struct/trait lives in its own file. For example:
- `math/vector.rs` contains generic `Vector<const N: usize, T>`
- `math/matrix.rs` contains generic `Matrix<R, C>`
- `math/types.rs` contains convenience types (`Vec2`, `Vec3`, `Vec4`, `Mat4`, etc.)

## WebGL Renderer Implementation

### Current Features
- WebGL2 context management with automatic recovery
- Resource pooling with ID recycling for buffers, textures, shaders
- State caching to minimize redundant GL calls
- GLSL shader compilation with caching
- Command buffer system for deferred rendering
- Framebuffer object pooling for render targets
- Full implementation of BaseRenderer trait

### WebGL Limitations
- No compute shader support (returns NotSupported error)
- No storage buffers (SSBOs not available in WebGL2)
- WebGL types are not Send+Sync (single-threaded only)
- Maximum 16 texture units
- Maximum 64KB uniform buffer size

### Building with WebGL
```bash
# Build with WebGL feature enabled
cargo build -p playground-rendering --features webgl

# Note: WebGL feature is for WASM/browser targets
# Native builds should use default features
```

## Current Status

✅ **Implemented**
- Core layer (types, plugin, server, android)
- BaseRenderer trait with complete API
- WebGL2 renderer backend
- Resource management system
- Render graph with pass system
- Command buffer architecture
- State management (blend, depth, rasterizer)
- Performance metrics collection
- Debug naming and markers

🚧 **In Development**
- Browser IDE (playground-editor plugin)
- UI system components
- Hot-reload file watching

📋 **Next Steps**
- Vulkan renderer for compute support
- Physics system integration
- Networking protocols
- ECS implementation in logic system

## UI System Design

### Conversational-First IDE

The UI system implements a unique conversational IDE that prioritizes chat-based interactions while maintaining full traditional IDE capabilities. All UI state is server-side with the browser acting as a pure view.

### Core UI Architecture

- **Persistent UI Graph**: UI elements exist as persistent nodes in the RenderGraph
- **Single Draw Call**: All UI contributes to ONE optimized draw call per frame
- **Server-Side State**: Browser is purely a view, all logic/storage on server
- **Conversational-First**: IM chat as primary interface, traditional IDE as secondary

### Layout System

- **Flexbox Layout**: Full CSS flexbox properties with nested container support
- **Absolute Positioning**: Foundation layer beneath flexbox
- **Responsive Design**: Portrait/landscape layouts with screen-relative constraints
- **Docking System**: VSCode/Godot-style panes - draggable, dockable, save/restore layouts

### Conversational IDE Components

- **Chat Interface**: Message bubbles with inline editable code blocks
- **Focused Editing**: Request specific code sections (functions, classes)
- **Code Snippets**: Syntax-highlighted, editable code within chat
- **Context Actions**: Save, run, "Open in IDE" buttons inline
- **Version Control**: Inline diffs in chat conversation
- **Persistent History**: Searchable, branching conversations

### Traditional IDE Components

- **Code Editor**: 
  - Full rust-analyzer integration via LSP
  - Multi-cursor (VS Studio style alt-select)
  - Vim mode support
  - Inline error highlights
- **Terminal**: Real terminal connection to Termux instance
  - Direct Claude Code interaction
  - Full shell capabilities
  - No PTY - actual terminal
- **File Tree**: List/icon views with lazy loading
- **Debugger**: Breakpoints, watches, stack traces

### Input System

- **Mobile**: Touch, pinch, swipe gestures with floating toolbar
- **Desktop**: VS Code keyboard shortcuts
- **Event Flow**: Bubbling through visual hierarchy
- **Multi-cursor**: Alt-select for multiple line selection

### Rendering Integration

- **Text Rendering**: Runtime-generated SDF with caching
- **Batching**: UI geometry provided to WebGL batcher
- **Themes**: Dark and Light themes with per-element overrides
- **Animations**: Built-in transitions with configurable timing

### Performance Features

- **Dirty Flags**: Track changed UI elements
- **Occlusion Culling**: Skip off-screen elements
- **Message Virtualization**: Only render visible chat messages
- **Level-of-Detail**: Simplify complex UI when zoomed out

### Terminal Integration

The terminal connects directly to the Termux instance running on the phone, enabling Claude Code interaction without GPL requirements. Communication uses the Axum server to proxy terminal I/O to the browser.

### Mobile-Specific Features

- **Gesture Shortcuts**: Swipe between chat/IDE views
- **Touch Targets**: Appropriately sized for mobile
- **On-screen Keyboard**: Optimized for code input
- **Floating Toolbar**: Quick access to common actions

## Development Notes

### Renderer Usage
```rust
// Create WebGL renderer for browser
let canvas = get_canvas_element();
let mut renderer = WebGLRenderer::with_canvas(canvas)?;
renderer.initialize()?;

// Create resources
let vb = renderer.create_vertex_buffer(data, &format)?;
let texture = renderer.create_texture(&desc)?;

// Render frame
renderer.begin_frame()?;
renderer.render(&graph)?;
renderer.end_frame()?;
renderer.present()?;
```

### UI System Usage
```rust
// Create UI element in render graph
let chat_panel = ui.create_panel()
    .layout(FlexLayout::column())
    .theme(Theme::Dark);

// Add inline code editor to chat
let code_block = chat_panel.add_code_editor()
    .language("rust")
    .editable(true)
    .focused_lines(100..150);

// Handle conversational requests
ui.on_message("show me the update loop", |ui, context| {
    let code = context.find_function("update");
    ui.show_inline_editor(code);
});
```

### Important Considerations
- Always use feature flags for renderer selection
- WebGL renderer is for browser IDE only
- Vulkan will be primary renderer for production
- All rendering goes through BaseRenderer trait
- Single draw call per frame is the target
- UI system uses core/server for all communication