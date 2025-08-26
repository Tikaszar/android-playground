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

**Alternative**: Use concrete types and serialization

### NO dyn - Concrete Types Only
**Decision**: Avoid all `dyn` trait objects

**Why**:
- Type erasure happens at compile time, not runtime
- Better performance without vtable indirection
- Clearer ownership and lifetime semantics
- Forces better architectural decisions

**Implementation** (Session 17, Corrected Session 20, Extended Session 21-23):
- Component is a concrete struct (base class pattern)
- ComponentData trait for actual component types
- Component stores data as Bytes internally for type erasure
- MessageHandlerData follows same pattern for message handlers (Session 23)
- EventData wraps event data similarly
- Query system uses component IDs directly, no Box<dyn Query>
- World::execute_query is generic: `execute_query<Q: Query>`
- MessageHandler/BroadcasterWrapper use channels for runtime behavior (Session 23)

**Pattern** (Updated Session 20 - Async):
```rust
// Base class for all components
pub struct Component {
    data: Bytes,
    component_id: ComponentId,
    component_name: String,
    size_hint: usize,
}

// Trait for actual component types (async methods)
#[async_trait]
pub trait ComponentData: Send + Sync + 'static {
    fn component_id() -> ComponentId;
    async fn serialize(&self) -> Result<Bytes, Error>;
    async fn deserialize(bytes: &Bytes) -> Result<Self, Error>;
}

// Usage (async)
let component = Component::new(MyComponentData { ... }).await?;
```

**Evolution**:
- Session 17: Initial implementation with sync methods
- Session 19: Erroneously created ComponentData struct (migration attempt)
- Session 20: Corrected by removing struct, making trait methods async
- Session 21: Applied pattern to all systems/logic files (archetype, entity, event, messaging)
- Session 22: Eliminated all TypeId usage, replaced with string-based IDs
- Session 23: Applied pattern to core/ecs messaging.rs with channel-based handlers

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

### World Architecture - Arc<World> not Shared<World>
**Decision**: UiSystem and other systems use `Arc<World>` not `Shared<World>`

**Why**:
- World already has internal Shared<> fields for its components
- Adding Shared<World> creates nested RwLock situation
- Nested locks cause deadlocks in async code
- World's methods handle their own internal locking

**Implementation**:
```rust
// WRONG - causes deadlock
pub struct UiSystem {
    world: Shared<World>,  // Arc<RwLock<World>>
}

// CORRECT
pub struct UiSystem {
    world: Arc<World>,  // World handles its own locking
}
```

**Key Principle**: If a struct has internal Shared<> fields, don't wrap it in another Shared<>

### Avoiding Lock-Holding Across Await Points
**Decision**: Never hold RwLock guards across await points

**Why**:
- Holding locks across await points causes deadlocks
- Async executors can switch tasks at await points
- Other tasks may need the same locks

**Pattern**:
```rust
// WRONG
let guard = shared_data.read().await;
let result = guard.some_async_method().await; // Holds lock across await

// CORRECT
let data = {
    let guard = shared_data.read().await;
    guard.clone() // or extract needed data
}; // Lock released here
let result = data.some_async_method().await;
```

### Storage References Use Arc
**Decision**: ComponentStorage uses Arc<dyn ComponentStorage> not Box

**Why**:
- Need to clone storage references to avoid holding locks
- Box<dyn Trait> cannot be cloned
- Arc allows shared ownership without locks

**Implementation Change**:
```rust
// OLD
storages: Shared<HashMap<ComponentId, Box<dyn ComponentStorage>>>

// NEW
storages: Shared<HashMap<ComponentId, Arc<dyn ComponentStorage>>>
```

### Handle<T> vs Shared<T> Pattern
**Decision**: Two distinct types for different concurrency patterns

**Implementation**:
```rust
// core/types/src/shared.rs
pub type Handle<T> = Arc<T>;  // External reference
pub type Shared<T> = Arc<RwLock<T>>;  // Internal state

pub fn handle<T>(value: T) -> Handle<T> {
    Arc::new(value)
}
pub fn shared<T>(value: T) -> Shared<T> {
    Arc::new(RwLock::new(value))
}
```

**Why**:
- **Handle<T>**: For external references to objects that manage their own internal locking
- **Shared<T>**: For internal mutable state within a class (private fields only)
- Prevents nested RwLock deadlocks
- Clear ownership semantics
- No `.read().await` needed when using Handle

**Usage Rules**:
- Use Handle when object has internal Shared fields
- Use Shared for simple data structures (HashMap, Vec, etc.)
- Never expose Shared fields publicly - only through methods

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
- Plugins remain self-contained with no inter-dependencies

**Evolution**:
1. Initially had Plugin trait in core/plugin (WRONG)
2. Realized this violated layering - Core shouldn't know about Plugins
3. Understood Plugins are just Systems with special loading
4. Removed core/plugin entirely (Session 26)
5. Plugins now implement systems/logic::System trait
6. All IDE plugins refactored to be self-contained Systems (Session 27)

