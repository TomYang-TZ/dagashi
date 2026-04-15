use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, GenericImageView, ImageReader};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Cursor;

use crate::jikan;
use crate::klipy;
use crate::tenor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramePixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiFrame {
    pub pixels: Vec<Vec<FramePixel>>, // [row][col]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PipelineResult {
    pub frames: Vec<AsciiFrame>,
    pub cols: u32,
    pub rows: u32,
    pub source: String,        // "tenor", "klipy", or "jikan"
    pub source_url: String,    // original image URL
    pub matched_query: String, // query that found the image
}

/// A unified search result from any GIF source.
struct GifResult {
    url: String,
    title: String,
}

/// Strip season numbers, roman numerals, and other suffixes from anime titles.
/// "Overlord III" → "Overlord", "Shingeki no Kyojin Season 2" → "Shingeki no Kyojin"
fn clean_anime_title(title: &str) -> String {
    let roman = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
    let noise = ["Season", "Part", "Cour", "OVA", "ONA", "Special", "Movie",
                  "2nd", "3rd", "4th", "5th", "Final"];

    title.split_whitespace()
        .filter(|w| {
            !roman.contains(w)
                && !noise.iter().any(|n| w.eq_ignore_ascii_case(n))
                && !w.chars().all(|c| c.is_ascii_digit())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if a GIF result's title is relevant to the character/anime.
/// Requires character's first name to appear in the title.
/// Falls back to anime title match only if title has no character info.
fn is_relevant(title: &str, character_name: &str, _anime_title: &str) -> bool {
    if title.is_empty() {
        return true; // no metadata to filter on
    }
    let title_lower = title.to_lowercase();

    // Character's first name must appear in the GIF title
    let character_first = character_name.split_whitespace().next().unwrap_or("");
    if !character_first.is_empty() && title_lower.contains(&character_first.to_lowercase()) {
        return true;
    }

    eprintln!("[dagashi] Skipping irrelevant result: {:?} (wanted {} / {})", title, character_name, _anime_title);
    false
}

/// Full pipeline: search → download → extract frames → compute pixel grid.
/// Returns pixel data (brightness + color) for each cell. The frontend maps
/// brightness to characters and applies color.
pub fn fetch_frames(
    search_query: &str,
    character_name: &str,
    anime_title: &str,
    mal_id: u64,
    cols: u32,
    used_urls: &HashSet<String>,
    image_source: &str,
    klipy_api_key: Option<&str>,
) -> Result<PipelineResult, String> {
    let clean_title = clean_anime_title(anime_title);
    let queries = [
        format!("{} {}", character_name, clean_title),
        search_query.to_string(),
        format!("{} anime", character_name),
    ];

    for query in &queries {
        let results: Vec<GifResult> = match image_source {
            "klipy" => {
                let key = klipy_api_key.ok_or("klipy_api_key not set")?;
                eprintln!("[dagashi] Klipy search: {}", query);
                klipy::search_gifs(query, 10, key)
                    .into_iter()
                    .map(|r| GifResult { url: r.url, title: r.title })
                    .collect()
            }
            _ => {
                eprintln!("[dagashi] Tenor search: {}", query);
                tenor::search_gifs(query, 10)
                    .into_iter()
                    .map(|r| GifResult { url: r.url, title: r.title })
                    .collect()
            }
        };
        eprintln!("[dagashi] Got {} URLs", results.len());
        for (i, result) in results.iter().enumerate() {
            if used_urls.contains(&result.url) {
                eprintln!("[dagashi] GIF #{} already in collection, skipping", i);
                continue;
            }
            if !is_relevant(&result.title, character_name, anime_title) {
                continue;
            }
            match download(&result.url) {
                Ok(bytes) => {
                    match decode_gif(&bytes, cols) {
                        Ok(mut pr) if !pr.frames.is_empty() => {
                            eprintln!("[dagashi] Using GIF #{} ({:?}) from query: {}", i, result.title, query);
                            pr.source = image_source.to_string();
                            pr.source_url = result.url.clone();
                            pr.matched_query = query.clone();
                            return Ok(pr);
                        }
                        Ok(_) => eprintln!("[dagashi] GIF #{} had 0 frames", i),
                        Err(e) => eprintln!("[dagashi] GIF #{} decode failed: {}", i, e),
                    }
                }
                Err(e) => eprintln!("[dagashi] GIF #{} download failed: {}", i, e),
            }
        }
    }

    // Fallback: Jikan character portrait from MAL
    if let Some(url) = jikan::get_character_image(mal_id, character_name) {
        if !used_urls.contains(&url) {
            eprintln!("[dagashi] Trying Jikan image for {}", character_name);
            if let Ok(bytes) = download(&url) {
                if let Ok(frame) = decode_static_image(&bytes, cols) {
                    return Ok(PipelineResult {
                        frames: vec![frame],
                        cols,
                        rows: 0,
                        source: "jikan".to_string(),
                        source_url: url,
                        matched_query: character_name.to_string(),
                    });
                }
            }
        }
    }

    Err("no images found from any source".to_string())
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::blocking::get(url).map_err(|e| e.to_string())?;
    resp.bytes().map(|b| b.to_vec()).map_err(|e| e.to_string())
}

const MAX_FRAMES: usize = 40;

fn decode_gif(data: &[u8], cols: u32) -> Result<PipelineResult, String> {
    let cursor = Cursor::new(data);
    let decoder = GifDecoder::new(cursor).map_err(|e| e.to_string())?;

    // Collect all frames first so we can sample evenly
    let all_frames: Vec<_> = decoder
        .into_frames()
        .take(200) // hard cap to avoid huge GIFs
        .filter_map(|f| f.ok())
        .collect();

    let total = all_frames.len();
    eprintln!("[dagashi] GIF has {} total frames", total);
    let step = if total <= MAX_FRAMES { 1 } else { total / MAX_FRAMES };

    let mut ascii_frames = Vec::new();
    let mut result_rows = 0;

    for (i, frame) in all_frames.into_iter().enumerate() {
        if i % step != 0 || ascii_frames.len() >= MAX_FRAMES {
            continue;
        }
        let img = DynamicImage::ImageRgba8(frame.into_buffer());
        let af = image_to_pixel_grid(&img, cols);
        result_rows = af.pixels.len() as u32;
        ascii_frames.push(af);
    }

    Ok(PipelineResult {
        frames: ascii_frames,
        cols,
        rows: result_rows,
        source: String::new(),
        source_url: String::new(),
        matched_query: String::new(),
    })
}

fn decode_static_image(data: &[u8], cols: u32) -> Result<AsciiFrame, String> {
    let cursor = Cursor::new(data);
    let img = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    Ok(image_to_pixel_grid(&img, cols))
}

/// Convert an image to a grid of pixel data at the target column width.
/// Each cell represents a "character slot" — the frontend decides which character to render.
fn image_to_pixel_grid(img: &DynamicImage, cols: u32) -> AsciiFrame {
    let (w, h) = img.dimensions();
    let cell_w = w as f64 / cols as f64;
    let cell_h = cell_w * 2.2; // monospace chars are ~2.2x tall as wide
    let rows = (h as f64 / cell_h).max(1.0) as u32;

    let resized = img.resize_exact(cols, rows, image::imageops::FilterType::Triangle);

    let mut pixels = Vec::new();
    for y in 0..rows {
        let mut row = Vec::new();
        for x in 0..cols {
            let pixel = resized.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            // Luminance formula
            let brightness = (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) as u8;
            row.push(FramePixel { r, g, b, brightness });
        }
        pixels.push(row);
    }

    AsciiFrame { pixels }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relevance_filter() {
        // Character first name in title
        assert!(is_relevant("Gintoki Gintama: Intense Gaze", "Gintoki Sakata", "Gintama"));
        // Character first name in title (different position)
        assert!(is_relevant("Lucky Star Tsukasa cooking", "Tsukasa Hiiragi", "Lucky Star"));
        // Wrong character from same anime — must reject
        assert!(!is_relevant("Pandora's Actor Overlord pose", "Albedo", "Overlord III"));
        // Anime title match but no character name — must reject
        assert!(!is_relevant("Overlord epic scene", "Albedo", "Overlord III"));
        // Completely unrelated
        assert!(!is_relevant("Nishikata's Anime Blush", "Ichiro", "Inuyashiki"));
        // Empty title = no filtering
        assert!(is_relevant("", "Anyone", "Anything"));
    }

    #[test]
    fn clean_title_strips_season() {
        assert_eq!(clean_anime_title("Overlord III"), "Overlord");
        assert_eq!(clean_anime_title("Shingeki no Kyojin Season 2"), "Shingeki no Kyojin");
        assert_eq!(clean_anime_title("Kaguya-sama wa Kokurasetai"), "Kaguya-sama wa Kokurasetai");
        assert_eq!(clean_anime_title("Fairy Tail 2014"), "Fairy Tail");
        assert_eq!(clean_anime_title("Bleach: Sennen Kessen-hen Part 2"), "Bleach: Sennen Kessen-hen");
    }
}
