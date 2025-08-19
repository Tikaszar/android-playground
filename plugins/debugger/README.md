# Debugger Plugin

Debugging interface for the Android Playground IDE.

## Overview

The Debugger Plugin provides debugging capabilities for the Android Playground IDE. Currently a stub implementation.

## Current Status

**Stub Implementation** - Basic plugin structure only.

## Planned Features

- Breakpoint management
- Step through execution
- Variable inspection
- Call stack visualization
- Watch expressions
- Debug console
- Thread debugging

## Plugin Structure

```
debugger/
├── plugin.rs    # Plugin lifecycle (stub)
└── lib.rs       # Plugin exports
```

## Channel Allocation

Reserved channels: 1060-1069 (IDE plugins range)

## Dependencies

- `playground-core-plugin`: Plugin trait
- `playground-core-types`: Core types
- `playground-systems-ui`: UI integration
- `playground-systems-networking`: Channel communication
- `async-trait`: Async plugin support

## License

See the main project LICENSE file for details.