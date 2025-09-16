# Android Playground 🎮📱

A mobile-first, plugin-based game engine designed for development entirely on Android devices using Termux. Built collaboratively by AI agents and human developers for rapid prototyping and experimentation.

## 🚀 Quick Start

### Prerequisites
- [Termux](https://termux.com/) on Android
- Rust toolchain: `pkg install rust`
- For WASM builds: `pkg install rust-std-wasm32-unknown-unknown`

### Building & Running

```bash
# Clone the repository
git clone https://github.com/Tikaszar/android-playground.git
cd android-playground

# Run the Conversational IDE (ONE COMMAND!)
cargo run -p playground-apps-editor

# Open browser to:
http://localhost:8080/playground-editor/
```

That's it! This single command starts:
- Core server with WebSocket and MCP on port 8080
- All engine systems (networking, ui, rendering, logic)
- UI Framework Plugin for browser rendering
- Mobile-optimized IDE interface

### Connecting AI Agents

AI agents (Claude Code, GPT, etc.) connect via MCP:

**Claude Code**: Create `.claude/settings.json`:
```json
{
  "mcpServers": {
    "android-playground": {
      "type": "sse",
      "url": "http://localhost:8080/mcp"
    }
  }
}
```

## 🏗️ Architecture

### 4-Layer System
```
apps/           # Complete products (IDE, games)
├── playground-editor  # Browser-based IDE
└── idle-mmo-rpg      # Future MMO game

plugins/        # Reusable feature modules
├── ui-framework      # Conversational IDE core
├── editor-core       # Text editing, vim mode
├── file-browser      # File navigation
├── terminal          # Termux integration
├── inventory         # Game inventory system
├── combat            # Combat mechanics
└── chat              # Real-time messaging

systems/        # Engine components
├── ecs         # Unified ECS World implementation (NEW)
├── logic       # Public API gateway (stateless)
├── networking  # WebSocket channels
├── ui          # UI framework
├── webgl       # WebGL2 renderer implementation
└── physics     # 2D/3D physics (planned)

core/           # Foundation layer (contracts only)
├── types       # Shared types
├── ecs         # ECS contracts/traits (no implementation)
├── server      # WebSocket + MCP server
├── client      # WASM browser client
└── ui          # UI contracts/traits
```

### Unified ECS Architecture (Session 43)

The engine uses a single, unified ECS with clean separation of concerns:

#### **core/ecs** - Contracts Only (Stateless)
Defines WHAT the ECS must do through traits and interfaces:
- `WorldContract` - Interface for World implementations
- `ComponentData` - Trait for component types
- `Storage` - Trait for storage backends
- `System` - Trait for all systems
- `Query` - Trait for query operations
- `MessageBusContract` - Interface for messaging
- Type definitions (EntityId, ComponentId, ChannelId)
- NO implementation code whatsoever

#### **systems/ecs** - Unified Implementation (Stateful)
The ONLY ECS implementation in the entire engine:
- Single authoritative World for all entities/components
- Generational entity IDs prevent use-after-free
- Component storage using Sparse/Dense enum pattern (NO dyn)
- Query system without turbofish (`.with_component(ComponentId)`)
- Three-stage execution pipeline: Update → Layout → Render
- Messaging as core ECS functionality (not a separate system)
- Incremental garbage collection with frame budget
- Implements ALL contracts from core/ecs

#### **systems/logic** - Public API Gateway (Stateless)
The ONLY interface plugins/apps can use:
- Provides public types (e.g., `UiElementComponent`)
- Translates public API calls to internal ECS operations
- Hides all implementation details
- Stateless design for clean boundaries
- All other packages are hidden from plugins/apps

### Key Design Principles
- **Stateless Core**: core/* defines contracts only, no implementation
- **Unified ECS**: Single systems/ecs World for entire engine
- **API Gateway**: systems/logic is the ONLY public API for plugins/apps
- **Clean Separation**: Contracts (core) → Implementation (systems) → API (logic)
- **Mobile-First**: Designed for touch, optimized for battery
- **Server Authority**: Browser is pure view, logic on server
- **NO unsafe code**: 100% safe Rust
- **NO dyn**: Concrete types with enum wrapper pattern
- **NO turbofish**: Use `.with_component(ComponentId)` instead of `::<T>`
- **Handle vs Shared**: Handle<T> for external refs, Shared<T> for internal state
- **Async Everything**: Built on tokio
- **Self-Contained Plugins**: No inter-plugin dependencies, App coordinates all

## 🎯 Features

### Conversational IDE
- **Discord-Style Chat**: Channels, DMs, multi-agent collaboration
- **Inline Components**: Edit code, browse files, run terminals in chat
- **Bubble States**: Collapsed/Compressed/Expanded views
- **MCP Integration**: Any LLM can connect and assist
- **Mobile Touch**: Full gesture support with floating toolbar

### Engine Capabilities
- **WebSocket Networking**: Binary protocol, 60fps batching
- **Dynamic Channel System**: Channel 0 for control, all others dynamically allocated
- **Unified ECS Framework**: Single World, staged execution, NO dyn patterns
- **WebGL Renderer**: Single draw call batching
- **Plugin System**: Plugins are high-level systems scheduled by ECS
- **WASM Support**: Browser client with reconnection

### Development Tools
- **Vim Mode**: Full vim emulation in editor
- **Syntax Highlighting**: Tree-sitter integration
- **File Browser**: Git status, lazy loading
- **Terminal**: Direct Termux process connection
- **Docking System**: VSCode-style panels
- **Multi-Cursor**: Alt-select editing

## 📊 Project Stats

- **Lines of Code**: ~46,000+
- **Packages**: 22 active (1 app, 8 IDE plugins, 7 systems, 6 core)
- **Zero Unsafe Code**: 100% safe Rust
- **Zero dyn Usage**: Enum patterns instead of trait objects
- **WASM Size**: 431KB optimized
- **Compilation**: < 30s on modern Android
- **Memory**: < 50MB baseline
- **Build Status**: ✅ **FULLY COMPILING**
- **Architecture**: ✅ **Unified ECS Implemented (Session 43)**
- **Dashboard**: Real-time terminal monitoring
- **Logging**: Component-specific log files + dashboard
- **Channel System**: Dynamic allocation (only channel 0 hardcoded)

## 🛠️ Development

### Building Individual Components
```bash
# Build everything
cargo build --workspace

# Build specific plugin
cargo build -p playground-plugins-inventory

# Build WASM client
cargo build -p playground-core-client --target wasm32-unknown-unknown --release

# Run tests
cargo test --workspace
```

### Project Structure
- **Package Naming**: `playground-{layer}-{name}`
  - Core: `playground-core-*`
  - Systems: `playground-systems-*`
  - Plugins: `playground-plugins-*`
  - Apps: `playground-apps-*`

### Architecture Rules
1. **Strict Layering**: Apps → Plugins → Systems → Core
2. **Stateless Core**: core/* provides contracts only (NO implementation)
3. **Unified ECS**: systems/ecs is the ONLY World implementation
4. **API Gateway**: Plugins/Apps use ONLY systems/logic API
5. **NO turbofish**: Use `.with_component(ComponentId)` instead
6. **NO dyn**: Use enum patterns for type erasure
7. **Handle/Shared Pattern**: Handle<T> for external, Shared<T> for internal
8. **Async Everywhere**: All I/O operations must be async
9. **ONLY tokio::sync::RwLock**: Never use parking_lot (Send issues)

## 🤝 Contributing

This is an experimental project between AI agents and developers. Each session builds on the last:

1. Read `CLAUDE.md` for architecture and rules
2. Check `CONTEXT.md` for current session state
3. Follow the architectural guidelines strictly
4. Complete implementations fully (no TODOs)
5. Update documentation for next session

### For AI Agents
- Start with CLAUDE.md and CONTEXT.md
- Use TodoWrite to plan tasks
- Mark todos completed immediately
- Maintain architectural integrity
- Document changes thoroughly

## 📚 Documentation

- `CLAUDE.md` - Architecture, rules, and AI agent guidance
- `CONTEXT.md` - Current session state and handoff notes
- Wiki - Coming soon with detailed guides

## 🎮 Roadmap

### Current Focus (2025-09-16)
- ✅ Data vs Logic separation pattern implemented (Sessions 55-56)
- ✅ core/ecs and systems/ecs fully refactored with VTable pattern
- ✅ core/console and systems/console completely rewritten
- ✅ NO dyn compliance achieved across entire codebase
- ✅ Feature flag system implemented for modular capabilities
- 🔴 Remaining systems need refactoring: ui, logic, physics, networking, webgl
- 🔴 All 9 IDE plugins need rewriting to use systems/logic API
- 🔴 Build script for automatic system registration needed
- 🔴 Plugin functionality implementation pending

### Next Up
- Fix server channel manifest response (type 8 → type 9)
- Complete Discord UI rendering pipeline
- Text rendering with Canvas/SDF
- IDE feature implementation
- Game design and mechanics (future)

### Future
- Complete game plugins
- APK packaging
- Vulkan renderer
- Physics system
- App store distribution

## 💡 Philosophy

> "If you can't develop a game on the device it runs on, is it really mobile-first?"

This project challenges conventional mobile development. We believe the future is in our pockets, and development tools should embrace that reality.

## 📄 License

This project is currently private and experimental. License details will be added when the project reaches production readiness.

---

Built with ❤️ by humans and AI in collaboration