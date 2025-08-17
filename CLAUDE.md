# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Android Playground is a mobile-first, plugin-based game engine designed for development entirely on Android devices using Termux. The architecture prioritizes hot-reload capabilities, battery efficiency, and touch-friendly development.

## Architecture

### 4-Layer Architecture
```
apps/           # Complete products (games, IDEs, etc.)
├── playground-editor  # Browser-based IDE
└── idle-mmo-rpg      # Future production game

plugins/        # Reusable feature modules
├── inventory   # Inventory management system
├── combat      # Combat mechanics
├── chat        # Real-time chat system
└── editor-core # Core editor functionality

systems/        # Engine components (depend on core)
├── ui          # UI framework with persistent graph
├── networking  # Game networking and plugin communication
├── physics     # 2D/3D physics simulation
├── logic       # ECS, state machines
└── rendering   # Multi-backend renderer (WebGL, future Vulkan)

core/           # Foundation layer (minimal dependencies)
├── types       # Shared types and traits (zero dependencies)
├── ecs         # Minimal ECS primitives for Systems to use
├── android     # Android JNI bindings
├── server      # WebSocket multiplexer and channel management
├── client      # Browser WASM WebSocket client
└── plugin      # Plugin trait and loading mechanism
```

### Architectural Rules
1. **Apps** manage and coordinate collections of Plugins
2. **Plugins** ONLY use Systems APIs (NEVER use Core directly)
3. **Systems** ONLY use Core APIs
4. **Core** provides foundational functionality
5. **Exception**: Plugins may implement custom Systems internally that use Core

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

1. **Strict layer separation** - Apps → Plugins → Systems → Core
2. **Message passing over direct calls** - Enables hot-reload without breaking references
3. **WebSocket-only networking** - Binary protocol with frame-based batching
4. **Battery-efficient builds** - Incremental compilation and minimal rebuilds
5. **Server-side authority** - Browser is purely a view, all logic on server
6. **APK packaging** - Final distribution through standard Android packages
7. **NO unsafe code** - The entire engine avoids `unsafe` blocks for reliability
8. **NO std::any::Any** - Avoid runtime type casting, use proper serialization instead

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

## Networking System Design

### WebSocket Architecture

The entire networking layer is built on WebSockets with binary protocol for efficiency:

#### Core Layer Networking (IMPLEMENTED)
- **core/server**: WebSocket multiplexer with channel management ✅
  - Channel 0: Control channel for registration and discovery ✅
  - Channels 1-999: Reserved for Systems ✅
  - Channels 1000+: Dynamically allocated to Plugins/Apps ✅
  - Frame-based packet batching (60fps default) ✅
  - Binary serialization using `bytes` crate ✅
  - Priority queue system (5 levels) ✅
  - Passkey authentication with 1Password integration (pending)
  - Google OAuth support for external access (pending)

- **core/client**: Browser WASM WebSocket client ✅
  - Mirrors server channel architecture ✅
  - Binary message handling and routing ✅
  - WASM bindings for browser integration ✅
  - Automatic reconnection with exponential backoff (pending)

#### Channel System
- Dynamic channel registration at runtime
- KV store for channel discovery (query by name)
- Priority queues per channel (5 levels: Low, Medium, High, Critical, Blocker)
- Frame-based batching with configurable rates (default 60fps)
- Single WebSocket endpoint: `ws://localhost:3000/ws`

#### Message Protocol
```rust
struct Packet {
    channel_id: u16,
    packet_type: u16,
    priority: u8,
    payload_size: u32,
    payload: Vec<u8>,
}
```

#### Communication Flow
1. Plugin → System API call
2. System → Serialize to binary packet
3. System → Queue in core/server
4. core/server → Batch packets per frame
5. core/server → Send via WebSocket
6. core/client → Receive and route by channel
7. Client System → Deserialize
8. Client System → Deliver to Plugin

### System Registration Flow
- **Systems**: Register with core/server or core/client, receive channels 1-999
- **Plugins**: Register through systems/networking, receive channels 1000+
- **Apps**: Coordinate plugins through systems/networking

### WASM Compilation Strategies

Three supported compilation modes (configurable via feature flags and runtime config):
1. **Separate**: Each System, Plugin, and App compiles to individual .wasm
2. **Hybrid**: Each System and App compiles to .wasm
3. **Unified**: Complete App compiles as single .wasm

## Current Status

