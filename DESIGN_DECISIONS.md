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

### Shared<T> Pattern for Concurrency
**Decision**: Use Shared<T> type alias for ALL concurrent access

**Implementation**:
```rust
// core/types/src/shared.rs
pub type Shared<T> = Arc<RwLock<T>>;
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(RwLock::new(value))
}
```

**Why**:
- Single source of truth for concurrent access patterns
- Cleaner API than Arc<RwLock<T>> everywhere
- Easy to audit - just search for "Shared<"
- If implementation needs to change, one place to update

**Usage**:
- Core/Systems: `use playground_core_types::{Shared, shared};`
- Plugins/Apps: `use playground_systems_logic::{Shared, shared};`

### tokio::sync::RwLock ONLY
**Decision**: Use ONLY tokio::sync::RwLock, NEVER parking_lot::RwLock

**Why**:
- parking_lot RwLock guards don't implement Send trait
- This causes compilation failures with tokio::spawn
- tokio::sync::RwLock is designed for async contexts
- Guards can be held across await points safely

**Alternative Rejected**: parking_lot::RwLock
**Why Rejected**: 
- GuardNoSend marker type prevents Send across threads
- Incompatible with async/await patterns
- Causes "cannot be sent between threads safely" errors

### NO DashMap
**Decision**: Use Shared<HashMap> instead of DashMap

**Why**:
- DashMap adds unnecessary complexity
- Shared<HashMap> with tokio::sync::RwLock is cleaner
- Consistent with our Shared<T> pattern
- Better for our async-first architecture

**Migration Pattern**:
```rust
// OLD - WRONG
use dashmap::DashMap;
let map = Arc::new(DashMap::new());
map.insert(key, value);

// NEW - CORRECT  
use playground_core_types::{Shared, shared};
let map: Shared<HashMap<K, V>> = shared(HashMap::new());
map.write().await.insert(key, value);
```

**Impact**: All functions using these collections must be async

## Plugin System

### Plugins ARE Systems
**Decision**: No separate Plugin trait - Plugins implement systems/logic::System

**Why**:
- Maintains architectural boundaries
- Core doesn't need to know about Plugins
- Plugins are just Systems to the ECS
- Cleaner abstraction layers
- Apps handle plugin loading, Systems handle execution

**Evolution**:
1. Initially had Plugin trait in core/plugin (WRONG)
2. Realized this violated layering - Core shouldn't know about Plugins
3. Understood Plugins are just Systems with special loading
4. Removed core/plugin entirely
5. Plugins now implement systems/logic::System trait

### Plugin Loading by Apps
**Decision**: Apps load plugins and register them as Systems

**Why**:
- Only Apps know about plugin libraries
- Systems just see other Systems
- Clean separation of concerns
- Apps orchestrate, Systems execute

### Apps as Authority
**Decision**: Apps are the complete authority over state and flow

**Why**:
- Apps are complete products (games, IDEs)
- Apps control plugin loading and timing
- Apps can override plugin behavior when needed
- Apps own the main update loop
- Prevents plugins from conflicting

**Example**: playground-editor app loads ui-framework plugin but maintains control over when systems run and can override plugin decisions

### Plugins as Feature Providers
**Decision**: Plugins provide reusable features using generic Systems

**Why**:
- Plugins customize generic systems for specific use cases
- Example: UI Framework Plugin uses generic systems/ui to create Discord-style interface
- Plugins are modular and reusable across different apps
- Maintains clean separation between generic capabilities and specific features

**Examples**:
- Inventory Plugin: Item management using ECS
- UI Framework Plugin: Discord chat using systems/ui
- Combat Plugin: Battle mechanics using systems/logic

### Async System Methods
**Decision**: All System lifecycle methods are async

**Why**:
- Systems may need I/O in initialization
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

## Dashboard & Monitoring Decisions

### Terminal Dashboard over Web UI
**Decision**: Real-time terminal dashboard instead of web-based monitoring

**Why**:
- Immediate visibility without browser
- Lower resource overhead
- Better for mobile development
- Clean, organized display
- Works in Termux natively

**Implementation**:
- ANSI color codes for formatting
- Emoji status indicators
- 1-second refresh rate
- Fits on single screen

### Dual Logging Strategy
**Decision**: Dashboard display + file logging

**Why**:
- Dashboard shows current state
- Files preserve full history
- Debugging needs verbose logs
- Dashboard stays clean and focused

**Implementation**:
- Recent logs in dashboard (last 10)
- Full logs to timestamped files
- logs/ directory for organization
- Automatic file rotation by session

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