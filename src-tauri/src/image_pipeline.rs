use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, GenericImageView, ImageReader};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Cursor;

use crate::jikan;
use crate::klipy;

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
    pub source: String,        // "klipy" or "jikan"
    pub source_url: String,    // original image URL
    pub matched_query: String, // query that found the image
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
/// Requires character's first name AND a word from the anime title to appear.
/// This prevents false positives from common names (e.g. "Madoka" matching Madoka Magica
/// when we wanted a character from a completely different anime).
fn is_relevant(title: &str, character_name: &str, anime_title: &str) -> bool {
    if title.is_empty() {
        return true; // no metadata to filter on
    }
    let title_lower = title.to_lowercase();

    // Character's first name must appear
    let character_first = character_name.split_whitespace().next().unwrap_or("");
    let has_character = !character_first.is_empty()
        && title_lower.contains(&character_first.to_lowercase());

    // At least one significant word from the anime title must also appear
    let clean_title = clean_anime_title(anime_title);
    let has_anime = clean_title
        .split_whitespace()
        .filter(|w| w.len() >= 3) // skip short words like "no", "wa", "de"
        .any(|w| title_lower.contains(&w.to_lowercase()));

    // Full name match (both first and last name in title) — accept without anime check
    let has_full_name = character_name.split_whitespace().count() >= 2
        && character_name.split_whitespace()
            .all(|part| title_lower.contains(&part.to_lowercase()));

    if has_full_name {
        return true;
    }
    // First name + anime keyword — both required to avoid common name false positives
    if has_character && has_anime {
        return true;
    }

    eprintln!("[dagashi] Skipping irrelevant result: {:?} (wanted {} / {})", title, character_name, anime_title);
    false
}

/// Full pipeline: search → download → extract frames → compute pixel grid.
/// Returns pixel data (brightness + color) for each cell. The frontend maps
/// brightness to characters and applies color.
pub fn fetch_frames(
    search_query: &str,
    character_name: &str,
    anime_title: &str,
    scene: &str,
    mal_id: u64,
    cols: u32,
    used_urls: &HashSet<String>,
    klipy_api_key: Option<&str>,
) -> Result<PipelineResult, String> {
    let clean_title = clean_anime_title(anime_title);
    // Strip character name and anime title from LLM query to get just the descriptive words
    let descriptors = search_query
        .to_lowercase()
        .replace(&character_name.to_lowercase(), "")
        .replace(&clean_title.to_lowercase(), "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Build deduplicated query list
    let mut queries = Vec::new();
    let mut seen = HashSet::new();

    let mut add = |q: String| {
        let key = q.to_lowercase();
        if seen.insert(key) { queries.push(q); }
    };

    // 1. Character name + LLM descriptors (most targeted)
    if !descriptors.is_empty() {
        add(format!("{} {}", character_name, descriptors));
    }
    // 2. Character name + first few meaningful words from scene description
    let scene_words: Vec<&str> = scene.split_whitespace()
        .filter(|w| w.len() >= 4) // skip short words
        .take(2)
        .collect();
    if !scene_words.is_empty() {
        add(format!("{} {}", character_name, scene_words.join(" ")));
    }
    // 3. Just character name
    add(character_name.to_string());
    // 4. Character name + anime title (fallback)
    add(format!("{} {}", character_name, clean_title));

    for query in &queries {
        let key = klipy_api_key.ok_or("klipy_api_key not set")?;
        eprintln!("[dagashi] Klipy search: {}", query);
        let results = klipy::search_gifs(query, 10, key);
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
                            pr.source = "klipy".to_string();
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
    let cell_h = cell_w * 2.0;
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
