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
├── logic       # ECS and system initialization
├── networking  # WebSocket channels
├── ui          # UI framework
├── rendering   # Legacy renderer (deprecated)
├── webgl       # WebGL2 renderer implementation
└── physics     # 2D/3D physics (planned)

core/           # Foundation layer
├── types       # Shared types
├── ecs         # Minimal ECS for Systems
├── server      # WebSocket + MCP server
├── client      # WASM browser client
└── plugin      # Plugin system
```

### Key Design Principles
- **Mobile-First**: Designed for touch, optimized for battery
- **Hot-Reload**: Change plugins without restart
- **Server Authority**: Browser is pure view, logic on server
- **NO unsafe code**: 100% safe Rust
- **Async Everything**: Built on tokio
- **ECS Architecture**: Two-layer design (core/ecs + systems/logic)

## 🎯 Features

### Conversational IDE
- **Discord-Style Chat**: Channels, DMs, multi-agent collaboration
- **Inline Components**: Edit code, browse files, run terminals in chat
- **Bubble States**: Collapsed/Compressed/Expanded views
- **MCP Integration**: Any LLM can connect and assist
- **Mobile Touch**: Full gesture support with floating toolbar

### Engine Capabilities
- **WebSocket Networking**: Binary protocol, 60fps batching
- **Channel System**: 1-999 for Systems, 1000+ for Plugins
- **ECS Framework**: Hybrid storage, parallel execution
- **WebGL Renderer**: Single draw call batching
- **Plugin System**: Dynamic loading with hot-reload
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
- **WASM Size**: 431KB optimized
- **Compilation**: < 30s on modern Android
- **Memory**: < 50MB baseline
- **Build Status**: ✅ **FULLY COMPILING**
- **Dashboard**: Real-time terminal monitoring
- **Logging**: File + dashboard display
- **WebSocket Channels**: 
  - UI: 10
  - IDE Plugins: 1000-1079
  - Game Plugins: 1100-1199
  - UI Framework: 1200-1209
  - LLM Sessions: 2000-2999

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
1. Apps → Plugins → Systems → Core (strict layering)
2. Systems use core/ecs for internal state
3. Plugins use systems/logic for game ECS
4. No turbofish syntax - use `.with_component(ComponentId)`
5. **ONLY tokio::sync::RwLock** - Never use parking_lot (Send issues)
6. All async functions must properly propagate with .await

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

### Current Focus (2025-08-22)
- ✅ WebGL2 renderer fully working - Clear and DrawQuad commands render!
- ✅ Server-controlled renderer initialization implemented
- ✅ Resource caching with LRU eviction (100MB limit)
- ✅ Clean shutdown protocol with proper resource disposal
- 🔄 Implementing Discord-style UI layout with actual elements
- ⏳ Text rendering implementation (DrawText command)

### Next Up
- Discord UI layout with sidebar and chat
- Client lifecycle management improvements
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