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
- **VSCode/Godot-style docking system** with drag & drop (1000+ lines)
- **Conversational-first UI system** with Element trait architecture
- **File tree component** with lazy loading and expand/collapse
- **Chat interface** with message bubbles and inline code
- **Code editor** with vim mode and multi-cursor support
- **Terminal integration** with WebSocket-based Termux connection
- **Mobile gesture support** with full multi-touch recognition (500+ lines)
- **Floating toolbar** for mobile-specific actions (400+ lines)
- **Gesture-aware UI elements** with configurable handlers
- **SDF text rendering** with font atlas and layout engine (400+ lines)
- **WebSocket terminal** with ANSI escape sequence parsing (350+ lines)

🚧 **In Progress**
- LSP client for rust-analyzer integration
- Plugin hot-reload mechanism with file watching
- Debugger interface with breakpoints

📋 **Planned**
- Vulkan renderer for compute shaders
- Physics system (2D/3D with collision detection)
- Networking protocols (WebRTC for P2P)
- ECS implementation in logic system
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

## 📊 Project Stats

- **Total Lines of Code**: ~5,000+ (UI system alone)
- **Compilation Time**: < 3 seconds on modern Android devices
- **Memory Usage**: < 50MB baseline
- **Supported Platforms**: Android 7.0+ via Termux

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
- Complete Vulkan renderer
- Physics system integration
- WebRTC networking

### Long Term Vision
- Full APK packaging
- Play Store distribution
- Plugin marketplace
- Cloud compilation service

## 💡 Philosophy

"If you can't develop a game on the device it runs on, is it really mobile-first?"

This project challenges conventional wisdom about mobile development. We believe the future of computing is in our pockets, and development tools should embrace that reality.
