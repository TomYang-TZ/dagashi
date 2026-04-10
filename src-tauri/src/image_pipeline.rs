use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, GenericImageView, ImageReader};
use serde::Serialize;
use std::io::Cursor;

use crate::giphy;
use crate::wiki;

#[derive(Debug, Clone, Serialize)]
pub struct FramePixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsciiFrame {
    pub pixels: Vec<Vec<FramePixel>>, // [row][col]
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineResult {
    pub frames: Vec<AsciiFrame>,
    pub cols: u32,
    pub rows: u32,
    pub source: String, // "giphy" or "wiki"
}

/// Full pipeline: search → download → extract frames → compute pixel grid.
/// Returns pixel data (brightness + color) for each cell. The frontend maps
/// brightness to characters and applies color.
pub fn fetch_frames(
    search_query: &str,
    character_name: &str,
    cols: u32,
    giphy_api_key: Option<&str>,
    stats_seed: u64,
) -> Result<PipelineResult, String> {
    // Try Giphy first (animated GIF)
    let gif_urls = giphy::search_gifs(search_query, 5, giphy_api_key);
    if !gif_urls.is_empty() {
        let idx = (stats_seed as usize) % gif_urls.len();
        let url = &gif_urls[idx];
        if let Ok(bytes) = download(url) {
            if let Ok(frames) = decode_gif(&bytes, cols) {
                if !frames.frames.is_empty() {
                    return Ok(frames);
                }
            }
        }
    }

    // Fallback: Wiki static image
    let wiki_urls = wiki::get_character_images(character_name);
    if !wiki_urls.is_empty() {
        let idx = (stats_seed as usize) % wiki_urls.len();
        let url = &wiki_urls[idx];
        if let Ok(bytes) = download(url) {
            if let Ok(frame) = decode_static_image(&bytes, cols) {
                return Ok(PipelineResult {
                    frames: vec![frame],
                    cols,
                    rows: 0, // set below
                    source: "wiki".to_string(),
                });
            }
        }
    }

    Err("no images found from any source".to_string())
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let resp = reqwest::blocking::get(url).map_err(|e| e.to_string())?;
    resp.bytes().map(|b| b.to_vec()).map_err(|e| e.to_string())
}

fn decode_gif(data: &[u8], cols: u32) -> Result<PipelineResult, String> {
    let cursor = Cursor::new(data);
    let decoder = GifDecoder::new(cursor).map_err(|e| e.to_string())?;
    let frames = decoder.into_frames();

    let mut ascii_frames = Vec::new();
    let mut result_rows = 0;

    for (i, frame) in frames.enumerate() {
        if i >= 20 {
            break; // cap at 20 frames
        }
        let frame = frame.map_err(|e| e.to_string())?;
        let img = DynamicImage::ImageRgba8(frame.into_buffer());
        let af = image_to_pixel_grid(&img, cols);
        result_rows = af.pixels.len() as u32;
        ascii_frames.push(af);
    }

    Ok(PipelineResult {
        frames: ascii_frames,
        cols,
        rows: result_rows,
        source: "giphy".to_string(),
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
