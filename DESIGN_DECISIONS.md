# DESIGN_DECISIONS.md - Architectural Evolution & Decisions

This file documents key architectural decisions, why they were made, and how the architecture evolved.

## Core Architectural Decisions

### 4-Layer Architecture
**Decision**: Apps → Plugins → Systems → Core

**Why**: 
- Clear separation of concerns
- Prevents circular dependencies
- Enables hot-reload without breaking references
- Each layer has specific responsibilities

**Evolution**:
1. Started with Apps directly using Core (wrong)
2. Discovered Apps were creating systems directly (violation)
3. Realized systems/logic should initialize ALL other systems
4. Final: Apps create logic, logic creates everything else

### NO Unsafe Code
**Decision**: Zero `unsafe` blocks anywhere in codebase

**Why**:
- Mobile devices need maximum stability
- Crashes are unacceptable in production
- Rust's safety guarantees are sufficient
- Easier to maintain and debug

**Enforcement**: Compilation will fail if unsafe is used

### NO std::any::Any
**Decision**: Avoid runtime type casting entirely

**Why**:
- Type erasure breaks at runtime, not compile time
- Proper trait abstractions are cleaner
- Binary serialization is more efficient
- Maintains type safety throughout

**Alternative**: Use trait objects and serialization

### NO Turbofish Syntax
**Decision**: Use `.with_component(ComponentId)` instead of `::<T>`

**Why**:
- Turbofish syntax is verbose and error-prone
- ComponentId is more flexible for runtime registration
- Cleaner API for dynamic component systems
- Better for hot-reload scenarios

**Implementation**: All ECS queries use ComponentId

## Networking Decisions

### WebSocket-Only (No WebRTC)
**Decision**: All networking via WebSockets with binary protocol

**Why**:
- WebRTC is overly complex for our needs
- WebSocket has better mobile browser support
- Binary protocol is more efficient than JSON
- Simpler to implement and debug

**Evolution**:
1. Initially considered WebRTC for P2P
2. Realized server-authority model fits better
3. WebSocket with binary protocol chosen

### Channel-Based Architecture
**Decision**: Channel system with ID ranges

**Ranges**:
- 0: Control channel
- 1-999: Systems
- 1000-1999: Plugins
- 2000-2999: LLM sessions

**Why**:
- Clear ownership of channels
- Easy to route messages
- Supports dynamic registration
- Scales well with plugin system

### Frame-Based Batching
**Decision**: Batch packets at 60fps, never send immediately

**Why**:
- Reduces network overhead
- Consistent frame timing
- Better for mobile battery life
- Predictable performance

## ECS Architecture

### Two-Layer ECS Design
**Decision**: core/ecs (minimal) + systems/logic (full-featured)

**Why**:
- Systems need simple ECS for internal state
- Plugins/Apps need rich game development features
- Separation prevents feature creep in core
- Each layer optimized for its use case

**core/ecs Features**:
- Generational IDs
- Async operations
- Binary serialization
- Basic queries

**systems/logic Features**:
- Hybrid archetype storage
- Parallel execution
- NetworkedComponent
- Query caching
- Event system

### Batch-Only API
**Decision**: All ECS operations work on collections

**Why**:
- Better performance on mobile
- Reduces allocator pressure
- Encourages efficient code patterns
- Simplifies implementation

**Example**: `spawn_batch([...])` not `spawn()`

### Async Everything
**Decision**: All I/O operations are async

**Why**:
- Non-blocking is essential on mobile
- Better resource utilization
- Natural fit for networked games
- Tokio provides excellent runtime

## UI System Decisions

### Server-Side UI State
**Decision**: Browser is pure view, all state on server

**Why**:
- Consistency across clients
- Easier to debug and test
- No client-side state synchronization
- Supports thin clients

### Conversational IDE First
**Decision**: Chat-based interface as primary, traditional IDE secondary

**Why**:
- Better for mobile interaction
- Natural for AI collaboration
- Progressive disclosure of complexity
- More approachable for beginners

### Bubble States (Collapsed/Compressed/Expanded)
**Decision**: Three-state system for chat messages

