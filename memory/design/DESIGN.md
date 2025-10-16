# Design - Long-term Vision and Goals

## Project Vision

A mobile-first game engine and IDE that runs entirely on Android devices (via Termux), providing a complete development environment with conversational AI assistance, designed for battery efficiency and mobile constraints.

## Target Architecture

### Complete Engine Stack
```
┌─────────────────────────────────────┐
│            Apps Layer               │
│  (playground-editor, mmo-rpg, etc) │
├─────────────────────────────────────┤
│          Plugins Layer              │
│  (IDE plugins, game features, etc)  │
├─────────────────────────────────────┤
│          Systems Layer              │
│  (Concrete implementations)         │
├─────────────────────────────────────┤
│           Core Layer                │
│  (Contracts and data structures)    │
└─────────────────────────────────────┘
```

## Core Design Principles

### Mobile-First Design
- Battery efficiency as primary constraint
- Touch gestures as primary input
- Small screen optimization
- Termux-native development
- Offline-first capabilities

### Server-Side Authority
- Browser is pure view layer
- All state managed server-side
- Thin client architecture
- WebSocket binary protocol
- Frame-based batching at 60fps

### Conversational IDE
- Chat-based interface primary
- Traditional IDE secondary
- AI collaboration built-in
- Progressive disclosure
- Mobile-friendly interactions

### Zero Runtime Failures
- All errors at compile time when possible
- **Build-Time Validation**: An App's `build.rs` script validates that chosen `System` modules meet all `Core` feature requirements before compilation, preventing runtime dependency errors.
- NO unsafe code anywhere (except single Library::new)
- NO runtime type casting
- NO dyn (except modules/* for hot-loading via Arc<dyn Trait>) - Session 79
- Result<T, Error> everywhere
- Graceful degradation

## Planned Features

### Rendering Systems
- **WebGL2** - Current browser target (systems/webgl)
- **Vulkan** - Future native Android renderer (systems/vulkan)
- **Software** - Fallback renderer (systems/software)
- All implement core/rendering contracts

### Networking Systems
- **WebSocket** - Current implementation (systems/networking)
- **TCP/UDP** - Raw socket support (future)
- **WebRTC** - P2P capabilities (future)
- **IPC** - Local process communication (future)

### Platform Support
- **Android/Termux** - Primary platform
- **Browser/WASM** - Current client
- **Native Linux** - Future support
- **Native Windows** - Future support

### Physics System
- Start with 2D physics
- Design for 3D upgrade path
- Mobile-optimized performance
- Integrate with ECS

### Audio System
- Spatial audio support
- Compressed formats
- Streaming capabilities
- Mobile battery efficiency

## Performance Targets

### Memory Usage
- Base engine < 50MB
- With plugins < 100MB
- WASM client < 500KB
- Incremental GC with budgets

### Compilation
- Full rebuild < 30 seconds on mobile
- Incremental < 5 seconds
- Hot reload for plugins
- Parallel compilation

### Runtime Performance
- Stable 60fps on mobile
- < 2ms frame budget for engine
- Batch operations everywhere
- Zero allocations in hot paths

### Battery Efficiency
- Minimal background processing
- Frame batching reduces network
- Lazy loading everywhere
- Configurable quality levels

## API Design Goals

### Developer Experience
- Clean, intuitive APIs
- Compile-time feature discovery
- Self-documenting code
- Minimal boilerplate

### Extensibility
- Plugin architecture
- Hot-swappable systems
- Feature flags for capabilities
- Runtime feature detection

### Type Safety
- No runtime casting (except Any for downcasting trait objects)
- No dyn (except modules/* for Arc<dyn Trait> hot-loading) - Session 79
- Compile-time guarantees
- Direct trait method signatures (no serialization)
- Strong typing throughout

## Long-term Goals

### Package Distribution
- APK packaging for Android
- Play Store distribution
- Plugin marketplace
- Asset store integration

### Development Tools
- Visual debugging tools
- Performance profilers
- Asset pipeline
- Build automation

### AI Integration
- MCP server built-in
- Multiple AI providers
- Tool system extensible
- Conversational debugging

### Community Features
- Multiplayer support
- Social features
- Cloud saves
- Analytics integration

## Architecture Decisions

### Why MVVM (Model-View-ViewModel)?
- **Model** (Core): Pure data structures, no logic
- **View** (Core): API contracts as traits, no implementation
- **ViewModel** (Systems): Trait implementations with business logic
- Enables hot-swapping implementations via trait objects
- Maintains compile-time type safety within each layer
- Allows multiple backends (e.g., WebGL vs Vulkan)

### Why Trait-Based Hot-Loading? (Session 79)
- **Direct trait method calls** - ~1-5ns overhead (just vtable dispatch)
- **No serialization** - Parameters passed directly, not as bytes
- **Everything reloadable** - Core, Systems, Plugins, and Apps
- **Single unsafe exception** - Only Library::new() needed
- **Pure Rust interfaces** - No C ABI or extern "C"
- **Arc<dyn Trait>** - The ONLY allowed use of dyn for hot-loading
- **State Preservation (Session 81 Design)** - An optional `StatefulModule` trait allows `ViewModels` to serialize their state before being unloaded and restore it after a new version is loaded.
- **Self-modifying** - IDE can reload itself while running
- **Concurrent Registry** - The central `BindingRegistry` is designed for high-throughput, non-blocking reads and concurrent writes, enabling facades like `systems/ecs` to be simple, performant pass-through layers.

### Why Feature Flags?
- Compile-time optimization
- Reduced binary size
- Platform-specific builds
- Optional capabilities
- Dependency negotiation
- Minimal module loading

### Why Message Bus for Cold Paths?
- Module discovery at runtime
- Complex inter-module operations
- Debugging and inspection
- MCP tool integration

## Success Metrics

### Technical
- Zero unsafe code
- Zero runtime panics
- 100% Result usage
- Full async/await

### Performance
- 60fps on Pixel 8 Pro
- < 100MB memory usage
- < 10% CPU usage idle
- 8+ hour battery life

### Usability
- 5 minute setup time
- Intuitive chat interface
- Touch-friendly UI
- Offline development

## Non-Goals

### What We Don't Support
- Desktop-first workflows
- Synchronous APIs
- Runtime plugin loading (for now)
- Cross-platform UI (each platform native)

### What We Don't Optimize For
- Maximum performance over battery life
- Feature completeness over stability
- Desktop development over mobile
- Traditional over conversational UI