# CLAUDE.md - AI Agent Memory

This file contains critical memory for Claude Code when working with this repository.

## #strict-rules
- **NO unsafe code** - Never use `unsafe` blocks anywhere
- **NO std::any::Any** - Avoid runtime type casting
- **NO super** - Explicit trait implementations only
- **NO turbofish** - Use `.with_component(ComponentId)` instead of `::<T>`
- **NO TODOs** - Complete all implementations fully
- **NO incomplete code** - Everything must compile and work
- **NO dyn** - Use concrete types/base classes instead of trait objects
- **NO enums for type erasure** - Use concrete wrapper types instead
- **Handle<T> for external refs** - Use Handle<T> (Arc<T>) for referencing objects with internal state
- **Shared<T> for internal state** - Use Shared<T> (Arc<RwLock<T>>) ONLY for private fields
- **tokio::sync::RwLock ONLY** - NEVER use parking_lot::RwLock (Send issues)
- **NO DashMap** - Use Shared<HashMap> instead
- **Result everywhere** - All fallible operations return Result<T, Error>
- **Async by default** - All I/O must be async
- **Batch operations** - APIs operate on collections, not singles
- **NO dangling imports** - Remove unused imports immediately
- **Async propagates** - If a function uses .await, it must be async
- **Files under 1000 lines** - Split large files into directories
- **lib.rs/mod.rs exports only** - No implementation code in these files

## #architecture-rules
- Apps → Plugins → Systems → Core (STRICT LAYERING)
- Apps are THE AUTHORITY - control flow, state, and timing
- **Core is stateless** - defines contracts/traits only, NO implementation
- **systems/ecs is the unified ECS** - single World for entire engine
- **systems/logic is the ONLY public API** - plugins/apps use nothing else
- Systems provide engine implementations
- Core provides contracts and interfaces only
- Plugins are high-level systems run by systems/ecs scheduler
- **App and Browser are ONE application** - distributed across network
- **Browser is the App's frontend** - not a separate entity
- **Channel 0 is control** - only hardcoded channel for discovery
- **System isolation** - systems cannot depend on each other
- **Two-stage setup** - engine systems auto-register, then apps register plugins

## #package-naming
- Core: `playground-core-*` (e.g., playground-core-ecs)
- Systems: `playground-systems-*` (e.g., playground-systems-ui)
- Plugins: `playground-plugins-*` (e.g., playground-plugins-inventory)
- Apps: `playground-apps-*` (e.g., playground-apps-editor)

