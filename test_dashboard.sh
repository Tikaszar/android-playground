#!/bin/bash
# Test dashboard display for 5 seconds
timeout 5 cargo run -p playground-apps-editor 2>&1 | head -40