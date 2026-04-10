use std::sync::atomic::{AtomicBool, Ordering};

use crate::stats::{self, SharedStats};

static DEAF_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_deaf_mode(deaf: bool) {
    DEAF_MODE.store(deaf, Ordering::Relaxed);
}

pub fn is_deaf() -> bool {
    DEAF_MODE.load(Ordering::Relaxed)
}

fn keycode_to_name(keycode: u16) -> Option<&'static str> {
    match keycode {
        0 => Some("a"), 1 => Some("s"), 2 => Some("d"), 3 => Some("f"),
        4 => Some("h"), 5 => Some("g"), 6 => Some("z"), 7 => Some("x"),
        8 => Some("c"), 9 => Some("v"), 11 => Some("b"), 12 => Some("q"),
        13 => Some("w"), 14 => Some("e"), 15 => Some("r"), 16 => Some("y"),
        17 => Some("t"), 18 => Some("1"), 19 => Some("2"), 20 => Some("3"),
        21 => Some("4"), 22 => Some("6"), 23 => Some("5"), 24 => Some("="),
        25 => Some("9"), 26 => Some("7"), 27 => Some("-"), 28 => Some("8"),
        29 => Some("0"), 30 => Some("]"), 31 => Some("o"), 32 => Some("u"),
        33 => Some("["), 34 => Some("i"), 35 => Some("p"), 36 => Some("⏎"),
        37 => Some("l"), 38 => Some("j"), 39 => Some("'"), 40 => Some("k"),
        41 => Some(";"), 42 => Some("\\"), 43 => Some(","), 44 => Some("/"),
        45 => Some("n"), 46 => Some("m"), 47 => Some("."),
        48 => Some("⇥"), 49 => Some("␣"), 50 => Some("`"),
        51 => Some("⌫"), 53 => Some("⎋"),
        55 => Some("⌘"), 56 => Some("⇧"), 57 => Some("⇪"),
        58 => Some("⌥"), 59 => Some("⌃"),
        122 => Some("F1"), 120 => Some("F2"), 99 => Some("F3"),
        118 => Some("F4"), 96 => Some("F5"), 97 => Some("F6"),
        98 => Some("F7"), 100 => Some("F8"), 101 => Some("F9"),
        109 => Some("F10"), 103 => Some("F11"), 111 => Some("F12"),
        123 => Some("←"), 124 => Some("→"), 125 => Some("↓"), 126 => Some("↑"),
        _ => Some("?"),
    }
}

/// Start keystroke capture using raw macOS CGEventTap.
/// Uses a lock-free channel to avoid blocking the callback
/// (macOS disables the tap if the callback is slow).
#[cfg(target_os = "macos")]
pub fn start_capture(stats: SharedStats) {
    use std::sync::atomic::AtomicU64;
    use std::sync::mpsc;

    static KEY_COUNT: AtomicU64 = AtomicU64::new(0);

    // Lock-free channel: callback sends keycodes, worker thread records stats
    static SENDER: std::sync::OnceLock<mpsc::Sender<u16>> = std::sync::OnceLock::new();

    let (tx, rx) = mpsc::channel::<u16>();
    SENDER.set(tx).ok();

    // Worker thread: receives keycodes and updates stats (can safely lock mutex)
    let stats_clone = stats;
    std::thread::spawn(move || {
        while let Ok(keycode) = rx.recv() {
            if let Some(name) = keycode_to_name(keycode) {
                let count = KEY_COUNT.fetch_add(1, Ordering::Relaxed);
                if count < 10 || count % 100 == 0 {
                    eprintln!("[dagashi] Key #{}: code={} -> {}", count + 1, keycode, name);
                }
                stats::record_key(&stats_clone, name);
            }
        }
    });

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventTapCreate(
            tap: u32, place: u32, options: u32, events_of_interest: u64,
            callback: extern "C" fn(*mut std::ffi::c_void, u32, *mut std::ffi::c_void, *mut std::ffi::c_void) -> *mut std::ffi::c_void,
            user_info: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        fn CGEventTapEnable(tap: *mut std::ffi::c_void, enable: bool);
        fn CFMachPortCreateRunLoopSource(alloc: *const std::ffi::c_void, port: *mut std::ffi::c_void, order: i64) -> *mut std::ffi::c_void;
        fn CFRunLoopAddSource(rl: *mut std::ffi::c_void, source: *mut std::ffi::c_void, mode: *const std::ffi::c_void);
        fn CFRunLoopGetCurrent() -> *mut std::ffi::c_void;
        fn CFRunLoopRun();
        fn CGEventGetIntegerValueField(event: *mut std::ffi::c_void, field: u32) -> i64;
        static kCFRunLoopCommonModes: *const std::ffi::c_void;
    }

    // CGEventTap callback — must return FAST, no mutex, no allocation
    extern "C" fn tap_callback(
        _proxy: *mut std::ffi::c_void, event_type: u32, event: *mut std::ffi::c_void, _user_info: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void {
        if event.is_null() || DEAF_MODE.load(Ordering::Relaxed) {
            return event;
        }
        // kCGEventKeyDown=10, kCGEventFlagsChanged=12
        if event_type == 10 || event_type == 12 {
            unsafe {
                let keycode = CGEventGetIntegerValueField(event, 9) as u16;
                // Send through channel — non-blocking, no mutex
                if let Some(tx) = SENDER.get() {
                    let _ = tx.send(keycode);
                }
            }
        }
        event
    }

    unsafe {
        // Listen for key down + modifier flag changes
        let event_mask: u64 = (1 << 10) | (1 << 12);
        let tap = CGEventTapCreate(0, 0, 1, event_mask, tap_callback, std::ptr::null_mut());
        if tap.is_null() {
            eprintln!("[dagashi] CGEventTapCreate failed — no Accessibility permission?");
            return;
        }
        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        eprintln!("[dagashi] CGEventTap installed, listening...");
        CFRunLoopRun();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn start_capture(_stats: SharedStats) {
    eprintln!("[dagashi] Keystroke capture not supported on this platform");
}
