#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod gacha;
mod giphy;
mod image_pipeline;
mod keylogger;
mod llm;
mod stats;
mod storage;
mod wiki;

use std::sync::Mutex;
use tauri::State;

struct AppState {
    stats: stats::SharedStats,
    config: Mutex<config::Config>,
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> stats::DailyStats {
    state.stats.lock().unwrap().clone()
}

#[tauri::command]
fn get_config(state: State<AppState>) -> config::Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config_cmd(state: State<AppState>, new_config: config::Config) -> Result<(), String> {
    config::save_config(&new_config)?;
    *state.config.lock().unwrap() = new_config;
    Ok(())
}

#[tauri::command]
fn toggle_deaf_mode(state: State<AppState>) -> bool {
    let deaf = !keylogger::is_deaf();
    keylogger::set_deaf_mode(deaf);
    let mut cfg = state.config.lock().unwrap();
    cfg.keystroke_capture.deaf_mode = deaf;
    config::save_config(&cfg).ok();
    deaf
}

#[tauri::command]
fn get_collection() -> storage::Collection {
    storage::load_collection()
}

#[tauri::command]
fn load_pull_frames(date: String) -> Result<image_pipeline::PipelineResult, String> {
    storage::load_pull_frames(&date)
}

#[tauri::command]
fn do_pull(state: State<AppState>) -> Result<storage::PullMeta, String> {
    let stats_snapshot = state.stats.lock().unwrap().clone();
    let cfg = state.config.lock().unwrap().clone();

    // 1. Roll rarity
    let rarity = gacha::roll_rarity(stats_snapshot.total, &cfg.rarity_thresholds);

    // 2. Select character via LLM
    let recent = storage::recent_pull_names(10);
    let selection = llm::select_character(&stats_snapshot, &rarity, &recent, &cfg.llm)?;

    // 3. Determine color mode
    let color_mode = if rand::random::<f64>() < cfg.ascii.color_probability {
        "color"
    } else {
        "mono"
    };

    // 4. Build stats seed for deterministic image selection
    let stats_seed = stats_snapshot.total;

    // 5. Fetch and process image
    let pipeline = image_pipeline::fetch_frames(
        &selection.search_query,
        &selection.character,
        cfg.ascii.columns,
        cfg.giphy_api_key.as_deref(),
        stats_seed,
    )?;

    // 6. Save pull
    let meta = storage::PullMeta {
        date: stats_snapshot.date.clone(),
        character: selection.character,
        scene: selection.scene,
        rarity: rarity.label().to_string(),
        flavor_text: selection.flavor_text,
        source: pipeline.source.clone(),
        color_mode: color_mode.to_string(),
        frame_count: pipeline.frames.len(),
    };

    storage::save_pull(&meta, &pipeline)?;

    Ok(meta)
}

fn main() {
    eprintln!("[dagashi] Starting...");
    let cfg = config::load_config();
    eprintln!("[dagashi] Config loaded");
    let shared_stats = stats::new_shared();
    eprintln!("[dagashi] Stats initialized");

    // Keystroke capture requires macOS Accessibility permission.
    // rdev::listen crashes the process if permission is not granted.
    // Skip on startup — user enables via Settings after granting permission.
    // TODO: add permission check before starting capture
    eprintln!("[dagashi] Keystroke capture disabled until Accessibility permission is granted.");
    eprintln!("[dagashi] Grant permission in System Settings > Privacy & Security > Accessibility,");
    eprintln!("[dagashi] then enable in Dagashi Settings.");

    // Periodic stats save (every 5 minutes)
    let stats_for_save = shared_stats.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(300));
        let s = stats_for_save.lock().unwrap();
        stats::save(&s);
    });

    eprintln!("[dagashi] Starting Tauri...");
    tauri::Builder::default()
        .manage(AppState {
            stats: shared_stats,
            config: Mutex::new(cfg),
        })
        .invoke_handler(tauri::generate_handler![
            get_stats,
            get_config,
            save_config_cmd,
            toggle_deaf_mode,
            get_collection,
            load_pull_frames,
            do_pull,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