✅ **Implemented**
- Core layer (types, plugin, server, android, client, **ecs**)
- **Core/ECS** with async, generational IDs, and batch-only API
- **Systems/Logic** full-featured ECS with hybrid storage and scheduler
- **WebSocket multiplexer** in core/server with binary protocol
- **Channel management system** with dynamic registration
- **Frame-based packet batching** at 60fps
- **Priority queue system** with 5 levels
- **Binary packet protocol** with efficient serialization
- **WASM client module** with browser integration
- BaseRenderer trait with complete API
- WebGL2 renderer backend
- Resource management system
- Render graph with pass system
- Command buffer architecture
- State management (blend, depth, rasterizer)
- Performance metrics collection
- Debug naming and markers
- **Complete docking system** (1000+ lines, drag & drop, tabs, serialization)
- **UI Element trait system** with layout, render, input handling
- **File tree component** with expand/collapse and lazy loading
- **Chat interface** with message bubbles and code blocks
- **Code editor** with vim mode, syntax highlighting, multi-cursor
- **Terminal** with WebSocket connection and ANSI parsing
- **Mobile gesture support** with full multi-touch recognition (500+ lines)
- **Floating toolbar** for mobile-specific actions (400+ lines)
- **Gesture element wrapper** for adding gestures to any UI element (300+ lines)
- **SDF text rendering** with font atlas and layout engine (400+ lines)
- **WebSocket terminal** with full Termux integration (350+ lines)
- **Hybrid archetype storage** optimized for iteration and insertion
- **System scheduler** with parallel execution and dependency graph
- **NetworkedComponent trait** for automatic replication
- **Event system** with components as events
- **Query caching** with builder pattern

🚧 **In Development**
- Systems/networking integration with core/server
- Reconnection logic in core/client
- Passkey/1Password authentication
- LSP client for rust-analyzer
- Hot-reload file watching
- Debugger interface

📋 **Next Steps**
- Integrate systems/networking with core/server channels
- Update systems/ui to use WebSocket infrastructure
- Add reconnection logic to core/client
- Integrate Passkey/1Password authentication
- Implement LSP client for rust-analyzer
- Hot-reload mechanism with file watching
- Debugger interface with breakpoints
- Vulkan renderer for compute support
- Physics system integration
- Complete systems/networking for multiplayer

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

### Docking System Implementation

The docking system (`systems/ui/src/layout/docking.rs`) provides professional IDE-style panel management:

**Core Features:**
- **DockNode Tree**: Hierarchical structure supporting splits and tabs
- **Panel Operations**: Split (horizontal/vertical), merge, close, rearrange
- **Drag & Drop**: Full drag and drop between any docks with visual feedback
- **Resize Handles**: Interactive borders between panels for resizing
- **Tab System**: Multiple panels can share same dock space as tabs
- **Serialization**: Save/load layouts to JSON for persistence
- **Responsive**: Automatic layout switching between portrait/landscape
- **Mobile Optimized**: Touch-friendly sizes, auto-collapse in portrait

**Usage Example:**
```rust
let mut docking = DockingLayout::new();

// Split dock horizontally
let (left, right) = docking.split_dock(root_id, DockOrientation::Horizontal, 0.5)?;

// Add panels as tabs
docking.add_panel(left, TabInfo {
    id: Uuid::new_v4(),
    title: "Code Editor".to_string(),
    element_id: editor_id,
    closable: true,
    icon: Some("file-code"),
})?;

// Save layout
let layout_json = docking.save_layout()?;
```

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

- **Gesture Recognition**: Full multi-touch gesture system
  - Tap, double-tap, long press detection
  - Swipe with direction and velocity tracking
  - Pinch-to-zoom and rotation gestures
  - Pan/drag with momentum
  - Fling for fast navigation
- **Gesture Element Wrapper**: Add gestures to any UI element
  - Thread-safe callbacks using Arc<RwLock>
  - Chainable API for registering handlers
  - Configurable thresholds and timings
- **Floating Toolbar**: Mobile-optimized action bar
  - Animated show/hide transitions
  - Auto-hide timer support
  - Configurable positioning
  - Touch-friendly button sizes
- **Docking Gestures**: Panel management via touch
  - Swipe to switch between tabs
  - Double-tap to maximize/restore
  - Pinch to zoom panels
  - Long press for context menus
- **Touch Targets**: Appropriately sized for mobile
- **On-screen Keyboard**: Optimized for code input

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
// Create docking layout
let mut docking = DockingLayout::new();
docking.update_orientation(screen_width, screen_height);

// Create UI elements
let chat = ChatInterface::new();
let editor = CodeEditor::new();
let file_tree = FileTree::new();
let terminal = Terminal::new();

// Add to dock system
let (left, right) = docking.split_dock(root, DockOrientation::Horizontal, 0.3)?;
docking.add_panel(left, TabInfo::new("Files", file_tree.id()));
docking.add_panel(right, TabInfo::new("Editor", editor.id()));

// Handle conversational requests
chat.on_message("show me the update loop", |chat, context| {
    let code = context.find_function("update");
    chat.show_inline_editor(code);
});

// Process input through element graph
let result = element.handle_input(&event);
if result.handled == EventHandled::Yes {
    // Event was handled by UI
}
```

### Gesture System Usage
```rust
use playground_ui::input::{GestureRecognizer, GestureExt, GestureConfig};
use playground_ui::mobile::{FloatingToolbar, ToolbarAction};

// Add gestures to any element
let mut button = Button::new("Click me")
    .with_gestures()
    .on_tap(|_| {
        println!("Tapped!");
        true
    })
    .on_long_press(|_| {
        println!("Long pressed!");
        true
    });

