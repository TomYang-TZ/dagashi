<p align="center">
  <img src="assets/dagashi-demo.gif" alt="Dagashi — Animated ASCII art from your keystrokes" width="600">
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
  <img src="https://img.shields.io/badge/usefulness-zero-C4A35A?style=flat-square" alt="Zero usefulness">
</p>

---

Dagashi is a desktop app that silently records your keystrokes all day, then at 11:59 PM uses them to generate a gacha pull of animated ASCII art from 1000+ anime series. More popular anime are rarer pulls. Your typing patterns determine which character you get.

It is completely, utterly, magnificently **useless**.

<table align="center">
<tr>
<td align="center"><strong>mono</strong></td>
<td align="center"><strong>color</strong></td>
</tr>
<tr>
<td><img src="assets/dagashi-demo.gif" alt="Monochrome ASCII art animation" width="360"></td>
<td><img src="assets/dagashi-color.gif" alt="Color ASCII art animation" width="360"></td>
</tr>
</table>

## How It Works

```
  type all day          11:59 PM              your daily pull
 ┌──────────┐      ┌──────────────┐      ┌──────────────────┐
 │ ⌘⌥⇧⌃     │      │ gacha roll   │      │  ╔══════════╗    │
 │ etaoinsr  │ ──►  │ anime pick   │ ──►  │  ║ ASCII    ║    │
 │ 48,291    │      │ GIF fetch    │      │  ║ art of   ║    │
 │ keys      │      │ ASCII render │      │  ║ Gintoki  ║    │
 └──────────┘      └──────────────┘      │  ╚══════════╝    │
                                          │  EPIC #147       │
                                          └──────────────────┘
```

1. **Type normally.** Dagashi runs in the background, counting your keystrokes. It never records what you type — only how much and which keys. Scrambled. No passwords, no credit cards, nothing reconstructable.

2. **At 11:59 PM, your daily pull triggers automatically.** Your keystroke volume determines the rarity odds. More typing = better chance of a rare pull. But even a lazy day has a shot at legendary.

3. **An AI picks your character.** Based on your typing stats — backspace ratio, shift usage, peak hours — Claude interprets your "typing personality" and picks a character + scene that matches.

4. **Animated ASCII art appears.** A GIF of that character is fetched, converted to ASCII art using your actual typed characters as pixels, and rendered in a retro pixel UI.

5. **Collect them all.** Your pull is saved to a gallery. Every day is a new pull. Rarity shifts over time as anime popularity changes.

## The Gacha

1000 anime from [MyAnimeList](https://myanimelist.net), ranked by popularity. **More popular = rarer.**

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

- **Daily auto-pull** at 11:59 PM with countdown timer
- **Mono + color** ASCII art rendered with your keystroke characters
- **1000+ anime** from MyAnimeList with popularity-based rarity
- **AI character selection** via Claude CLI — interprets your typing personality
- **Gallery** of past pulls with replay
- **Roster** showing all available anime with rarity tiers
- **Deaf mode** — one-click pause on keystroke recording
- **Retro pixel UI** — Press Start 2P font, CRT scanlines, amber-on-dark
- **All keys captured** — letters, numbers, `⌘` `⌥` `⇧` `⌃` `⏎` `⌫` arrows, function keys

## Installation

### Prerequisites

- **macOS** (Accessibility permission required for keystroke capture)
- **[Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code)** — used for AI character selection

### From Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/tomyang-dev/dagashi.git
cd dagashi
pnpm install
pnpm tauri build --bundles app

# Launch
open src-tauri/target/release/bundle/macos/Dagashi.app
```

On first launch, macOS will prompt for **Accessibility permission** — grant it so Dagashi can count your keystrokes.

### From Release (coming soon)

Download `Dagashi.app` from the [Releases](https://github.com/tomyang-dev/dagashi/releases) page and drag to `/Applications`.

## Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| App | Tauri v2 | Rust core + webview frontend |
| Backend | Rust | Keystroke capture, stats, gacha, image pipeline |
| Frontend | Vanilla JS | Retro pixel UI, ASCII renderer, gallery |
| AI | Claude CLI | Character selection from typing personality |
| Images | Giphy API | Animated GIF search for anime characters |
| Anime DB | Jikan API | 1000 anime ranked by MAL popularity |
| Keystroke | CGEventTap | macOS native key event capture |

## Why "Dagashi"?

[Dagashi](https://en.wikipedia.org/wiki/Dagashi) (駄菓子) are cheap Japanese penny candies — the kind you find in corner stores for a few yen. Worthless, nostalgic, and they bring inexplicable joy. Like this app.

The characters in Gintama literally hang out at a dagashi shop. It felt right.

## Roadmap

- [ ] Reveal animation with rarity-specific effects
- [ ] IPFS pull receipts for verifiable collection
- [ ] Server-side gacha rolls for anti-cheat
- [ ] Multiplayer leaderboard
- [ ] Mobile port (Tauri v2 supports iOS/Android)
- [ ] Nerd Font character rendering for special keys

## License

MIT
