#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod anime_db;
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

    // 2. Pick anime from the database based on rarity
    let stats_seed = stats_snapshot.total;
    let anime = anime_db::pick_anime(&db, &rarity, stats_seed)
        .ok_or("no anime found for this rarity tier")?;
    eprintln!("[dagashi] Picked anime: {} (rank {})", anime.title, anime.popularity_rank);

    // 3. Select character via LLM
    let recent = storage::recent_pull_names(10);
    let selection = llm::select_character(
        &stats_snapshot, &rarity, &anime.title, &recent, &cfg.llm
    )?;
    eprintln!("[dagashi] LLM picked: {} - {}", selection.character, selection.scene);

    // 4. Determine color mode
    let color_mode = if rand::random::<f64>() < cfg.ascii.color_probability {
        "color"
    } else {
        "mono"
    };

    // 5. Fetch and process image
    let pipeline = image_pipeline::fetch_frames(
        &selection.search_query,
        &selection.character,
        cfg.ascii.columns,
        cfg.giphy_api_key.as_deref(),
        stats_seed,
    )?;
    eprintln!("[dagashi] Got {} frames from {}", pipeline.frames.len(), pipeline.source);

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
        anime_title: anime.title.clone(),
        anime_rank: anime.popularity_rank,
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
            do_pull,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
