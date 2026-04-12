#!/bin/bash
set -e

echo "=== DAGASHI REBUILD ==="

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
"$SCRIPT_DIR/stop.sh"
sleep 1

# Build daemon
echo "[1/4] Building daemon..."
cd "$SCRIPT_DIR/../daemon"
cargo build --release

# Build app
echo "[2/4] Building app..."
cd "$SCRIPT_DIR/.."
pnpm tauri build --bundles app

# Build island
echo "[3/4] Building island..."
cd "$SCRIPT_DIR/../island"
swift build -c release

# Install and start
echo "[4/4] Installing to /Applications..."
cp -R "$SCRIPT_DIR/../src-tauri/target/release/bundle/macos/Dagashi.app" /Applications/

echo "Starting..."
"$SCRIPT_DIR/start.sh"
