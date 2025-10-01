# Patterns - Code Patterns and Examples

## Module Pattern (IMPLEMENTED Sessions 68-70 - Replaces VTable)

### Pure Rust Module Interface
```rust
// NO extern "C", NO repr(C) - Pure Rust!
#[no_mangle]
pub static PLAYGROUND_MODULE: Module = Module {
    metadata: &METADATA,
    vtable: &VTABLE,
};

static VTABLE: ModuleVTable = ModuleVTable {
    create: module_create,
    destroy: module_destroy,
    call: module_call,
    save_state: module_save_state,
    restore_state: module_restore_state,
};

// Pure Rust function pointers
fn module_call(state: *mut u8, method: &str, args: &[u8]) -> Result<Vec<u8>, String> {
    // Implementation
}
```

### Module Loader (Single Unsafe) - IMPLEMENTED
```rust
// THE ONLY UNSAFE BLOCK - in modules/loader/src/loader.rs
unsafe {
    // 1. Load the dynamic library
    let library = Library::new(&module_path)?;

    // 2. Get module symbol
    let module_symbol: Symbol<*const Module> = library.get(b"PLAYGROUND_MODULE\0")?;

    // 3. Get View/ViewModel symbols
    // 4. Initialize module
    // All in ONE unsafe block!
}
```

## VTable Handler Pattern (DEPRECATED - Being Replaced)

### Correct Pattern (from systems/console)
```rust
use once_cell::sync::OnceCell;

// Global state using OnceCell (NO unsafe!)
static CONSOLE_IMPL: OnceCell<ConsoleImpl> = OnceCell::new();

struct ConsoleImpl {
    terminal: Arc<Terminal>,
    dashboard: Option<Arc<Dashboard>>,
    console_handle: Handle<Console>,
}

// Initialize once
pub async fn initialize() -> CoreResult<()> {
    let terminal = Arc::new(Terminal::new(true));
    let console_handle = Console::new();

    CONSOLE_IMPL.set(ConsoleImpl {
        terminal,
        console_handle: console_handle.clone(),
    }).map_err(|_| CoreError::AlreadyInitialized)?;

    Ok(())
}

// Get implementation
fn get_impl() -> CoreResult<&'static ConsoleImpl> {
    CONSOLE_IMPL.get().ok_or(CoreError::NotInitialized)
}

// Handle commands
pub async fn handle_output_command(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "write" => {
            let text: String = match bincode::deserialize(&payload) {
                Ok(t) => t,
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            };

            match get_impl() {
                Ok(impl_) => {
                    if let Err(e) = impl_.terminal.write(&text).await {
                        return VTableResponse {
                            success: false,
                            payload: None,
                            error: Some(e.to_string()),
                        };
                    }
                },
                Err(e) => return VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(e.to_string()),
                },
            }

            VTableResponse {
                success: true,
                payload: None,
                error: None,
            }
        },
        _ => // ... other operations
    }
}
```

### Registration Pattern
```rust
pub async fn register_handlers() -> CoreResult<()> {
    let console = playground_core_console::get_console_instance()?;

    // Create channel for operations
    let (tx, mut rx) = mpsc::channel::<VTableCommand>(100);

    // Register with VTable
    console.vtable.register("console.output".to_string(), tx).await?;

    // Spawn handler task
    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            let response = handle_output_command(cmd.operation, cmd.payload).await;
            let _ = cmd.response.send(response).await;
        }
    });

    Ok(())
}
```

## Anti-Patterns to Avoid

### WRONG: Using unsafe and static mut
```rust
// ❌ NEVER DO THIS
static mut SERVER_INSTANCE: Option<Arc<NetworkServer>> = None;

unsafe {
    SERVER_INSTANCE = Some(server.clone());
}

// ✅ DO THIS INSTEAD
static SERVER_STATE: OnceCell<ServerState> = OnceCell::new();
SERVER_STATE.set(state).map_err(|_| CoreError::AlreadyInitialized)?;
```

### WRONG: Systems importing other systems
```rust
// ❌ NEVER DO THIS
use playground_systems_webgl::WebGLRenderer;

// ✅ DO THIS INSTEAD
// Use VTable/ECS for cross-system communication
// Or forward through core packages
```

