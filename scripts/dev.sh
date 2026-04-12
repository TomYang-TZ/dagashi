#!/bin/bash

echo "=== DAGASHI DEV ==="

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
"$SCRIPT_DIR/stop.sh"
sleep 1

# Build and launch island in background
echo "Building island..."
cd "$SCRIPT_DIR/../island" && swift build && .build/debug/DagashiIsland &

echo "Building and launching app from source..."
cd "$SCRIPT_DIR/.." && cargo tauri dev
