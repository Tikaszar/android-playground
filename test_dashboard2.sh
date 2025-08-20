#!/bin/bash
# Test dashboard display - set environment variable and run
export PLAYGROUND_DASHBOARD=true
timeout 5 cargo run -p playground-apps-editor --quiet 2>/dev/null | head -50