### WRONG: Implementing logic in core
```rust
// ❌ NEVER DO THIS in core/server/src/server.rs
impl Server {
    pub async fn start(&self, config: ServerConfig) -> CoreResult<()> {
        // Actual implementation logic here
        let listener = TcpListener::bind(addr).await?;
        // ...
    }
}

// ✅ DO THIS INSTEAD in core/server/src/operations.rs
impl Server {
    pub async fn start(&self, config: ServerConfig) -> CoreResult<()> {
        let payload = bincode::serialize(&config)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let response = self.vtable.send_command(
            "server",
            "start".to_string(),
            Bytes::from(payload)
        ).await?;

        if !response.success {
            return Err(CoreError::Generic(
                response.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(())
    }
}
```

## Proper Error Handling

### VTableResponse Pattern
```rust
// Helper functions for consistent responses
fn error_response(msg: String) -> VTableResponse {
    VTableResponse {
        success: false,
        payload: None,
        error: Some(msg),
    }
}

fn success_response(payload: Option<Bytes>) -> VTableResponse {
    VTableResponse {
        success: true,
        payload,
        error: None,
    }
}

// Use in handlers
pub async fn handle_operation(operation: String, payload: Bytes) -> VTableResponse {
    match operation.as_str() {
        "start" => {
            match do_start(payload).await {
                Ok(result) => success_response(Some(result)),
                Err(e) => error_response(format!("Failed to start: {}", e)),
            }
        },
        _ => error_response(format!("Unknown operation: {}", operation)),
    }
}
```

## Accessing Global Instances

### From Systems
```rust
// Get core instances
let server = playground_core_server::get_server_instance()?;
let client = playground_core_client::get_client_instance()?;

// Update their data fields
{
    let mut stats = server.stats.write().await;
    stats.total_messages_sent += 1;
}

// Read their configuration
let config = server.config.read().await.clone();
```

### From Apps/Plugins
```rust
// Use core API functions directly
playground_core_server::start_server(config).await?;
playground_core_server::send_to_connection(conn_id, message).await?;

// Or get instance for complex operations
let server = playground_core_server::get_server_instance()?;
let is_running = *server.is_running.read().await;
```

## Feature Flag Usage

### In Core Packages
```rust
pub struct Server {
    // Always present
    pub vtable: VTable,
    pub config: Shared<ServerConfig>,

    // Feature-gated
    #[cfg(feature = "channels")]
    pub channels: Shared<HashMap<ChannelId, ChannelInfo>>,

    #[cfg(feature = "batching")]
    pub message_queue: Shared<Vec<(ConnectionId, Message)>>,
}

impl Server {
    pub fn new() -> Handle<Self> {
        handle(Self {
            vtable: VTable::new(),
            config: shared(ServerConfig::default()),

            #[cfg(feature = "channels")]
            channels: shared(HashMap::new()),

            #[cfg(feature = "batching")]
            message_queue: shared(Vec::new()),
        })
    }
}
```

### In Systems
```rust
#[cfg(feature = "channels")]
pub async fn handle_channel_operations(operation: String, payload: Bytes) -> VTableResponse {
    // Channel-specific operations
}

// Registration
pub async fn register_handlers(server: Handle<Server>) -> CoreResult<()> {
    // Always register basic operations
    register_server_operations(&server).await?;

    // Conditionally register feature-specific handlers
    #[cfg(feature = "channels")]
    register_channel_operations(&server).await?;

    Ok(())
}
```

## Component Pattern (Session 71)

### Pure Data Wrapper (NO Trait)
```rust
// Concrete wrapper type - pure data only
pub struct Component {
    pub data: Bytes,
    pub component_id: ComponentId,
    pub component_name: String,
    pub size_hint: usize,
}

// Helper functions for serialization
impl Component {
    pub fn from_serializable<T: serde::Serialize + 'static>(value: &T) -> EcsResult<Self> {
        let data = bincode::serialize(value)?;
        Ok(Self {
            data: Bytes::from(data),
            component_id: ComponentId::from_type_name::<T>(),
            component_name: std::any::type_name::<T>().to_string(),
            size_hint: data.len(),
        })
    }

    pub fn to_deserializable<T: serde::de::DeserializeOwned>(&self) -> EcsResult<T> {
        bincode::deserialize(&self.data)
    }
}
```

### Consistent Model Pattern (All Modules)
Every ECS module follows this pattern:
- **{Name}Id** - Simple ID type (Copy, Eq, Hash, Serialize)
- **{Name}** - Strong reference with Handle<World>
- **{Name}Ref** - Weak reference with Weak<World>
- NO Options - use Handle/Shared or Weak directly
- NO async - pure data only