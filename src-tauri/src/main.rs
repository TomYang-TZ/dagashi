#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod anime_db;
mod config;
mod gacha;
mod image_pipeline;
mod jikan;
mod klipy;
mod tenor;
mod ipfs;
mod keylogger;
mod llm;
mod stats;
mod storage;

use chrono::Timelike;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    config: Mutex<config::Config>,
    anime_db: std::sync::Arc<Mutex<anime_db::AnimeDb>>,
}

#[tauri::command]
fn get_stats() -> stats::DailyStats {
    // Read stats from disk (written by dagashi-daemon)
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = config::data_dir().join("stats").join(format!("{today}.json"));
    if path.exists() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(s) = serde_json::from_str::<stats::DailyStats>(&data) {
                return s;
            }
        }
    }
    stats::DailyStats::default()
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

    // Write/remove file flag so the daemon sees it too
    let flag_path = config::data_dir().join("deaf");
    if deaf {
        std::fs::write(&flag_path, "1").ok();
    } else {
        std::fs::remove_file(&flag_path).ok();
    }

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
fn get_anime_db_status(state: State<AppState>) -> serde_json::Value {
    let db = state.anime_db.lock().unwrap();
    serde_json::json!({
        "count": db.anime.len(),
        "fetched_at": db.fetched_at,
        "anime": db.anime.iter().map(|a| {
            serde_json::json!({
                "title": a.title,
                "rank": a.popularity_rank,
                "members": a.members,
                "score": a.score,
                "rarity": crate::anime_db::rank_to_rarity(a.popularity_rank).label(),
            })
        }).collect::<Vec<_>>(),
    })
}

#[tauri::command]
fn load_pull_frames(date: String) -> Result<image_pipeline::PipelineResult, String> {
    storage::load_pull_frames(&date)
}

#[tauri::command]
fn load_pull_meta(date: String) -> Result<storage::PullMeta, String> {
    storage::load_pull_meta(&date)
}

#[tauri::command]
fn get_pull_cid(date: String) -> Option<String> {
    ipfs::load_pull_cid(&date)
}

#[tauri::command]
async fn pin_pull(state: State<'_, AppState>, date: String) -> Result<String, String> {
    let cfg = state.config.lock().unwrap().clone();
    let jwt = cfg.pinata_jwt
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or("no Pinata JWT configured — add one in Settings")?;

    // Check if already pinned
    if let Some(cid) = ipfs::load_pull_cid(&date) {
        return Ok(cid);
    }

    let jwt = jwt.to_string();
    tokio::task::spawn_blocking(move || {
        let meta = storage::load_pull_meta(&date)?;
        let pipeline = storage::load_pull_frames(&date)?;
        let receipt = ipfs::build_receipt(&meta, &pipeline)?;
        let cid = ipfs::pin_receipt(&receipt, &jwt)?;

        // Save locally
        ipfs::save_receipt(&date, &receipt, &cid);

        // Update meta with CID
        let mut meta = meta;
        meta.ipfs_cid = Some(cid.clone());
        let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
        std::fs::write(
            storage::pulls_dir().join(&date).join("meta.json"),
            meta_json,
        ).map_err(|e| e.to_string())?;

        Ok(cid)
    })
    .await
    .map_err(|e| format!("pin task failed: {e}"))?
}

#[tauri::command]
async fn do_pull(state: State<'_, AppState>) -> Result<storage::PullMeta, String> {
    let stats_snapshot = get_stats();
    let cfg = state.config.lock().unwrap().clone();
    let db = state.anime_db.lock().unwrap().clone();

    // Run the heavy work (LLM call + image download) on a background thread
    // so the UI stays responsive
    tokio::task::spawn_blocking(move || {
        do_pull_inner(stats_snapshot, cfg, db)
    })
    .await
    .map_err(|e| format!("pull task failed: {e}"))?
}

