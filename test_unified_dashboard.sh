#!/bin/bash
# Test unified dashboard - run for 10 seconds
export PLAYGROUND_DASHBOARD=true
timeout 10 cargo run -p playground-apps-editor --quiet 2>/dev/null