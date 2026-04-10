use std::sync::atomic::{AtomicBool, Ordering};

use crate::stats::{self, SharedStats};

static DEAF_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_deaf_mode(deaf: bool) {
    DEAF_MODE.store(deaf, Ordering::Relaxed);
}

pub fn is_deaf() -> bool {
    DEAF_MODE.load(Ordering::Relaxed)
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

/// Start keystroke capture using CGEventTap with a lock-free channel.
/// The tap callback sends keycodes instantly through mpsc, a worker thread
/// does the actual stats recording.
pub fn start_capture(stats: SharedStats) {
    use std::sync::atomic::AtomicU64;
    use std::sync::mpsc;

    static KEY_COUNT: AtomicU64 = AtomicU64::new(0);
    static SENDER: std::sync::OnceLock<mpsc::Sender<u16>> = std::sync::OnceLock::new();

    let (tx, rx) = mpsc::channel::<u16>();
    SENDER.set(tx).ok();

    // Worker thread: receives keycodes and updates stats
    std::thread::spawn(move || {
        while let Ok(keycode) = rx.recv() {
            let name = keycode_to_name(keycode);
            let count = KEY_COUNT.fetch_add(1, Ordering::Relaxed);
            if count < 10 || count % 100 == 0 {
                eprintln!("[dagashi] Key #{}: code={} -> {}", count + 1, keycode, name);
            }
            stats::record_key(&stats, name);
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

    extern "C" fn tap_callback(
        _proxy: *mut std::ffi::c_void, event_type: u32, event: *mut std::ffi::c_void, _user_info: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void {
        if event.is_null() || DEAF_MODE.load(Ordering::Relaxed) {
            return event;
        }
        if event_type == 10 || event_type == 12 {
            unsafe {
                let keycode = CGEventGetIntegerValueField(event, 9) as u16;
                if let Some(tx) = SENDER.get() {
                    let _ = tx.send(keycode);
                }
            }
        }
        event
    }

    unsafe {
        // kCGSessionEventTap=1 (session level, not HID level)
        // kCGHeadInsertEventTap=0
        // kCGEventTapOptionListenOnly=1
        // Listen for keyDown (1<<10) and flagsChanged (1<<12)
        let event_mask: u64 = (1 << 10) | (1 << 12);

        // Try session-level tap first (works better for .app bundles)
        let tap = CGEventTapCreate(1, 0, 1, event_mask, tap_callback, std::ptr::null_mut());
        if tap.is_null() {
            eprintln!("[dagashi] Session tap failed, trying HID tap...");
            let tap = CGEventTapCreate(0, 0, 1, event_mask, tap_callback, std::ptr::null_mut());
            if tap.is_null() {
                eprintln!("[dagashi] HID tap also failed — no permission?");
                return;
            }
            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            let run_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);
            eprintln!("[dagashi] HID CGEventTap listening...");
            CFRunLoopRun();
        } else {
            let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
            let run_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap, true);
            eprintln!("[dagashi] Session CGEventTap listening...");
            CFRunLoopRun();
        }
    }
}
