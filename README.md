<p align="center">
  <img src="assets/dagashi-banner.gif" alt="Dagashi" width="800">
</p>

<h1 align="center">D A G A S H I</h1>

<p align="center">
  <strong>Your keystrokes, digested into anime ASCII art.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-000?style=flat-square&logo=apple" alt="macOS">
  <img src="https://img.shields.io/badge/built_with-Tauri_v2-24C8DB?style=flat-square&logo=tauri" alt="Tauri v2">
  <img src="https://img.shields.io/badge/swift-F05138?style=flat-square&logo=swift&logoColor=white" alt="Swift">
  <img src="https://img.shields.io/badge/anime-300-E91E63?style=flat-square" alt="300 anime">
</p>

---

A desktop app that counts your keystrokes, then turns them into gacha pulls of animated ASCII art from 300 top anime. More popular anime = rarer pulls.

Includes a **Dynamic Island** overlay at the macOS notch — a pixel art dagashi shop with walking characters, that expands to show your latest pull.

<p align="center">
  <img src="assets/dagashi-app-demo.gif" alt="Dagashi app demo" width="600">
</p>

<p align="center">
  <img src="assets/kakashi.gif" alt="Kakashi pull" width="600">
</p>

## Quick Start

```bash
git clone https://github.com/TomYang-TZ/dagashi.git
cd dagashi && pnpm install
./scripts/install.sh
./scripts/start.sh
```

**Requires:** macOS 14+, Rust, pnpm, [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code)

## How It Works

1. A daemon counts keystrokes (never records what you type — only counts)
2. Every hour, your stats trigger a gacha pull
3. Claude picks a character matching your typing personality
4. A GIF is fetched, converted to ASCII art, and rendered

## The Gacha

300 anime from MAL, ranked by popularity.

| Rarity | Rank | Examples |
|--------|------|----------|
| **Legendary** | #1-10 | Attack on Titan, Death Note |
| **Epic** | #11-50 | Steins;Gate, Gintama |
| **Rare** | #51-150 | Monster, Trigun |
| **Uncommon** | #151-250 | Mid-tier discoveries |
| **Common** | #251-300 | Edge of the top 300 |

## Dynamic Island

A standalone Swift app that sits at the macOS notch.

<p align="center">
  <img src="assets/island-demo.gif" alt="Dynamic Island — crowd gathers when a new pull arrives" width="600">
</p>

- **Collapsed:** pixel art dagashi shop with walkers and bikers — live weather
- **Expanded:** click to reveal latest pull as animated ASCII art
- **Auto-cycles:** color clean → color block → mono clean → mono block

<p align="center">
  <img src="assets/island-sunny.gif" alt="Dynamic Island — sunny weather" width="600">
  <img src="assets/island-cloudy.gif" alt="Dynamic Island — cloudy weather" width="600">
  <img src="assets/island-night.gif" alt="Dynamic Island at night — expanded pull view" width="600">
</p>

<p align="center">
  <img src="assets/megumi.gif" alt="Megumi pull demo" width="600">
</p>

## Scripts

```bash
./scripts/start.sh      # Start daemon + app + island
./scripts/stop.sh        # Stop everything
./scripts/dev.sh         # Build from source + launch
./scripts/rebuild.sh     # Full rebuild + install + start
```

## Architecture

Three processes, shared filesystem (`~/.dagashi/`):

| Component | Stack | Role |
|-----------|-------|------|
| **dagashi-daemon** | Rust | Keystroke stats aggregation |
| **Dagashi.app** | Tauri v2 (Rust + JS) | UI, gacha pulls, gallery |
| **DagashiIsland** | Swift/SwiftUI | Dynamic Island overlay at notch |

## Why "Dagashi"?

Dagashi (駄菓子) are cheap Japanese candy from tiny neighborhood shops. You'd walk in after school with pocket change, spin a lottery wheel, and peel open a mystery wrapper. The candy was cheap. The moment wasn't.

You type thousands of keystrokes every day. You work hard. You deserve a mystery surprise every hour.

That's Dagashi. Your keystrokes are pocket change. Every hour, the shop cashes them in and hands you a gacha pull, a character from 300 anime, rendered as animated ASCII art. Common pulls come easy. Legendaries take real effort.

The Dynamic Island at your macOS notch is the shop itself, a pixel art dagashi-ya with live weather and foot traffic. Click to open it like unwrapping a candy. Your latest pull is inside.

## Privacy

Only aggregate stats — character frequencies, hourly volume, key regions. No words, no sentences, no order. All processing happens locally on your machine — nothing is sent to any server. **Deaf mode** instantly pauses recording.

## Credits

[Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) | [MyAnimeList](https://myanimelist.net) via [Jikan API](https://jikan.moe) | [Klipy](https://klipy.com) | [Open Vibe Island](https://github.com/Octane0411/open-vibe-island)

## License

MIT