**Why**:
- Collapsed: Minimal screen space
- Compressed: Show relevant parts
- Expanded: Full content
- Optimizes mobile screen usage

## MCP Integration

### MCP in Core/Server
**Decision**: MCP server integrated directly, not separate process

**Why**:
- Reuses existing WebSocket infrastructure
- Shares channel management system
- Better performance (no IPC)
- Simpler deployment

### Channel-Based Tool Forwarding
**Decision**: MCP tools forward to handler channels

**Why**:
- Maintains architectural boundaries
- Core doesn't know about Systems
- Plugins can register tools dynamically
- Clean separation of concerns

**Evolution**:
1. Initially tried direct tool execution in MCP
2. Realized this violated architecture (Core using Systems)
3. Changed to channel-based forwarding
4. Moved from channel 1050 to 1200 for UI Framework

## Package Naming

### Standardized Naming Convention
**Decision**: `playground-{layer}-{name}` format

**Examples**:
- playground-core-ecs
- playground-systems-ui
- playground-plugins-inventory
- playground-apps-editor

**Why**:
- Clear layer identification
- Prevents naming conflicts
- Easier to understand dependencies
- Consistent with Rust conventions

## Threading Model

### Arc<RwLock<>> Pattern
**Decision**: Use Arc<RwLock<>> consistently for shared state

**Why**:
- Thread-safe by default
- Read-heavy workloads benefit from RwLock
- Arc enables safe sharing across threads
- Standard pattern reduces cognitive load

**Alternative Considered**: DashMap
**Rejected Because**: Async borrow issues, less control

## Plugin System

### Dynamic Loading with .so Files
**Decision**: Plugins compile to shared libraries

**Why**:
- Hot-reload without restart
- Reduced compilation times
- Plugin marketplace potential
- Standard approach in game engines

### Plugin Trait with Async Methods
**Decision**: All plugin lifecycle methods are async

**Why**:
- Plugins may need I/O in initialization
- Consistent with async-everywhere principle
- Better for network operations
- Natural fit with tokio runtime

## Rendering Decisions

### BaseRenderer Trait
**Decision**: Abstract trait with WebGL/Vulkan implementations

**Why**:
- Platform flexibility
- WebGL for browser IDE
- Vulkan for production games
- Shared high-level interface

### Single Draw Call Target
**Decision**: Batch everything into one draw call

**Why**:
- Critical for mobile GPU performance
- Reduces driver overhead
- Better battery life
- Simpler render pipeline

## Development Principles

### Complete Implementations Only
**Decision**: NO TODOs, no partial implementations

**Why**:
- Partially working code is worse than no code
- Forces thinking through problems completely
- Reduces technical debt
- Makes handoffs between sessions cleaner

### Result<T, Error> Everywhere
**Decision**: All fallible operations return Result

**Why**:
- Explicit error handling
- No hidden panics
- Better error messages
- Graceful degradation

## Mobile-First Decisions

### Touch Gestures as Primary Input
**Decision**: Full gesture system, keyboard secondary

**Why**:
- Natural for mobile devices
- Supports complex interactions
- Better than porting desktop UX
- Native mobile experience

### Battery-Efficient Design
**Decision**: Every system considers power usage

**Examples**:
- Frame batching at 60fps
- Incremental GC with budgets
- Lazy loading everywhere
- Minimal background processing

**Why**:
- Mobile battery life is critical
- Users expect efficiency
- Heat generation affects performance
- Better user experience

## Future Architectural Decisions (Planned)

### APK Packaging
**Plan**: Bundle everything into standard Android APK

**Why**:
- Easy distribution via Play Store
- Standard Android deployment
- Includes all assets and plugins
- Professional distribution

### Vulkan Renderer
**Plan**: Primary renderer for production

**Why**:
- Better mobile GPU support
- Compute shader capability
- Lower overhead than OpenGL
- Modern graphics features

### Physics System
**Plan**: Start with 2D, design for 3D upgrade

**Why**:
- 2D is sufficient for many games
- Easier to implement and debug
- Clear upgrade path to 3D
- Better mobile performance