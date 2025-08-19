# Chat Plugin

Basic chat functionality plugin for the IDE.

## Overview

The Chat Plugin provides foundational chat capabilities for the Android Playground IDE. Currently a stub implementation.

## Current Status

**Stub Implementation** - Basic plugin structure only.

## Planned Features

- Text-based chat interface
- Message history
- User presence indicators
- Channel management
- Message formatting

## Plugin Structure

```
chat/
├── plugin.rs    # Plugin lifecycle (stub)
└── lib.rs       # Plugin exports
```

## Channel Allocation

Reserved channels: 1030-1039 (IDE plugins range)

## Dependencies

- `playground-core-plugin`: Plugin trait
- `playground-core-types`: Core types
- `playground-systems-ui`: UI integration
- `playground-systems-networking`: Channel communication
- `async-trait`: Async plugin support

## License

See the main project LICENSE file for details.