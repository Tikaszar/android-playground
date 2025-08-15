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
├── ui          # Immediate mode GUI / DOM rendering
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

## Current Status

- Core layer implementation complete
- Plugin system functional
- Rendering system design finalized
- Ready for BaseRenderer and WebGL implementation