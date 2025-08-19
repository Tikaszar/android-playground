# Version Control Plugin

Git integration for the Android Playground IDE.

## Overview

The Version Control Plugin provides Git integration and version control features for the Android Playground IDE. Currently a stub implementation.

## Current Status

**Stub Implementation** - Basic plugin structure only.

## Planned Features

- Git repository management
- Commit interface
- Branch management
- Diff viewer
- Merge conflict resolution
- Push/pull operations
- Commit history viewer
- Blame annotations

## Plugin Structure

```
version-control/
├── plugin.rs    # Plugin lifecycle (stub)
└── lib.rs       # Plugin exports
```

## Channel Allocation

Reserved channels: 1080-1089 (IDE plugins range)

## Dependencies

- `playground-core-plugin`: Plugin trait
- `playground-core-types`: Core types
- `playground-systems-ui`: UI integration
- `playground-systems-networking`: Channel communication
- `async-trait`: Async plugin support

## License

See the main project LICENSE file for details.