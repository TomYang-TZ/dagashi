#!/bin/bash

echo "=== DAGASHI STOP ==="

pkill -f "Dagashi.app" 2>/dev/null && echo "App stopped" || echo "App not running"
pkill -f DagashiIsland 2>/dev/null && echo "Island stopped" || echo "Island not running"
pkill -f dagashi-daemon 2>/dev/null && echo "Daemon stopped" || echo "Daemon not running"
pkill -f dagashi-keytap 2>/dev/null && echo "Key helper stopped" || echo "Key helper not running"

echo "Done."
