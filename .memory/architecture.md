# Android Playground Architecture

## Core Design Principles

### 1. Mobile-First Development
- All development happens on Android device
- Optimized for Termux environment
- Touch-friendly development tools
- Battery-efficient builds

### 2. Plugin-Based Architecture
- Everything is a plugin (even core systems)
- Hot-reload without restart
- Plugins compiled to .so files
- State preservation across reloads

### 3. Microservice Crate Structure
```
core/           # Foundation (minimal deps)
  ├── types     # Shared types, no dependencies
  ├── android   # JNI bindings, Android-specific
  ├── server    # Web server, API endpoints
  └── plugin    # Plugin trait, loading mechanism

systems/        # Engine components (depend on core)
  ├── ui        # GUI framework
  ├── networking # Network protocols
  ├── physics   # Physics simulation
  ├── logic     # Game logic, ECS
  └── rendering # Graphics abstraction

plugins/        # Games/Apps (depend on core + systems)
  ├── idle-game # Production game
  └── editor    # Development tools
```

### 4. Communication Patterns
- Plugins communicate through message passing
- Shared state via core types
- Event bus for decoupled systems
- WebSocket for real-time updates

### 5. Build Strategy
- Workspace with shared dependencies
- Incremental compilation
- Plugin-specific build scripts
- APK packaging for distribution