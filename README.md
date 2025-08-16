# Android Playground

A mobile-first, plugin-based game engine designed for development entirely on Android devices using Termux. Built by AI agents for rapid prototyping and experimentation.

## Purpose

This repository serves as a development environment for AI agents to build and test game engine designs. It features a multi-backend rendering system, hot-reload capabilities, and battery-efficient architecture.

## Getting Started

### Prerequisites

- [Termux](https://termux.com/) for terminal environment on Android
- [Rust](https://rustup.rs/) for building the engine and plugins
- Web browser for the development IDE

### Building the Project

```bash
# Clone the repository
git clone https://github.com/Tikaszar/android-playground.git
cd android-playground

# Build all crates
cargo build --workspace

# Build with WebGL renderer (for browser IDE)
cargo build -p playground-rendering --features webgl

# Run the development server
cargo run -p playground-server
```

## Architecture

### Core Systems
- **Plugin System**: Dynamic loading of game modules with hot-reload support
- **Rendering System**: Multi-backend renderer (WebGL implemented, Vulkan planned)
- **Type System**: Shared types ensuring plugin compatibility
- **Server**: Axum-based web server for browser IDE

### System Layers
```
core/           # Foundation (minimal dependencies)
├── types       # Shared types and traits
├── android     # Android JNI bindings
├── server      # Web server for browser editor
└── plugin      # Plugin trait and loading

systems/        # Engine components
├── rendering   # Multi-backend renderer with BaseRenderer trait
├── ui          # Immediate mode GUI / DOM rendering
├── networking  # WebSocket, WebRTC protocols
├── physics     # 2D/3D physics simulation
└── logic       # ECS, state machines

plugins/        # Games and applications
├── idle-game   # First production game
└── playground-editor  # Browser development tools
```

## Current Implementation Status

✅ **Completed**
- Core type system and plugin infrastructure
- BaseRenderer trait with full rendering API
- WebGL2 renderer implementation
- Resource management with handle recycling
- Render graph system with passes
- State caching and batching
- Shader compilation and hot-reload
- Performance metrics and debugging
- **VSCode/Godot-style docking system** with drag & drop
- **Conversational-first UI system** with Element trait architecture
- **File tree component** with lazy loading and expand/collapse
- **Chat interface** with message bubbles and inline code
- **Code editor** with vim mode and multi-cursor support
- **Terminal integration** with Termux connection
- **Mobile gesture support** with multi-touch recognition
- **Floating toolbar** for mobile-specific actions
- **Gesture-aware UI elements** with configurable handlers

🚧 **In Progress**
- Text rendering system with SDF fonts
- WebSocket connection for terminal
- Plugin hot-reload mechanism

📋 **Planned**
- Vulkan renderer for compute shaders
- Debugger interface with breakpoints
- LSP client for rust-analyzer
- Physics and networking systems
- APK packaging for distribution

## Key Features

### Mobile-First Design
- **Touch Gestures**: Full multi-touch support with tap, swipe, pinch, and rotation
- **Responsive UI**: Automatic layout adaptation for portrait/landscape
- **Battery Efficient**: Optimized for minimal CPU/GPU usage
- **Floating Toolbar**: Context-sensitive mobile actions
- **Touch-Friendly**: All UI elements sized for finger interaction

### Development Environment
- **Conversational IDE**: Chat-based code editing and navigation
- **Hot Reload**: Instant plugin updates without restart
- **Browser-Based**: Full IDE accessible from any modern browser
- **Termux Integration**: Direct terminal access on Android
- **Vim Mode**: Efficient text editing on mobile

## Technical Details

For detailed technical information, architecture decisions, and development guidelines, see:
- `CLAUDE.md` - AI agent development instructions
- `CONTEXT.md` - Current development session context
- `GEMINI.md` - Alternative AI context
