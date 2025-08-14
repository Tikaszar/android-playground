# playground-android

Android platform integration for the Android Playground game engine.

## Overview

This crate provides JNI bindings and Android-specific functionality for running the game engine on Android devices through Termux or as a native APK.

## Features

- **JNI Bindings** - Safe wrappers around Android JNI APIs
- **Logging Integration** - Android logcat output support
- **Asset Management** - Access to APK assets and resources
- **Activity Lifecycle** - Handle Android app lifecycle events
- **Touch Input** - Process touch events from Android

## Components

### JNI Module
Provides safe Rust bindings for Java Native Interface operations:
- Class loading and method invocation
- Object creation and field access
- Exception handling

### Logger Module
Integrates with Android's logging system:
- Output to logcat
- Configurable log levels
- Automatic tag generation

## Usage

```rust
use playground_android::{init_logger, jni_helpers};

// Initialize Android logging
init_logger();

// Use JNI to interact with Android APIs
let env = jni_helpers::get_jni_env();
```

## Building for Android

This crate is automatically included when building for Android targets:
- `aarch64-linux-android` (64-bit ARM)
- `armv7-linux-androideabi` (32-bit ARM)

## Dependencies

- `jni` - Rust JNI bindings
- `android_logger` - Android logging support
- `ndk` - Android NDK bindings (when building APKs)