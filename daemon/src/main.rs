use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static TOTAL_KEYS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct DailyStats {
    date: String,
    total: u64,
    chars: HashMap<String, u64>,
    categories: CategoryCounts,
    backspace_count: u64,
    shift_count: u64,
    capslock_count: u64,
    hourly_volume: Vec<u64>,
    regions: RegionCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CategoryCounts {
    letter: u64,
    number: u64,
    symbol: u64,
    modifier: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RegionCounts {
    left_hand: u64,
    right_hand: u64,
    home_row: u64,
}

fn data_dir() -> std::path::PathBuf {
    dirs::home_dir().expect("no home dir").join(".dagashi")
}

fn stats_path(date: &str) -> std::path::PathBuf {
    data_dir().join("stats").join(format!("{date}.json"))
}

fn load_or_create(date: &str) -> DailyStats {
    let path = stats_path(date);
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(stats) = serde_json::from_str(&data) {
                return stats;
            }
        }
    }
    DailyStats {
        date: date.to_string(),
        hourly_volume: vec![0; 24],
        ..Default::default()
    }
}

fn save_stats(stats: &DailyStats) {
    let dir = data_dir().join("stats");
    fs::create_dir_all(&dir).ok();
    if let Ok(json) = serde_json::to_string_pretty(stats) {
        fs::write(stats_path(&stats.date), json).ok();
    }
}

fn keycode_to_name(keycode: u16) -> &'static str {
    match keycode {
        0 => "a", 1 => "s", 2 => "d", 3 => "f",
        4 => "h", 5 => "g", 6 => "z", 7 => "x",
        8 => "c", 9 => "v", 11 => "b", 12 => "q",
        13 => "w", 14 => "e", 15 => "r", 16 => "y",
        17 => "t", 18 => "1", 19 => "2", 20 => "3",
        21 => "4", 22 => "6", 23 => "5", 24 => "=",
        25 => "9", 26 => "7", 27 => "-", 28 => "8",
        29 => "0", 30 => "]", 31 => "o", 32 => "u",
        33 => "[", 34 => "i", 35 => "p", 36 => "⏎",
        37 => "l", 38 => "j", 39 => "'", 40 => "k",
        41 => ";", 42 => "\\", 43 => ",", 44 => "/",
        45 => "n", 46 => "m", 47 => ".",
        48 => "⇥", 49 => "␣", 50 => "`",
        51 => "⌫", 53 => "⎋",
        55 => "⌘", 56 => "⇧", 57 => "⇪",
        58 => "⌥", 59 => "⌃",
        122 => "F1", 120 => "F2", 99 => "F3",
        118 => "F4", 96 => "F5", 97 => "F6",
        98 => "F7", 100 => "F8", 101 => "F9",
        109 => "F10", 103 => "F11", 111 => "F12",
        123 => "←", 124 => "→", 125 => "↓", 126 => "↑",
        _ => "?",
    }
}

fn record_key(stats: &Arc<Mutex<DailyStats>>, key_name: &str) {
    let mut s = stats.lock().unwrap();
    let today = Local::now().format("%Y-%m-%d").to_string();

    if s.date != today {
        save_stats(&s);
        *s = DailyStats {
            date: today,
            hourly_volume: vec![0; 24],
            ..Default::default()
        };
    }

    s.total += 1;
    *s.chars.entry(key_name.to_string()).or_insert(0) += 1;

    let hour = Local::now().hour() as usize;
    if hour < s.hourly_volume.len() {
        s.hourly_volume[hour] += 1;
    }

    match key_name {
        "⌫" => s.backspace_count += 1,
        "⇧" => { s.shift_count += 1; s.categories.modifier += 1; }
        "⇪" => { s.capslock_count += 1; s.categories.modifier += 1; }
        "⌘" | "⌃" | "⌥" | "⇥" | "⎋" | "⏎" => s.categories.modifier += 1,
        name if name.len() == 1 => {
            let c = name.chars().next().unwrap();
            if c.is_alphabetic() { s.categories.letter += 1; }
            else if c.is_numeric() { s.categories.number += 1; }
            else { s.categories.symbol += 1; }

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

fn compile_helper() -> std::path::PathBuf {
    let bin_path = data_dir().join("bin").join("dagashi-keytap");

    if bin_path.exists() {
        return bin_path;
    }

    let swift_code = r#"
import Cocoa

let eventMask = (1 << CGEventType.keyDown.rawValue)

guard let tap = CGEvent.tapCreate(
    tap: .cgSessionEventTap,
    place: .headInsertEventTap,
    options: .listenOnly,
    eventsOfInterest: CGEventMask(eventMask),
    callback: { _, type, event, _ in
        if type == .keyDown {
            let keycode = event.getIntegerValueField(.keyboardEventKeycode)
            print(keycode)
            fflush(stdout)
        }
        return Unmanaged.passRetained(event)
    },
    userInfo: nil
) else {
    fputs("FAILED\n", stderr)
    exit(1)
}

let source = CFMachPortCreateRunLoopSource(nil, tap, 0)
CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
CGEvent.tapEnable(tap: tap, enable: true)
fputs("LISTENING\n", stderr)
CFRunLoopRun()
"#;

    let tmp_path = std::env::temp_dir().join("dagashi_keytap.swift");
    fs::write(&tmp_path, swift_code).expect("failed to write swift code");
    fs::create_dir_all(bin_path.parent().unwrap()).ok();

    eprintln!("[dagashi-daemon] Compiling key capture helper...");
    let output = Command::new("swiftc")
        .args(["-O", "-o"])
        .arg(&bin_path)
        .arg(&tmp_path)
        .output()
        .expect("swiftc not found");

    if !output.status.success() {
        panic!("Failed to compile helper: {}", String::from_utf8_lossy(&output.stderr));
    }

    eprintln!("[dagashi-daemon] Helper compiled at {:?}", bin_path);
    bin_path
}

fn main() {
    eprintln!("╔═══════════════════════════════════╗");
    eprintln!("║  DAGASHI DAEMON — key listener    ║");
    eprintln!("╚═══════════════════════════════════╝");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let stats = Arc::new(Mutex::new(load_or_create(&today)));
    {
        let s = stats.lock().unwrap();
        eprintln!("[dagashi-daemon] Loaded stats for {}: {} keys so far", s.date, s.total);
    }

    // Periodic save every 10 seconds
    let stats_for_save = stats.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(10));
        let s = stats_for_save.lock().unwrap();
        save_stats(&s);
    });

    // Compile/find the Swift helper
    let helper_path = compile_helper();

    // Launch the helper
    eprintln!("[dagashi-daemon] Launching key capture...");
    let mut child = Command::new(&helper_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to launch key capture helper");

    // Read helper stderr
    let stderr = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            eprintln!("[dagashi-keytap] {}", line);
        }
    });

    // Read keycodes from helper stdout
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    for line in reader.lines().map_while(Result::ok) {
        if let Ok(keycode) = line.trim().parse::<u16>() {
            let name = keycode_to_name(keycode);
            let count = TOTAL_KEYS.fetch_add(1, Ordering::Relaxed) + 1;
            if count <= 10 || count % 100 == 0 {
                eprintln!("[dagashi-daemon] Key #{}: {} ({})", count, name, keycode);
            }
            record_key(&stats, name);

            // Save every 50 keys
            if count % 50 == 0 {
                let s = stats.lock().unwrap();
                save_stats(&s);
            }
        }
    }

    // Save on exit
    let s = stats.lock().unwrap();
    save_stats(&s);
    eprintln!("[dagashi-daemon] Saved {} keys. Goodbye.", s.total);
}
