#!/bin/bash

echo "=== DAGASHI START ==="

# Start daemon if not running
if pgrep -f dagashi-daemon > /dev/null; then
  echo "Daemon already running ($(pgrep -f dagashi-daemon))"
else
  echo "Starting daemon..."
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
  nohup "$SCRIPT_DIR/../daemon/target/release/dagashi-daemon" > /dev/null 2>&1 &
  echo "Daemon started (PID $!)"
fi

# Open app
echo "Opening Dagashi.app..."
open /Applications/Dagashi.app

# Start island if built
ISLAND_BIN="$SCRIPT_DIR/../island/.build/release/DagashiIsland"
if [ -f "$ISLAND_BIN" ]; then
  if ! pgrep -f DagashiIsland > /dev/null; then
    echo "Starting island..."
    nohup "$ISLAND_BIN" > /dev/null 2>&1 &
    echo "Island started (PID $!)"
  else
    echo "Island already running ($(pgrep -f DagashiIsland))"
  fi
fi
