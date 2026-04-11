"""Generate ASCII art GIFs for the README.

Uses the same rendering logic as the app:
- CLEAN mode: ' .:-=+*#%@' brightness ramp
- BLOCK mode: ' ░▒▓█' unicode block characters
- Mono: amber color with brightness-based opacity
- Color: original pixel RGB

Usage:
    python3 scripts/generate_gifs.py
"""

from PIL import Image, ImageDraw, ImageFont
import requests
import os

RAMP_CLEAN = ' .:-=+*#%@'
RAMP_BLOCK = ' ░▒▓█'

FONT_SIZE = 10
COLS = 120
CHAR_W = 6
CHAR_H = 12
BG = (10, 10, 18)
AMBER = (196, 163, 90)

try:
    font = ImageFont.truetype('/System/Library/Fonts/Menlo.ttc', FONT_SIZE)
except:
    font = ImageFont.load_default()

GIPHY_KEY = "GlVGYHkr3WSBnllca54iNt0yFbjz7L65"


def fetch_gif(query, out_path):
    """Fetch a GIF from Giphy and save to disk."""
    if os.path.exists(out_path):
        print(f"  Using cached: {out_path}")
        return
    resp = requests.get("https://api.giphy.com/v1/gifs/search", params={
        "api_key": GIPHY_KEY, "q": query, "limit": 1, "rating": "g"
    }, timeout=10)
    data = resp.json().get("data", [])
    if not data:
        print(f"  No GIF found for: {query}")
        return
    url = data[0]["images"]["original"]["url"]
    print(f"  Fetching: {url[:80]}...")
    gif_data = requests.get(url, timeout=30).content
    with open(out_path, 'wb') as f:
        f.write(gif_data)


def render_gif(gif_path, ramp, label):
    """Render a GIF into mono + color ASCII art frame lists."""
    gif = Image.open(gif_path)
    n = min(gif.n_frames, 20)
    mono_frames = []
    color_frames = []

    for fi in range(n):
        gif.seek(fi)
        rgb = gif.convert("RGB")
        gray = gif.convert("L")
        w, h = gray.size
        cell_w = w / COLS
        cell_h = cell_w * 2.2
        rows = max(1, int(h / cell_h))
        rgb_r = rgb.resize((COLS, rows))
        gray_r = gray.resize((COLS, rows))

        IMG_W = COLS * CHAR_W + 16
        IMG_H = rows * CHAR_H + 16

        mono_img = Image.new('RGB', (IMG_W, IMG_H), BG)
        color_img = Image.new('RGB', (IMG_W, IMG_H), BG)
        mono_d = ImageDraw.Draw(mono_img)
        color_d = ImageDraw.Draw(color_img)

        for y in range(rows):
            for x in range(COLS):
                brightness = gray_r.getpixel((x, y))
                r, g, b = rgb_r.getpixel((x, y))
                idx = int(brightness / 255 * (len(ramp) - 1))
                ch = ramp[min(idx, len(ramp) - 1)]
                if ch == ' ':
                    continue
                px_x = 8 + x * CHAR_W
                px_y = 8 + y * CHAR_H
                alpha = brightness / 255
                mono_d.text((px_x, px_y), ch,
                    fill=(int(AMBER[0]*alpha), int(AMBER[1]*alpha), int(AMBER[2]*alpha)),
                    font=font)
                color_d.text((px_x, px_y), ch, fill=(r, g, b), font=font)

        mono_frames.append(mono_img)
        color_frames.append(color_img)

    print(f"  {label}: {n} frames, {mono_frames[0].size}")
    return mono_frames, color_frames


def save_gif(frames, path):
    frames[0].save(path, save_all=True, append_images=frames[1:], duration=120, loop=0)
    print(f"  Saved: {path}")


def save_banner(mono, color, path):
    """Combine mono + color side by side into one synced GIF."""
    combined = []
    for m, c in zip(mono, color):
        w, h = m.size
        img = Image.new('RGB', (w * 2 + 4, h), BG)
        img.paste(m, (0, 0))
        img.paste(c, (w + 4, 0))
        combined.append(img)
    save_gif(combined, path)


def main():
    out = os.path.join(os.path.dirname(__file__), '..', 'assets')
    tmp = '/tmp'

    # Fetch source GIFs
    print("=== Fetching GIFs ===")
    fetch_gif("gintoki gintama", f"{tmp}/gintoki.gif")
    fetch_gif("kagura gintama umbrella", f"{tmp}/kagura.gif")

    # Gintoki
    print("\n=== Gintoki CLEAN ===")
    g_clean_m, g_clean_c = render_gif(f"{tmp}/gintoki.gif", RAMP_CLEAN, 'clean')
    save_gif(g_clean_m, f'{out}/dagashi-demo.gif')
    save_gif(g_clean_c, f'{out}/dagashi-color.gif')
    save_banner(g_clean_m, g_clean_c, f'{out}/dagashi-banner.gif')

    print("\n=== Gintoki BLOCK ===")
    g_block_m, g_block_c = render_gif(f"{tmp}/gintoki.gif", RAMP_BLOCK, 'block')
    save_gif(g_block_m, f'{out}/dagashi-block-mono.gif')
    save_gif(g_block_c, f'{out}/dagashi-block-color.gif')
    save_banner(g_block_m, g_block_c, f'{out}/dagashi-block-banner.gif')

    # Kagura
    print("\n=== Kagura CLEAN ===")
    k_clean_m, k_clean_c = render_gif(f"{tmp}/kagura.gif", RAMP_CLEAN, 'clean')
    save_gif(k_clean_m, f'{out}/kagura-mono.gif')
    save_gif(k_clean_c, f'{out}/kagura-color.gif')
    save_banner(k_clean_m, k_clean_c, f'{out}/kagura-banner.gif')

    print("\n=== Kagura BLOCK ===")
    k_block_m, k_block_c = render_gif(f"{tmp}/kagura.gif", RAMP_BLOCK, 'block')
    save_gif(k_block_m, f'{out}/kagura-block-mono.gif')
    save_gif(k_block_c, f'{out}/kagura-block-color.gif')
    save_banner(k_block_m, k_block_c, f'{out}/kagura-block-banner.gif')

    print("\n=== Done ===")


if __name__ == '__main__':
    main()
