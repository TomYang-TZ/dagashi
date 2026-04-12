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
    pub source: String,        // "tenor" or "jikan"
    pub source_url: String,    // original image URL
    pub matched_query: String, // query that found the image
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
    let queries = [
        search_query.to_string(),
        format!("{} {}", character_name, anime_title),
        format!("{} anime", character_name),
    ];

    for query in &queries {
        let gif_urls = match image_source {
            "klipy" => {
                let key = klipy_api_key.ok_or("klipy_api_key not set")?;
                eprintln!("[dagashi] Klipy search: {}", query);
                klipy::search_gifs(query, 10, key)
            }
            _ => {
                eprintln!("[dagashi] Tenor search: {}", query);
                tenor::search_gifs(query, 10)
            }
        };
        eprintln!("[dagashi] Got {} URLs", gif_urls.len());
        for (i, url) in gif_urls.iter().enumerate() {
            if used_urls.contains(url) {
                eprintln!("[dagashi] GIF #{} already in collection, skipping", i);
                continue;
            }
            match download(url) {
                Ok(bytes) => {
                    match decode_gif(&bytes, cols) {
                        Ok(mut result) if !result.frames.is_empty() => {
                            eprintln!("[dagashi] Using GIF #{} from query: {}", i, query);
                            result.source = image_source.to_string();
                            result.source_url = url.clone();
                            result.matched_query = query.clone();
                            return Ok(result);
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
    fn reprocess_erza() {
        let url = "https://media.tenor.com/e8yuV9CLbLEAAAAd/erza-scarlet.gif";
        let bytes = download(url).expect("download failed");
        let result = decode_gif(&bytes, 100).expect("decode failed");
        eprintln!("Erza: {} frames sampled from GIF", result.frames.len());
        assert!(!result.frames.is_empty());

        // Save to pull directory
        let dir = dirs::home_dir().unwrap().join(".dagashi/pulls/2026-04-11-13");
        let mut full_result = result;
        full_result.source_url = url.to_string();
        let json = serde_json::to_string(&full_result).unwrap();
        std::fs::write(dir.join("frames.json"), json).unwrap();
        eprintln!("Saved {} frames to {:?}", full_result.frames.len(), dir);
    }
}
