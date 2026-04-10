use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyStats {
    pub date: String,
    pub total: u64,
    pub chars: HashMap<String, u64>,
    pub categories: CategoryCounts,
    pub backspace_count: u64,
    pub shift_count: u64,
    pub capslock_count: u64,
    pub hourly_volume: Vec<u64>,
    pub regions: RegionCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryCounts {
    pub letter: u64,
    pub number: u64,
    pub symbol: u64,
    pub modifier: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegionCounts {
    pub left_hand: u64,
    pub right_hand: u64,
    pub home_row: u64,
}

pub type SharedStats = Arc<Mutex<DailyStats>>;

pub fn new_shared() -> SharedStats {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let stats = load_or_create(&today);
    Arc::new(Mutex::new(stats))
}

pub fn record_key(stats: &SharedStats, key_name: &str) {
    let mut s = stats.lock().unwrap();
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Roll over to new day
    if s.date != today {
        save(&s);
        *s = new_day(&today);
    }

    s.total += 1;
    *s.chars.entry(key_name.to_string()).or_insert(0) += 1;

    let hour = Local::now().hour() as usize;
    if hour < s.hourly_volume.len() {
        s.hourly_volume[hour] += 1;
    }

    // Categorize
    match key_name {
        "backspace" => s.backspace_count += 1,
        "shift" => { s.shift_count += 1; s.categories.modifier += 1; }
        "capslock" => { s.capslock_count += 1; s.categories.modifier += 1; }
        "cmd" | "ctrl" | "option" | "tab" | "escape" | "return" => s.categories.modifier += 1,
        name if name.len() == 1 => {
            let c = name.chars().next().unwrap();
            if c.is_alphabetic() { s.categories.letter += 1; }
            else if c.is_numeric() { s.categories.number += 1; }
            else { s.categories.symbol += 1; }

            // Keyboard region (QWERTY)
            let left = "qwertasdfgzxcvb`12345";
            let home = "asdfghjkl;'";
            if left.contains(c.to_ascii_lowercase()) {
                s.regions.left_hand += 1;
            } else {
                s.regions.right_hand += 1;
            }
            if home.contains(c.to_ascii_lowercase()) {
                s.regions.home_row += 1;
            }
        }
        _ => s.categories.symbol += 1,
    }
}

pub fn save(stats: &DailyStats) {
    if stats.date.is_empty() {
        return;
    }
    let dir = config::data_dir().join("stats");
    fs::create_dir_all(&dir).ok();
    let path = dir.join(format!("{}.json", stats.date));
    if let Ok(json) = serde_json::to_string_pretty(stats) {
        fs::write(path, json).ok();
    }
}

fn new_day(date: &str) -> DailyStats {
    DailyStats {
        date: date.to_string(),
        hourly_volume: vec![0; 24],
        ..Default::default()
    }
}

fn load_or_create(date: &str) -> DailyStats {
    let path = config::data_dir().join("stats").join(format!("{date}.json"));
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(stats) = serde_json::from_str(&data) {
                return stats;
            }
        }
    }
    new_day(date)
}
