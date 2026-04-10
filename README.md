<p align="center">
  <img src="assets/dagashi-banner.gif" alt="Dagashi — mono and color ASCII art side by side" width="800">
</p>

<h1 align="center">D A G A S H I</h1>

<p align="center">
  <strong>Your keystrokes, digested into anime ASCII art.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-000?style=flat-square&logo=apple" alt="macOS">
  <img src="https://img.shields.io/badge/built_with-Tauri_v2-24C8DB?style=flat-square&logo=tauri" alt="Tauri v2">
  <img src="https://img.shields.io/badge/rust-000?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/anime-1000+-E91E63?style=flat-square" alt="1000+ anime">
  <img src="https://img.shields.io/badge/dagashi-駄菓子-C4A35A?style=flat-square" alt="Dagashi">
</p>

---

Dagashi is a desktop app that silently records your keystrokes all day, then at 11:59 PM uses them to generate a gacha pull of animated ASCII art from 1000+ anime series. More popular anime are rarer pulls. Your typing patterns determine which character you get.

It won't make you more productive. It won't optimize your workflow. It's just a small, quiet thing that turns your day into something to look forward to.

<table align="center">
<tr>
<td align="center"><strong>Kagura — mono</strong></td>
<td align="center"><strong>Kagura — color</strong></td>
</tr>
<tr>
<td><img src="assets/kagura-mono.gif" alt="Kagura monochrome ASCII art" width="360"></td>
<td><img src="assets/kagura-color.gif" alt="Kagura color ASCII art" width="360"></td>
</tr>
</table>

<p align="center"><em>ASCII art rendered using your actual keystroke characters — <code>e t a o i n s r</code> become your pixels</em></p>

<p align="center">
  <img src="assets/dagashi-app.png" alt="Dagashi app — pull page with keyboard heatmap and countdown" width="720">
</p>

## How It Works

```
  Terminal (daemon)                    Dagashi.app (UI)
 ┌──────────────────┐               ┌──────────────────┐
 │ dagashi-daemon   │   writes to   │  Reads stats     │
 │                  │──────────────►│  from disk        │
 │ Captures ALL     │  ~/.dagashi/  │                  │
 │ keystrokes via   │  stats/*.json │  Shows keyboard  │
 │ CGEventTap       │               │  heatmap, pulls, │
 │                  │               │  gallery, roster  │
 └──────────────────┘               └──────────────────┘
```

1. **Type normally.** A lightweight daemon runs in your terminal, counting every keystroke globally. It never records what you type — only how much and which keys. No passwords, no credit cards, nothing reconstructable.

2. **At 11:59 PM, your daily pull triggers automatically.** Your keystroke volume determines the rarity odds. More typing = better chance of a rare pull. But even a lazy day has a shot at legendary.

3. **An AI picks your character.** Based on your typing stats — backspace ratio, shift usage, peak hours — Claude interprets your "typing personality" and picks a character + scene that matches.

4. **Animated ASCII art appears.** A GIF of that character is fetched, converted to ASCII art using your actual typed characters as pixels, and rendered in a retro pixel UI with a live keyboard heatmap.

5. **Collect them all.** Your pull is saved to a gallery. Every day is a new pull. Rarity shifts over time as anime popularity changes.

## The Gacha