## #ecs-rules
- **systems/ecs** is the unified ECS implementation for entire engine
- **core/ecs** defines contracts only (traits, no implementation)
- **systems/logic** is the ONLY public API gateway (stateless)
- Plugins/Apps can ONLY use systems/logic, nothing else
- NO playground_ecs (doesn't exist)
- Query API: `.with_component(ComponentId)` - NO TURBOFISH!
- Plugins are high-level systems scheduled by systems/ecs
- Plugins remain hot-reloadable through systems/logic API
- Engine systems (ui, webgl, etc) are isolated from each other

## #current-violations
✅ **Core Layer: NONE** - Full architectural compliance achieved in Session 51!
❌ **Systems Layer: MAJOR** - ui, logic, physics need rewrite; networking, ecs, webgl need refactor
❌ **Plugins Layer: COMPLETE REWRITE** - All 9 IDE plugins bypass systems/logic API

## #unified-ecs-architecture
**NEW DESIGN**: Single unified ECS in systems/ecs
- systems/ecs contains the ONLY World implementation
- Replaces previous dual-ECS design (core/ecs + systems/logic)
- core/ecs now defines contracts only
- systems/logic is pure API gateway (stateless)
- All systems and plugins scheduled by systems/ecs
- Staged execution pipeline (Update → Layout → Render)

## #immediate-goals
1. **Fix systems layer violations** 🔴 HIGH PRIORITY
   - Rewrite: systems/ui, systems/logic, systems/physics
   - Refactor: systems/networking, systems/ecs, systems/webgl
   - systems/console is already compliant
2. **Rewrite all IDE plugins** 🔴 BLOCKED (needs systems fixes first)
   - All 9 plugins must use ONLY systems/logic API
   - Remove all direct dependencies on other systems
   - Remove all core/* dependencies
3. **Implement dynamic channel system** 🔴 (Session 28)
   - SystemsManager channel registry
   - Channel discovery protocol on channel 0
   - Remove all hardcoded channels except 0
4. **Complete UI rendering pipeline** 🔴
   - UiSystem sends RenderCommandBatch
   - Browser receives and renders with WebGL
   - Full render loop working
5. **Implement plugin functionality** 🔴
   - Wire up actual editor functionality
   - Implement terminal PTY support
   - Add file browser operations

## #architecture-fixed
✅ Plugins ARE Systems - implement systems/logic::System trait
✅ Plugin trait doesn't exist (removed core/plugin in Session 26)
✅ NetworkingSystem starts server internally
✅ Apps only use systems/logic, never core/server directly
✅ Axum version unified to 0.8 workspace-wide
✅ All IDE plugins are self-contained (Session 27)
✅ playground-editor coordinates all plugins as the App authority

## #channel-allocation
- **0: Control channel** - ONLY hardcoded channel for discovery/registration
- **All others: Dynamic** - Assigned sequentially by SystemsManager
- **No ranges or categories** - Pure dynamic allocation
- **Browser discovers channels** - Via manifest on channel 0
- **Complete flexibility** - Add/remove components without client changes

## #key-apis
```rust
// Handle<T> for external references, Shared<T> for internal state
use playground_core_types::{Handle, handle, Shared, shared};

// External reference to object with internal state
let world: Handle<World> = handle(World::new());
world.some_method().await;  // No .read().await needed!

// Internal mutable state (private fields only)
struct MyStruct {
    data: Shared<HashMap<String, Value>>,  // INTERNAL state
}
let guard = self.data.read().await;  // or write().await

// ECS Query (NO TURBOFISH!)
query.with_component(Position::component_id())
     .with_component(Velocity::component_id())

// Plugin async trait
#[async_trait]
impl Plugin for MyPlugin {
    async fn update(&mut self, ctx: &Context) -> Result<()>
```

## #test-commands
```bash
# Run IDE (everything in one command)
cargo run -p playground-apps-editor

# Browser URL
http://localhost:8080/playground-editor/

# MCP endpoint for LLMs
http://localhost:8080/mcp
```

## #documentation-structure
- **CONTEXT.md** - Current session state only
- **HISTORY.md** - Detailed session history and bug fixes
- **DESIGN_DECISIONS.md** - Architecture evolution and why
- **README.md** - User guide, build instructions, features
- **CLAUDE.md** - This file, AI agent memory

## #implementation-status
✅ **Complete**:
- Core infrastructure (all 7 modules including core/ui)
- Systems (all 5 with ECS integration) 
- 8 IDE plugins created with structure
- WebSocket multiplexer with binary protocol (FIXED!)
- MCP server with tool system (WORKING!)
- UI Framework Plugin (3000+ lines) - NOW USES systems/logic!
- Package naming (all standardized)
- ECS Query API (no turbofish)
- Plugin async traits (all async)
- Build system (FULLY COMPILING!)
- All RwLock converted to tokio::sync
- 100+ functions made async
- Browser WebSocket connection (FIXED!)
- Terminal dashboard (unified in core/server)
- Dashboard owned by server, accessed by systems
- File logging for verbose output
- playground-editor IDE builds and runs!
- UI Framework Plugin architecture fixed (uses UiInterface)
- Render pipeline architecture established
- core/ui abstraction layer for UI systems
- Mobile Discord UI layout implementation

🟡 **Needs Testing**:
- Discord UI layout implementation  
- Multiple concurrent clients
- IDE plugin functionality

🔴 **Not Started**:
- Game plugins (inventory, combat, etc.)
- idle-mmo-rpg game design and implementation
- Production deployment

## #workflow
1. Read CONTEXT.md for current state
2. Check `git status` and recent commits
3. Review #strict-rules and #architecture-rules
4. Use TodoWrite to plan tasks
5. Complete implementations fully
6. Update CONTEXT.md with progress
7. Mark todos completed immediately

## #remember
- Always check current date with `date` command
- Check project age with `git log --reverse --format="%ai" | head -1`
- All development in Termux on Android
- Mobile-first, battery-efficient design
- Server-side authority (browser is pure view)
- Conversational IDE is primary interface
- Only the User can execute cargo run. Always include usage instructions
- ask the User to cargo run, never cargo run as an AI Agent
- No DYN or ENUMs in place of DYN
- **NEVER add migration ANYTHING - no migration code, notes, plans, or functions EVER**
- Always Read a file or directory directly, never use Search to find things.
- Read a file only once per Session, but always read a file before you write to it
- Never assume architecture, use the code as it is designed
- Diagnose an error fully, do not assume, before changing any code
- Read a file fully during a read, not partially, so it can easily be referenced later.
- No dyn, No any, No turbofish, No unsafe
- Use Shared<> for Internal, Handle<> for External
- Never use Super
- Do not cheat and search with Bash using grep, NO Searching!
- Use ccusage MCP to check your usage for the current 5-hour block. Tell me when you hit the next $10 increment or so, we can move to the next session.