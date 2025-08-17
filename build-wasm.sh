#!/bin/bash

# Build script for WASM client
# Builds for wasm32-unknown-unknown target

set -e

BUILD_MODE="${1:-release}"

echo "Building playground-client for wasm32-unknown-unknown..."

if [ "$BUILD_MODE" = "release" ]; then
    cargo build -p playground-client --target wasm32-unknown-unknown --release
    WASM_FILE="target/wasm32-unknown-unknown/release/playground_client.wasm"
else
    cargo build -p playground-client --target wasm32-unknown-unknown
    WASM_FILE="target/wasm32-unknown-unknown/debug/playground_client.wasm"
fi

if [ -f "$WASM_FILE" ]; then
    SIZE=$(du -h "$WASM_FILE" | cut -f1)
    echo "✅ Build successful!"
    echo "📦 WASM file: $WASM_FILE"
    echo "📏 Size: $SIZE"
    
    # Create output directory if it doesn't exist
    mkdir -p www/wasm
    
    # Copy to web directory
    cp "$WASM_FILE" "www/wasm/playground_client.wasm"
    echo "📋 Copied to: www/wasm/playground_client.wasm"
else
    echo "❌ Build failed - WASM file not found"
    exit 1
fi