1000 anime from [MyAnimeList](https://myanimelist.net) (via Jikan API), ranked by popularity. **More popular = rarer.**

| Rarity | Rank | Examples |
|--------|------|----------|
| **Legendary** | #1-25 | Attack on Titan, Death Note, One Punch Man, Naruto |
| **Epic** | #26-100 | Steins;Gate, Gintama, Mob Psycho 100, Cowboy Bebop |
| **Rare** | #101-300 | Monster, Trigun, Great Teacher Onizuka |
| **Uncommon** | #301-600 | Mid-tier shows you might discover |
| **Common** | #601-1000 | Obscure gems waiting to be found |

The anime database refreshes every 14 days. A show that's Common today could become Rare tomorrow if it blows up. Your collection's value is alive.

## Privacy

Dagashi records keystrokes but **never stores what you typed**. Only aggregate stats:

- Character frequencies (`e: 4312, t: 3100, ...`)
- Category counts (letters, numbers, symbols)
- Hourly volume patterns
- Key region heatmaps (left hand vs right hand)

No words. No sentences. No order. A **deaf mode** toggle instantly stops all recording when you're typing sensitive info.

## Features

- **Live keyboard heatmap** — see your typing patterns in real-time
- **Daily auto-pull** at 11:59 PM with countdown timer
- **Mono or color** ASCII art — color is harder to earn, based on your typing engagement
- **IPFS pull receipts** — optional verifiable proof-of-pull pinned via Pinata
- **1000+ anime** from MyAnimeList with popularity-based rarity
- **AI character selection** via Claude CLI — interprets your typing personality
- **Gallery** of past pulls with replay
- **Roster** showing all 1000 anime with rarity tiers and MAL scores
- **Deaf mode** — one-click pause on keystroke recording
- **Retro pixel UI** — Press Start 2P font, CRT scanlines, amber-on-dark
- **All keys captured** — letters, numbers, `⌘` `⌥` `⇧` `⌃` `⏎` `⌫` arrows, function keys

## Installation

### Prerequisites

- **macOS** (with Input Monitoring permission for the terminal)
- **Rust** — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **pnpm** — `npm install -g pnpm`
- **[Claude Code CLI](https://claude.ai/claude-code)** — for AI character selection during pulls

### Build & Install

```bash
# Clone
git clone https://github.com/tomyangdev/dagashi.git
cd dagashi
pnpm install

# Build and install everything
./scripts/install.sh
```

### Run

```bash
./scripts/start.sh        # Start daemon + open app
./scripts/stop.sh          # Stop everything
./scripts/restart.sh       # Stop then start
```

On first launch:
1. macOS may prompt for **Input Monitoring** permission — grant it
2. Start typing — the keyboard heatmap lights up in real-time
3. Wait for 11:59 PM — your daily pull triggers automatically

## Architecture

Two processes, shared filesystem:

| Component | Language | Role |
|-----------|----------|------|
| **dagashi-daemon** | Rust + Swift helper | Captures keystrokes globally via CGEventTap, writes stats JSON to `~/.dagashi/stats/` every 2 seconds |
| **Dagashi.app** | Rust (Tauri v2) + JS | Reads stats from disk, renders UI, runs gacha pulls, manages gallery |

```
~/.dagashi/
├── stats/
│   └── 2026-04-10.json    ← daemon writes, app reads
├── pulls/
│   └── 2026-04-10/
│       ├── meta.json       ← character, rarity, flavor text, IPFS CID
│       ├── frames.json     ← ASCII art pixel data
│       ├── receipt.json    ← IPFS pull receipt (if Pinata configured)
│       └── cid.txt         ← IPFS content identifier
├── collection.json         ← all pulls indexed
├── anime_db.json           ← 1000 anime cached from MAL
├── config.json             ← user settings
└── bin/
    └── dagashi-keytap      ← compiled Swift helper for CGEventTap
```

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Daemon | Rust + Swift | Keystroke capture via CGEventTap |
| App | Tauri v2 | Rust backend + webview frontend |
| Frontend | Vanilla JS | Retro pixel UI, ASCII renderer, keyboard heatmap |
| AI | Claude CLI | Character selection from typing personality |
| Images | Giphy API | Animated GIF search for anime characters |
| Anime DB | Jikan API | 1000 anime ranked by MAL popularity |

## Why "Dagashi"?

[Dagashi](https://en.wikipedia.org/wiki/Dagashi) (駄菓子) are cheap Japanese penny candies — the kind you find in corner stores for a few yen. They cost almost nothing. They serve no purpose. No one needs them.

But walk into a dagashi shop and watch what happens. Kids crowd around the shelves, eyes wide, agonizing over which 10-yen candy to pick. Adults stop in and suddenly they're eight years old again. The joy is completely disproportionate to the thing itself — and that's the whole point.

Dagashi aren't valuable because they're useful. They're valuable because they turn an ordinary moment into a small, unexpected delight. Cheap, accessible, unnecessary, and somehow unforgettable.

This app is the same idea. Your keystrokes become penny candies — tiny, colorful rewards that nobody asked for but everyone smiles at. The Gintama crew hangs out at a dagashi shop for a reason. Some of the best things in life cost nothing and do nothing.

## Roadmap

- [ ] Reveal animation with rarity-specific effects
- [x] IPFS pull receipts for verifiable collection
- [ ] Server-side gacha rolls for anti-cheat
- [ ] Multiplayer leaderboard and collection comparison
- [ ] Mobile port (Tauri v2 supports iOS/Android)
- [ ] Nerd Font character rendering for special keys
- [ ] Launch Agent for auto-start daemon at login

## Credits

- **[Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P)** by CodeMan38 — the pixel font that gives Dagashi its retro soul
- **[image](https://crates.io/crates/image)** crate — GIF/PNG/JPEG decoding and pixel-grid conversion for ASCII art rendering
- **[Jikan API](https://jikan.moe)** — unofficial MyAnimeList API powering the anime database
- **[Giphy API](https://developers.giphy.com)** — animated GIF search for character art
- **[Pinata](https://www.pinata.cloud)** — IPFS pinning service for verifiable pull receipts

## License

MIT