fn do_pull_inner(
    stats_snapshot: stats::DailyStats,
    cfg: config::Config,
    db: anime_db::AnimeDb,
) -> Result<storage::PullMeta, String> {

    // 1. Roll rarity
    let rarity = gacha::roll_rarity(stats_snapshot.total, &cfg.rarity_thresholds);
    eprintln!("[dagashi] Rolled rarity: {}", rarity.label());

    let used_urls: std::collections::HashSet<String> = storage::load_collection()
        .pulls
        .iter()
        .filter_map(|p| p.source_url.clone())
        .collect();
    let recent = storage::recent_pull_names(10);

    // 2. Try up to 3 anime, 3 characters each
    let stats_seed = stats_snapshot.total;
    let mut last_error = String::from("no pulls succeeded");
    let mut selection = None;
    let mut pipeline = None;
    let mut chosen_anime = None;

    'outer: for anime_try in 0..3u64 {
        let anime = match anime_db::pick_anime(&db, &rarity, stats_seed.wrapping_add(anime_try * 7)) {
            Some(a) => a,
            None => continue,
        };
        eprintln!("[dagashi] Try anime #{}: {} (rank {})", anime_try + 1, anime.title, anime.popularity_rank);

        for char_try in 0..3u64 {
            // Vary the LLM seed by adding char_try to stats
            let mut tweaked_stats = stats_snapshot.clone();
            tweaked_stats.total = tweaked_stats.total.wrapping_add(char_try * 13);

            let sel = match llm::select_character(
                &tweaked_stats, &rarity, &anime.title, &recent, &cfg.llm
            ) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[dagashi] LLM failed (try {}): {}", char_try + 1, e);
                    last_error = e;
                    continue;
                }
            };
            eprintln!("[dagashi] Try character #{}: {} - {}", char_try + 1, sel.character, sel.scene);

            match image_pipeline::fetch_frames(
                &sel.search_query,
                &sel.character,
                &anime.title,
                anime.mal_id,
                cfg.ascii.columns,
                &used_urls,
                &cfg.image_source,
                cfg.klipy_api_key.as_deref(),
            ) {
                Ok(p) if p.frames.len() > 1 => {
                    eprintln!("[dagashi] Got {} frames from {}", p.frames.len(), p.source);
                    selection = Some(sel);
                    pipeline = Some(p);
                    chosen_anime = Some(anime.clone());
                    break 'outer;
                }
                Ok(p) => {
                    eprintln!("[dagashi] Only {} frame(s), trying next character", p.frames.len());
                    // Keep as fallback if nothing better found
                    if selection.is_none() {
                        selection = Some(sel);
                        pipeline = Some(p);
                        chosen_anime = Some(anime.clone());
                    }
                    last_error = "only static image found".to_string();
                }
                Err(e) => {
                    eprintln!("[dagashi] Image fetch failed: {}", e);
                    last_error = e;
                }
            }
        }
    }

    let selection = selection.ok_or(last_error.clone())?;
    let pipeline = pipeline.ok_or(last_error)?;
    let anime = chosen_anime.unwrap();

    // Roll color mode
    let color_mode = if gacha::roll_color(&stats_snapshot) {
        "color"
    } else {
        "mono"
    };
    eprintln!("[dagashi] Final: {} from {} ({} frames)", selection.character, anime.title, pipeline.frames.len());

    // 6. Save pull — keyed by date + hour + minute
    let now = chrono::Local::now();
    let pull_key = format!(
        "{}-{:02}{:02}",
        stats_snapshot.date,
        now.hour(),
        now.minute()
    );
    let mut meta = storage::PullMeta {
        date: pull_key,
        character: selection.character,
        scene: selection.scene,
        rarity: rarity.label().to_string(),
        flavor_text: selection.flavor_text,
        source: pipeline.source.clone(),
        color_mode: color_mode.to_string(),
        frame_count: pipeline.frames.len(),
        anime_title: anime.title.clone(),
        anime_rank: anime.popularity_rank,
        source_url: Some(pipeline.source_url.clone()).filter(|s| !s.is_empty()),
        search_query: Some(pipeline.matched_query.clone()).filter(|s| !s.is_empty()),
        ipfs_cid: None,
    };

    storage::save_pull(&meta, &pipeline)?;

    Ok(meta)
}

fn launch_daemon_if_needed() {
    use std::process::Command;

    // Check if daemon is already running
    let check = Command::new("pgrep")
        .args(["-f", "dagashi-daemon"])
        .output();

    if let Ok(output) = check {
        if output.status.success() {
            eprintln!("[dagashi] Daemon already running");
            return;
        }
    }

    // Look for the daemon binary in known locations
    let home = dirs::home_dir().expect("no home dir");
    let daemon_paths = [
        home.join("dagashi/daemon/target/release/dagashi-daemon"),
        home.join(".dagashi/bin/dagashi-daemon"),
        std::path::PathBuf::from("/usr/local/bin/dagashi-daemon"),
    ];

    for path in &daemon_paths {
        if path.exists() {
            eprintln!("[dagashi] Launching daemon from {:?}", path);
            // Launch daemon in background via Terminal so it inherits Input Monitoring
            let script = format!(
                "tell application \"Terminal\" to do script \"{}\" & \"\"",
                path.display()
            );
            let _ = Command::new("osascript")
                .args(["-e", &script])
                .spawn();
            return;
        }
    }

    eprintln!("[dagashi] Daemon binary not found. Run: cd ~/dagashi/daemon && cargo build --release");
}

fn main() {
    // Catch panics so the app doesn't silently die
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[dagashi] PANIC: {info}");
    }));
    eprintln!("[dagashi] Starting...");
    let cfg = config::load_config();
    eprintln!("[dagashi] Config loaded");

    // Auto-launch the daemon if it's not already running
    launch_daemon_if_needed();

    // Start with empty anime DB — load in background so app opens instantly
    let shared_anime_db = std::sync::Arc::new(Mutex::new(
        anime_db::AnimeDb { anime: vec![], fetched_at: String::new() }
    ));

    // Try loading from cache first (instant), then refresh from API in background
    {
        let cache_path = config::data_dir().join("anime_db.json");
        if cache_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&cache_path) {
                if let Ok(db) = serde_json::from_str::<anime_db::AnimeDb>(&data) {
                    eprintln!("[dagashi] Loaded {} anime from cache", db.anime.len());
                    *shared_anime_db.lock().unwrap() = db;
                }
            }
        }
    }

    // Background fetch from Jikan API (refreshes cache if stale)
    let db_for_fetch = shared_anime_db.clone();
    std::thread::spawn(move || {
        match anime_db::load_or_fetch() {
            Ok(db) => {
                eprintln!("[dagashi] Anime DB ready: {} entries", db.anime.len());
                *db_for_fetch.lock().unwrap() = db;
            }
            Err(e) => eprintln!("[dagashi] Anime DB fetch failed: {e}"),
        }
    });

    eprintln!("[dagashi] Starting Tauri...");
    tauri::Builder::default()
        .manage(AppState {
            config: Mutex::new(cfg),
            anime_db: shared_anime_db,
        })
        .invoke_handler(tauri::generate_handler![
            get_stats,
            get_config,
            save_config_cmd,
            toggle_deaf_mode,
            get_collection,
            get_anime_db_status,
            load_pull_frames,
            load_pull_meta,
            get_pull_cid,
            pin_pull,
            do_pull,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
