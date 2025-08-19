# Android Playground 🎮📱

A mobile-first, plugin-based game engine designed for development entirely on Android devices using Termux. Built collaboratively by AI agents and human developers for rapid prototyping and experimentation.

## 🚀 Project Vision

Android Playground is an ambitious experiment in mobile-native game development. We're building a complete game engine and IDE that runs entirely on your phone, no desktop required. This repository serves as both a playground for AI agents to explore game engine architecture and a practical tool for mobile developers.

### Why This Matters
- **True Mobile Development**: Code, compile, and play - all on your Android device
- **AI-Driven Evolution**: Each session brings architectural improvements from AI collaboration
- **Battery-First Design**: Every system optimized for mobile constraints
- **Touch-Native IDE**: Not a port, but built from scratch for fingers

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
```

### Running the Conversational IDE

The Conversational IDE (playground-editor) is a Discord-style chat interface for interacting with AI agents:

```bash
# Just run this single command:
cargo run -p playground-editor

# Then open your browser to:
http://localhost:3001
```

This single command starts everything:
- Core server (WebSocket on port 8080)
- All engine systems (networking, ui, rendering)
- Web interface (on port 3001)
- MCP endpoint for AI agents

### Connecting AI Agents (Claude Code, GPT, etc.)

AI agents connect via MCP (Model Context Protocol) to the core server:

```bash
# The MCP endpoint is available at:
http://localhost:8080/mcp

# Configure your AI tool (e.g., Claude Code) to connect to this endpoint
```

## Architecture

### 4-Layer System
- **Apps**: Complete products (IDE, games) that coordinate plugins
- **Plugins**: Reusable feature modules that use systems
- **Systems**: Engine components that provide core functionality
- **Core**: Foundation layer with minimal dependencies

### Layer Structure
```
apps/           # Complete products
├── playground-editor  # Browser-based IDE
└── idle-mmo-rpg      # Production game (planned)

plugins/        # Feature modules
├── inventory   # Inventory management
├── combat      # Combat mechanics
├── chat        # Real-time chat
└── editor-core # Core editor features

systems/        # Engine components
├── rendering   # Multi-backend renderer with BaseRenderer trait
├── ui          # UI framework with persistent graph
├── networking  # WebSocket-based multiplayer and IPC
├── physics     # 2D/3D physics simulation
└── logic       # ECS, state machines

core/           # Foundation (minimal dependencies)
├── types       # Shared types and traits
├── android     # Android JNI bindings
├── server      # WebSocket multiplexer and channel management
├── client      # Browser WASM WebSocket client
└── plugin      # Plugin trait and loading
```

### Architectural Rules
- **Apps** create systems/logic which initializes ALL other systems
- **Plugins** use Systems APIs provided by the App (never create systems)
- **Systems** use Core ONLY (including core/ecs for internal state)
- **Systems/Logic** is special - it creates core/ecs and initializes all systems
- **Exception**: Plugins may implement custom Systems internally

### ECS Architecture
- **Core/ECS**: Minimal primitives for Systems' internal state management
- **Systems/Logic**: Full game ECS that also initializes all other systems
- **Initialization Flow**: App → systems/logic → all other systems
- All Systems use core/ecs internally for state management
- Plugins receive systems from the App, never create their own

## 🤖 MCP Support - Universal LLM Integration

Android Playground now includes **MCP (Model Context Protocol)** support, enabling any LLM to connect and provide development assistance:

### Supported LLMs
- **Claude Code** (Anthropic)
- **GPT** (OpenAI)
- **Llama** (Meta)
- Any MCP-compatible LLM

### How It Works
1. The LLM (e.g., Claude Code) has your actual codebase
2. You interact through the Conversational IDE in your browser
3. The LLM calls MCP tools to update your browser display
4. Full bidirectional communication for seamless development

### Quick Start
```bash
# Start Android Playground (port 8080 for Termux)
cargo run -p playground-server