// Configure gesture recognition
let config = GestureConfig {
    double_tap_time: 300,
    long_press_time: 500,
    swipe_min_distance: 50.0,
    ..Default::default()
};

// Create floating toolbar
let mut toolbar = FloatingToolbar::new();
toolbar.add_action(ToolbarAction {
    id: "save".to_string(),
    icon: "save".to_string(),
    label: "Save".to_string(),
    enabled: true,
    callback: || save_file(),
});
toolbar.set_toolbar_position(ToolbarPosition::Bottom);

// Handle gestures in docking system
let mut gesture_handler = DockingGestureHandler::new();
if gesture_handler.handle_gesture(&gesture, &mut docking, position) {
    // Gesture was handled
}
```

### Important Considerations
- Always use feature flags for renderer selection
- WebGL renderer is for browser IDE only
- Vulkan will be primary renderer for production
- All rendering goes through BaseRenderer trait
- Single draw call per frame is the target
- UI system uses core/server for all communication

## Code Quality Standards

### Safety Requirements
- **NO unsafe code**: Never use `unsafe` blocks anywhere in the codebase
- **NO std::any::Any**: Avoid runtime type casting, use proper trait abstractions
- **Result everywhere**: All fallible operations return Result<T, Error>
- **Graceful degradation**: Systems should handle errors without crashing

### Module Organization
- **Clean exports**: lib.rs and mod.rs files contain ONLY module declarations and exports
- **One type per file**: Each struct/trait gets its own file (except small related types)
- **Batch operations**: APIs should operate on collections, not single items
- **Async by default**: All I/O and potentially blocking operations must be async

## ECS System Design

### Two-Layer Architecture

The ECS system is split into two layers to provide both foundational primitives and rich game development features:

#### Core/ECS (Minimal Foundation)
- **Purpose**: Basic ECS primitives that Systems can use internally
- **Features**:
  - Generational entity IDs with recycling
  - Trait-based component storage interface
  - Simple World with add/remove/query operations
  - Async/concurrent from the ground up with `tokio`
  - Runtime component registration with type erasure
  - Binary serialization using `bytes` crate
- **Used by**: Systems for internal state management

#### Systems/Logic (Full Game ECS)
- **Purpose**: Complete game development framework
- **Features**:
  - Hybrid archetype storage (optimized for iteration AND insertion)
  - Parallel system execution with dependency graph
  - Component-based events (events ARE components)
  - Builder pattern queries with caching
  - NetworkedComponent trait for automatic replication
  - Incremental GC with per-frame budget
  - Hot-reload component migration
  - Batch-only API for all operations
- **Used by**: Plugins and Apps for game logic

### Key Design Decisions

#### Async & Multithreaded Core
- Everything is async and supports multithreading from the start
- Component access is async to allow I/O operations
- System scheduling uses tokio for concurrent execution
- Thread-safe by default with proper synchronization

#### Memory Management
- Global component pool with incremental growth
- Incremental per-frame garbage collection
- Memory warnings based on growth rate analysis
- Soft-fail philosophy - Results everywhere, no panics

#### Networking Integration
- NetworkedComponent trait for automatic synchronization
- Dirty tracking with batched updates per frame
- Binary serialization matching WebSocket protocol
- User-specified networking flow via Systems

#### Hot-Reload Support
- Runtime component registration for dynamic loading
- Custom migration functions for version changes
- Automatic migration in dev mode
- Strict version checking in release mode

#### System Architecture
- Systems are stateless (Plugins/Apps may have state)
- Dependency declaration using types: `depends_on<PhysicsSystem>`
- Retry logic: Continue 3 times then halt on failure
- Safe mode disables repeatedly failing systems

### Usage Examples

#### Core/ECS (Systems Internal Use)
```rust
// Systems use core/ecs for internal state
let mut world = World::new();
let entity = world.spawn_batch([
    (Position { x: 0.0, y: 0.0 },),
    (Velocity { x: 1.0, y: 0.0 },),
]).await?;

// Simple queries for system internals
let positions = world.query::<&Position>().await?;
```

#### Systems/Logic (Plugin/App Use)
```rust
// Plugins use rich ECS API
let mut ecs = ECS::new();

// Register networked component
ecs.register_component::<Position>()
   .networked()
   .migration(|old: &v1::Position| v2::Position { 
       x: old.x, 
       y: old.y, 
       z: 0.0 
   });

// Builder pattern queries with caching
let query = ecs.query()
   .with<Position>()
   .with<Velocity>()
   .without<Dead>()
   .cached()
   .build();

// Batch operations only
ecs.spawn_batch([
    bundle!(Position::default(), Velocity::default()),
    bundle!(Position::new(10, 20), Enemy::new()),
]).await?;

// System with dependencies
ecs.add_system(physics_system)
   .depends_on<InputSystem>()
   .runs_before<RenderSystem>();
```

### Performance Considerations

- Batch operations reduce allocator pressure on mobile
- Query caching trades memory for CPU efficiency
- Component size warnings prevent cache thrashing
- Incremental GC prevents frame drops
- Hybrid storage optimizes for both iteration and insertion