**Implementation Pattern** (Session 27):
```rust
pub struct PluginName {
    channel_id: u16,
    systems_manager: Arc<SystemsManager>,
}

impl PluginName {
    pub fn new(systems_manager: Arc<SystemsManager>) -> Self {
        Self { channel_id: ASSIGNED_CHANNEL, systems_manager }
    }
}

#[async_trait]
impl System for PluginName {
    fn name(&self) -> &'static str { "PluginName" }
    // Standard System methods
}
```

### Plugins MUST Use systems/logic ECS
**Decision**: Plugins cannot use core/ecs directly

**Why**:
- core/ecs is for Systems' internal state management only
- Mixing ECS layers causes confusion and violations
- systems/logic provides the game ECS for plugins/apps
- Clean separation of concerns

**Implementation**:
- UiSystem uses core/ecs internally (private)
- Plugins use systems/logic World for their state
- UiInterface provides clean API for UI interaction
- No direct component manipulation across layers

**Evolution**:
1. UI Framework Plugin was using core/ecs::Component (WRONG)
2. Was trying to manipulate UiSystem's internal components
3. Created UiInterface in systems/logic for proper abstraction
4. Plugins now use high-level APIs like create_discord_layout()

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
- Apps coordinate communication between plugins

**Example** (Session 27): playground-editor app:
- Loads 9 IDE plugins (UI Framework, Editor Core, File Browser, Terminal, LSP Client, Debugger, Chat Assistant, Version Control, Theme Manager)
- Each plugin is self-contained with its own channel
- App coordinates all inter-plugin communication
- Maintains 60fps update loop for all Systems

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

### Separate core/rendering and systems/webgl
**Decision**: Split rendering into contract layer (core) and implementation layer (systems)

**Why**:
- core/rendering defines traits and commands only
- systems/webgl provides WebGL2 implementation
- systems/vulkan can be added later for native
- Clean separation of interface from implementation
- Allows multiple renderer backends

**Evolution**:
1. Initially had systems/rendering with mixed concerns
2. Created core/rendering for base contracts
3. Created systems/webgl as pure WebGL implementation
4. Removed old systems/rendering to avoid confusion

### RenderCommand Enum Design
**Decision**: Simple enum with array-based data instead of complex types

**Why**:
- Easy to serialize for network transmission
- No dependency on math libraries in core
- Simple [f32; N] arrays for positions, colors, etc.
- Reduces compilation dependencies
- Better for WASM compatibility

**Implementation**:
```rust
RenderCommand::DrawQuad {
    position: [f32; 2],
    size: [f32; 2], 
    color: [f32; 4],
}
```

### WebGL2 as Primary Target
**Decision**: Target WebGL2 instead of WebGL1 or WebGPU

**Why**:
- WebGL2 has excellent browser support (95%+)
- Supports instancing and other modern features
- WebGPU still experimental in many browsers
- Better than WebGL1's limited capabilities
- Good balance of features vs compatibility

### Vertex/Index Buffer Batching
**Decision**: Batch all geometry into large buffers

**Why**:
- Reduces draw calls dramatically
- Better mobile GPU performance
- Flush at 100 commands or frame end
- Pre-allocated 64K vertices, 192K indices
- Amortizes allocation cost

### Transform Stack Architecture
**Decision**: Matrix3 transform stack with push/pop

**Why**:
- 2D rendering needs only 3x3 matrices
- Supports hierarchical transformations
- Efficient for UI element nesting
- Matches Canvas2D mental model
- Smaller than 4x4 matrices

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

## Renderer Initialization Architecture

### Server-Controlled Renderer Initialization
**Decision**: Server sends complete renderer configuration on client connect

**Why**:
- Ensures consistent rendering across all clients
- Server can update shaders without client changes
- Supports hot-reload of rendering resources
- Centralized control over visual appearance

**Implementation**:
- RendererInit message with viewport, shaders, blend mode
- Shaders sent as source code strings
- Client compiles and caches shaders
- Resources preserved across reconnection

### Resource Caching Strategy
**Decision**: Client-side LRU cache with 100MB limit

**Why**:
- Fast reconnection without re-downloading
- Reduces network traffic on reconnect
- Automatic memory management with eviction
- Preserves compiled shaders (expensive operation)

**Cache Management**:
- LRU eviction when over limit
- Use count tracking for prioritization
- Separate caches for shaders and textures
- Clear on explicit shutdown only

### WebGL Shader Management
**Decision**: Server provides shader source, client compiles

**Why**:
- Platform-specific optimizations possible
- Easier debugging with source access
- No binary shader format issues
- Supports shader hot-reload

**Shader Pipeline**:
1. Server sends shader source in RendererInit
2. Client compiles with WebGL2 context
3. Cached with ID for reuse
4. Uniforms set per batch, not per command

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