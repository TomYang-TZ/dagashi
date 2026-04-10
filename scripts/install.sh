#!/bin/bash
set -e

echo "=== DAGASHI INSTALL ==="

# Build daemon
echo "[1/4] Building daemon..."
cd "$(dirname "$0")/../daemon"
cargo build --release

# Build app
echo "[2/4] Building app..."
cd "$(dirname "$0")/.."
pnpm tauri build --bundles app

# Install
echo "[3/4] Installing to /Applications..."
pkill -f "Dagashi.app" 2>/dev/null || true
sleep 1
cp -R src-tauri/target/release/bundle/macos/Dagashi.app /Applications/

# Clear icon cache
echo "[4/4] Clearing icon cache..."
rm -rf ~/Library/Caches/com.apple.iconservices.store 2>/dev/null || true
find /private/var/folders -name "com.apple.iconservices*" -exec rm -rf {} + 2>/dev/null || true
killall Dock 2>/dev/null || true

echo ""
echo "Done! Run: ./scripts/start.sh"
