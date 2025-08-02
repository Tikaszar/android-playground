# Plugin Architecture Design

## Plugin Trait Definition

```rust
use playground_types::{Context, Event, RenderContext};

pub trait Plugin: Send + Sync + 'static {
    /// Unique identifier for the plugin
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Version string
    fn version(&self) -> &str;
    
    /// Dependencies on other plugins
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
    
    /// Called when plugin is loaded
    fn on_load(&mut self, ctx: &mut Context) -> Result<(), Box<dyn Error>>;
    
    /// Called when plugin is about to be unloaded
    fn on_unload(&mut self, ctx: &mut Context);
    
    /// Update tick (called ~60 times per second)
    fn update(&mut self, ctx: &mut Context, delta_time: f32);
    
    /// Render tick (called as needed)
    fn render(&mut self, ctx: &mut RenderContext);
    
    /// Handle events from other plugins or system
    fn on_event(&mut self, event: &Event) -> bool {
        false // Return true if event was handled
    }
}
```

## Plugin Loading

### Dynamic Library Loading
```rust
// Plugins are compiled as cdylib
// Entry point in each plugin:
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin::new())
}
```

### Hot Reload Process
1. Watch plugin directories for changes
2. On change detected:
   - Call `on_unload` for old version
   - Save plugin state
   - Unload old .so file
   - Load new .so file
   - Call `create_plugin`
   - Restore state
   - Call `on_load`

### State Preservation
```rust
pub trait Stateful {
    fn save_state(&self) -> serde_json::Value;
    fn load_state(&mut self, state: serde_json::Value);
}
```

## Plugin Communication

### Message Passing
```rust
// Send message to specific plugin
ctx.send_message("other-plugin-id", Message::Custom(data));

// Broadcast to all plugins
ctx.broadcast(Event::GameStateChanged);
```

### Shared Resources
```rust
// Register resource
ctx.register_resource::<AudioSystem>(audio);

// Access in plugin
let audio = ctx.get_resource::<AudioSystem>()?;
```

## Security Considerations
- Plugins run in same process (performance)
- Capability-based permissions
- Resource limits (memory, CPU)
- Sandboxing for untrusted plugins (future)