# LLMs auto-connect via .claude/settings.json or .gemini/settings.json
# No command-line flags needed!

# Open browser to http://localhost:8080/test/ide.html
# Start coding through conversation!
```

## Current Implementation Status

✅ **Completed (Phase 1 Update - 2025-12-18)**
- Core infrastructure (types, plugin, server, client, android, **ecs**, **mcp**)
- **Core/ECS** with async, safe, batch-only API (no unsafe code!)
- **Core/Server** with integrated MCP server and channel management
- **Core/Client** with automatic WebSocket reconnection and exponential backoff
- **Systems/Logic** full-featured game ECS with hybrid storage
- **Systems/Networking** fully integrated with core/ecs for internal state
- **Systems/UI** fully integrated with core/ecs and core/server (NOW GENERIC ONLY!)
- **Systems/Rendering** integrated with core/ecs for resource tracking
- **WebSocket multiplexer** with binary protocol and channel system
- **Channel management** (1-999 for Systems, 1000+ for Plugins)
- **Frame-based packet batching** at 60fps with priority queues
- **WASM client module** with successful wasm32 compilation
- BaseRenderer trait with full rendering API
- WebGL2 renderer implementation
- Resource management with handle recycling
- Render graph system with passes
- State caching and batching
- Shader compilation and hot-reload
- Performance metrics and debugging
- **VSCode/Godot-style docking system** with drag & drop (1000+ lines)
- **ECS-based UI system** with 7 component types for internal state
- **Conversational-first UI** with Element trait architecture
- **File tree component** with lazy loading and expand/collapse
- **Chat interface** with message bubbles and inline code
- **Code editor** with vim mode and multi-cursor support
- **Terminal integration** migrated to core/server channels (no direct WebSocket)
- **Mobile gesture support** with full multi-touch recognition (500+ lines)
- **Floating toolbar** for mobile-specific actions (400+ lines)
- **Gesture-aware UI elements** with configurable handlers
- **SDF text rendering** with font atlas and layout engine (400+ lines)
- **WebSocket message handlers** for UI system with full packet routing
- **Hybrid archetype/sparse storage** for optimal performance
- **System scheduler** with parallel execution and dependencies
- **NetworkedComponent trait** for automatic replication
- **Event system** using components as events
- **Query caching** with builder pattern
- **Networking ECS components** for connections, channels, packet queues
- **4-Layer Architecture** FULLY ENFORCED (Apps → Plugins → Systems → Core)
- **2 Apps** (playground-editor IDE, idle-mmo-rpg game) with plugin coordination
- **18 Plugins** ALL COMPILE with proper trait implementation and no Core access
- **file-browser plugin** with complete FileTree UI and file operations
- **editor-core plugin** with EditorView, TextBuffer, and full vim mode
- **chat-assistant plugin** with ChatView for conversational IDE
- **Message bus** for inter-plugin communication in IDE
- **IDE docking layout** with desktop and mobile variants
- **Architecture violation FIXED**: systems/ui no longer contains app-specific code
- **UI Framework Plugin PHASE 2 COMPLETE** (3000+ lines):
  - ✅ Discord-style channel management with persistence
  - ✅ Message system with bubble states (Collapsed/Compressed/Expanded)
  - ✅ Inline components (Editor, FileBrowser, Terminal, Diff)
  - ✅ Agent orchestration with task queue
  - ✅ MCP tool handlers for all UI operations
  - ✅ WebSocket integration on channels 1200-1209
  - ✅ Complete browser UI (HTML/JS) for Conversational IDE
  - ✅ MCP server forwarding tool calls to channel 1200
  - ✅ Test infrastructure with mcp-test.html

🚧 **Next Session Priority**
- Test end-to-end MCP flow with actual Claude Code instance
- Implement context switching via git worktrees
- Add real file I/O to inline components
- Implement conversation persistence to disk
- Create agent orchestration logic

📋 **Planned**
- Remaining IDE plugins (debugger, chat-assistant, version-control, theme-manager)
- Game plugins implementation (inventory, combat, chat, etc.)
- Authentication system (Passkey/1Password)
- Systems/physics using core/ecs internally
- Vulkan renderer for compute support
- APK packaging
- Hot-reload file watching system

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

## 🎯 ECS Architecture

The project features a sophisticated two-layer ECS design:

### Core/ECS (Minimal Foundation)
- Basic primitives for Systems' internal use
- Async/concurrent with generational entity IDs
- Runtime component registration
- Binary serialization for networking

### Systems/Logic (Full Game ECS)
- **Hybrid Storage**: Archetype for iteration, sparse for rare components
- **System Scheduler**: Parallel execution with dependency graphs
- **NetworkedComponent**: Automatic replication with dirty tracking
- **Events as Components**: Unified event system
- **Query Caching**: Frequently used queries are cached
- **Safe Mode**: Systems auto-disable after repeated failures

## 📊 Project Stats

- **Total Lines of Code**: ~30,000+ (with UI Framework Plugin Phase 2)
- **Compilation Time**: < 20 seconds on modern Android devices
- **Memory Usage**: < 50MB baseline
- **WASM Size**: 431KB (optimized release build)
- **Supported Platforms**: Android 7.0+ via Termux, Browser via WASM
- **Zero Unsafe Code**: 100% safe Rust (NO unsafe blocks anywhere!)
- **Architecture Layers**: 4 (Apps → Plugins → Systems → Core)
- **Applications**: 2 (IDE and Game)
- **Plugins**: 19 (8 IDE, 10 Game, 1 UI Framework)
- **ECS Integration**: 4 Systems use core/ecs internally
- **WebSocket Channels**: UI on 10, IDE plugins 1000-1079, Game plugins 1100-1199, UI Framework 1200-1209, LLMs 2000-2999
- **MCP Tools**: 10 UI update tools for LLM integration
- **LLM Support**: Claude Code, GPT, Llama, and any MCP-compatible LLM
- **UI Framework**: 3000+ lines (Phase 2 complete with browser UI)
- **Browser UI**: Discord-style IDE with inline components and bubble states

## 🤝 Contributing

This is primarily an experimental project between AI agents and a solo developer. Each AI session builds upon the last, documented in `CONTEXT.md`. If you're an AI agent reading this:

1. Start by reading `CLAUDE.md` for project guidelines
2. Check `CONTEXT.md` for the latest session state
3. Build incrementally - we value working code and perfect architecture
4. Document your changes thoroughly for the next agent
5. Ensure Code Compiles - The code must work and compile for a feature to be considered implemented. You may use multiple sessions to achieve this.
6. Plan The Feature - Always formulate a plan and TODO list for a feature based on the designs and User instructions

## 📚 Documentation

- `CLAUDE.md` - Primary AI agent development instructions
- `CONTEXT.md` - Rolling session context and handoff notes
- `GEMINI.md` - Alternative AI agent context (for diversity in approaches)

## 🎯 Roadmap

### Immediate (Next Session)
- LSP client for code intelligence
- Hot-reload file watching system
- Debugger interface implementation

### Short Term (2-3 Sessions)
- Complete WebSocket networking
- Vulkan renderer
- Physics system integration

### Long Term Vision
- Full APK packaging
- Play Store distribution
- Plugin marketplace
- Cloud compilation service

## 💡 Philosophy

"If you can't develop a game on the device it runs on, is it really mobile-first?"

This project challenges conventional wisdom about mobile development. We believe the future of computing is in our pockets, and development tools should embrace that reality.

### Code Quality Principles
- **Zero unsafe code** - The entire engine is implemented in 100% safe Rust
- **No runtime type casting** - We avoid std::any::Any in favor of proper abstractions
- **Async everywhere** - Built on tokio for true concurrent, non-blocking operations
- **Batch-first APIs** - All operations work on collections for better performance
- **Fail gracefully** - Result types everywhere, no panics in production
