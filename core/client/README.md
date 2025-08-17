# Playground Client (WASM)

Browser WebAssembly client for the Android Playground engine.

## Features

✅ **WebSocket Reconnection with Exponential Backoff**
- Automatic reconnection on connection loss
- Configurable exponential backoff (1s → 60s default)
- Maximum attempt limits
- Jitter for distributed reconnection
- Connection state callbacks

✅ **WebAssembly Support**
- Target: `wasm32-unknown-unknown`
- Maximum compatibility across all browsers
- 4GB heap limit (sufficient for browser applications)

## Reconnection Configuration

```javascript
// Default configuration
const client = new Client("ws://localhost:3000/ws");

// Custom configuration
const builder = new ClientBuilder("ws://localhost:3000/ws");
const client = builder.with_reconnect_config(
    1000,    // initial_delay_ms
    60000,   // max_delay_ms
    1.5,     // multiplier
    10,      // max_attempts (null for unlimited)
    true     // jitter
).build();

// Control reconnection
client.set_auto_reconnect(false); // Disable auto-reconnect
```

## Reconnection States

1. **Connected**: Active WebSocket connection
2. **Disconnected**: Connection lost, preparing to reconnect
3. **Reconnecting**: Actively attempting to reconnect
4. **Failed**: Maximum attempts reached or permanent failure

## Building

### Prerequisites

```bash
# In Termux, install rust with wasm support
pkg install rust

# The wasm32 target should be available, check with:
rustc --print target-list | grep wasm32-unknown-unknown
```

### Build Commands

```bash
# Build for wasm32 (release mode)
cargo build -p playground-client --target wasm32-unknown-unknown --release

# Build for wasm32 (debug mode)
cargo build -p playground-client --target wasm32-unknown-unknown

# Using build script
./build-wasm.sh release
./build-wasm.sh debug

# Using cargo alias
cargo wasm-client      # release build
cargo wasm-client-dev  # debug build
```

## Browser Support

### WASM32 Compatibility
- All modern browsers (Chrome 57+, Firefox 52+, Safari 11+, Edge 79+)
- Mobile browsers: Chrome Android, Firefox Android, Safari iOS 11+
- 4GB memory limit (more than sufficient for browser applications)

## Implementation Details

### Exponential Backoff Algorithm

```rust
delay = min(initial_delay * (multiplier ^ attempt), max_delay)
if jitter:
    delay = delay * random(0.85, 1.15)
```

### Connection Lifecycle

1. Initial connection attempt
2. On disconnect (code != 1000):
   - Start reconnection loop
   - Wait with exponential backoff
   - Attempt reconnection
   - On success: Reset state
   - On failure: Increment attempt, repeat

### Memory Management

- WASM32: 32-bit pointers, 4GB heap limit
- Uses `wee_alloc` for smaller binary size (optional)
- Efficient memory usage suitable for browser environments

## Testing

Create an HTML file to test the client:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Playground Client Test</title>
</head>
<body>
    <script type="module">
        import init, { Client } from './playground_client.js';
        
        async function run() {
            await init();
            
            const client = new Client("ws://localhost:3000/ws");
            
            // Set up reconnection callbacks
            client.set_auto_reconnect(true);
            
            try {
                await client.connect();
                console.log("Connected!");
                
                // Test reconnection by stopping server
                // Client will automatically reconnect
                
            } catch(e) {
                console.error("Connection failed:", e);
            }
        }
        
        run();
    </script>
</body>
</html>
```

## Architecture Notes

- **No unsafe code**: Entire client is 100% safe Rust
- **Async throughout**: Built on wasm-bindgen-futures
- **Binary protocol**: Efficient packet serialization
- **Channel-based**: Multiplexed communication