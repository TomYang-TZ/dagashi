use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::config;
use crate::image_pipeline::PipelineResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PullMeta {
    pub date: String,
    pub character: String,
    pub scene: String,
    pub rarity: String,
    pub flavor_text: String,
    pub source: String,
    pub color_mode: String,
    pub frame_count: usize,
    pub anime_title: String,
    pub anime_rank: u32,            // popularity rank at time of pull
    pub source_url: Option<String>,    // original image URL
    pub search_query: Option<String>, // query that found the image
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collection {
    pub pulls: Vec<PullMeta>,
    pub unique_characters: HashMap<String, u32>,
}

pub fn pulls_dir() -> std::path::PathBuf {
    config::data_dir().join("pulls")
}

pub fn save_pull(
    meta: &PullMeta,
    pipeline: &PipelineResult,
) -> Result<(), String> {
    let dir = pulls_dir().join(&meta.date);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Save meta
    let meta_json = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
    fs::write(dir.join("meta.json"), meta_json).map_err(|e| e.to_string())?;

    // Save pixel frames as JSON (frontend renders the actual characters)
    let frames_json = serde_json::to_string(pipeline).map_err(|e| e.to_string())?;
    fs::write(dir.join("frames.json"), frames_json).map_err(|e| e.to_string())?;

    // Update collection index
    update_collection(meta)?;

    Ok(())
}

fn update_collection(meta: &PullMeta) -> Result<(), String> {
    let path = config::data_dir().join("collection.json");
    let mut collection = load_collection();

    collection.pulls.push(meta.clone());
    *collection
        .unique_characters
        .entry(meta.character.clone())
        .or_insert(0) += 1;

    let json = serde_json::to_string_pretty(&collection).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_collection() -> Collection {
    let path = config::data_dir().join("collection.json");
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    } else {
        Collection::default()
    }
}

pub fn recent_pull_names(n: usize) -> Vec<String> {
    let collection = load_collection();
    collection
        .pulls
        .iter()
        .rev()
        .take(n)
        .map(|p| format!("{} - {}", p.character, p.scene))
        .collect()
}

fn validate_date(date: &str) -> Result<(), String> {
    if date.contains('/') || date.contains('\\') || date.contains("..") {
        return Err("invalid date".to_string());
    }
    Ok(())
}

/// Load a specific pull's frame data for the viewer
pub fn load_pull_frames(date: &str) -> Result<PipelineResult, String> {
    validate_date(date)?;
    let path = pulls_dir().join(date).join("frames.json");
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

/// Load a specific pull's metadata
pub fn load_pull_meta(date: &str) -> Result<PullMeta, String> {
    validate_date(date)?;
    let path = pulls_dir().join(date).join("meta.json");
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}
