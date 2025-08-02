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
└── rendering   # WebGL/Canvas abstraction

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

## Current Status

- Initial documentation and architecture design complete
- Repository structure created
- No implementation code yet - ready for core crate initialization