#!/bin/bash
set -e

echo "=== DAGASHI REBUILD ==="

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
"$SCRIPT_DIR/stop.sh"
sleep 1

# Build daemon
echo "[1/3] Building daemon..."
cd "$SCRIPT_DIR/../daemon"
cargo build --release

# Build app
echo "[2/3] Building app..."
cd "$SCRIPT_DIR/.."
pnpm tauri build --bundles app

# Install and start
echo "[3/3] Installing to /Applications..."
cp -R src-tauri/target/release/bundle/macos/Dagashi.app /Applications/

echo "Starting..."
"$SCRIPT_DIR/start.sh"
