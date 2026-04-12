# Dagashi Dynamic Island — Spec

## Overview

A standalone macOS Swift app that creates a Dynamic Island overlay anchored to the MacBook notch. It displays the latest anime ASCII art pull from Dagashi.

## Architecture

```
┌─────────────────────────┐     ┌─────────────────────────┐
│    Dagashi Main App     │     │    DagashiIsland App    │
│       (Tauri v2)        │     │     (Swift/SwiftUI)     │
│                         │     │                         │
│  Writes pulls to disk:  │     │  Watches filesystem:    │
│  ~/.dagashi/pulls/      │────►│  ~/.dagashi/collection  │
│  ~/.dagashi/collection  │     │                         │
│                         │     │  NSPanel at notch       │
│                         │     │  ├─ Collapsed: pixel art│
│                         │     │  └─ Expanded: WebView   │
└─────────────────────────┘     └─────────────────────────┘
         (writes)                      (reads)
```

Two independent processes. No IPC — communication via filesystem.

## States

### Collapsed (default)
- Small pill shape anchored to notch (~224×38px)
- Pixel art dagashi shop scene (daytime, idle animations)
- Pull count badge

### Expanded (on hover / new pull)
- Expands to ~680×400px
- WKWebView renders ASCII art via widget.html
- Candy wrapper cellophane overlay
- Mode cycling: color clean → color block → mono clean → mono block → original → loop
- Each mode plays all frames before transitioning

### Loading (during pull)
- Auto-expands when new pull detected
- Shows "PULLING..." animation
- Reveals new pull when frames.json appears

## Window Management

- **NSPanel** subclass with `[.borderless, .nonactivatingPanel]`
- `panel.level = .statusBar` (always on top)
- `backgroundColor = .clear`, `isOpaque = false`, `hasShadow = false`
- `collectionBehavior = [.fullScreenAuxiliary, .stationary, .canJoinAllSpaces, .ignoresCycle]`
- `hidesOnDeactivate = false`, `isMovable = false`
- Positioned: horizontally centered, pinned to screen top
- Notch detection via `NSScreen.main?.safeAreaInsets.top`

## Interactions

- **Hover near notch** → expand with spring animation
- **Click outside** → collapse
- **New pull detected** → auto-expand, show loading, reveal pull
- Global `NSEvent.addGlobalMonitorForEvents` for mouse tracking

## Custom Shape

`NotchShape` — SwiftUI `Shape` with:
- Concave top corners (mimicking notch edges)
- Convex bottom corners
- Animatable radius between closed (6/20) and opened (22/36)

## Data Flow

1. `FileWatcher` monitors `~/.dagashi/collection.json` via DispatchSource
2. On change, read latest pull's `meta.json` + `frames.json`
3. Decode into `PullMeta` + `PipelineResult` (Codable structs matching Rust)
4. Pass frame JSON to WKWebView via `evaluateJavaScript("loadFrames(...)")`
5. WebView (`widget.html`) renders ASCII art with candy wrapper overlay

## File Structure

```
island/
├── Package.swift
└── Sources/DagashiIsland/
    ├── DagashiIslandApp.swift      # @main, MenuBarExtra (no dock icon)
    ├── AppModel.swift              # Observable state
    ├── NotchPanel.swift            # NSPanel subclass
    ├── OverlayController.swift     # Window management, positioning
    ├── NotchShape.swift            # Custom animatable shape
    ├── FileWatcher.swift           # DispatchSource filesystem observer
    ├── PullData.swift              # Codable data models
    └── Views/
        ├── IslandView.swift        # Main view (collapsed/expanded switch)
        ├── CollapsedView.swift     # Pixel art idle scene
        └── ExpandedView.swift      # WKWebView wrapper

src/widget.html                     # WebView content (ASCII renderer + candy wrapper)
```

## Dependencies

None — pure Swift/SwiftUI + AppKit + WebKit. macOS 14+ target.
