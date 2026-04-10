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
    stats: stats::SharedStats,
    config: Mutex<config::Config>,
    anime_db: std::sync::Arc<Mutex<anime_db::AnimeDb>>,
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
    let stats_snapshot = state.stats.lock().unwrap().clone();
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

fn main() {
    // Catch panics so the app doesn't silently die
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[dagashi] PANIC: {info}");
    }));
    eprintln!("[dagashi] Starting...");
    let cfg = config::load_config();
    eprintln!("[dagashi] Config loaded");
    let shared_stats = stats::new_shared();
    eprintln!("[dagashi] Stats initialized");

    // Keystroke capture — requires macOS Accessibility permission.
    // In release builds (.app), permission persists. In dev mode, the binary
    // path changes on recompile so permission is lost — use mock stats instead.
    #[cfg(not(debug_assertions))]
    if cfg.keystroke_capture.enabled {
        if keylogger::check_and_prompt_accessibility() {
            let stats_for_capture = shared_stats.clone();
            keylogger::set_deaf_mode(cfg.keystroke_capture.deaf_mode);
            std::thread::spawn(move || {
                eprintln!("[dagashi] Starting keystroke capture...");
                keylogger::start_capture(stats_for_capture);
            });
        } else {
            eprintln!("[dagashi] No Accessibility permission — keystroke capture disabled.");
            eprintln!("[dagashi] Grant permission: System Settings > Privacy & Security > Accessibility");
            eprintln!("[dagashi] Then restart Dagashi.");
        }
    }

    #[cfg(debug_assertions)]
    {
        eprintln!("[dagashi] DEV MODE: keystroke capture disabled, using mock stats");
        let mut s = shared_stats.lock().unwrap();
        if s.total == 0 {
            s.total = 8500;
            for (ch, count) in [("e",890),("t",720),("a",680),("o",590),("i",510),
                ("n",480),("s",440),("r",410),("h",320),("l",280),("d",250),
                ("c",220),("u",200),("m",180),("f",150),("p",130),("g",110),
                ("w",100),("y",90),("b",80),("v",60),("k",50),("j",30),("x",20)] {
                s.chars.insert(ch.to_string(), count as u64);
            }
            s.categories.letter = 7400;
            s.categories.number = 500;
            s.categories.symbol = 400;
            s.categories.modifier = 200;
            s.backspace_count = 320;
            s.shift_count = 180;
        }
    }

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
