use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::stats::{self, SharedStats};

/// Check if we have Accessibility permission on macOS.
/// If not granted, shows the macOS permission dialog prompting the user.
#[cfg(target_os = "macos")]
pub fn check_and_prompt_accessibility() -> bool {
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(alloc: *const std::ffi::c_void, c_str: *const i8, encoding: u32) -> *const std::ffi::c_void;
        fn CFDictionaryCreate(
            alloc: *const std::ffi::c_void,
            keys: *const *const std::ffi::c_void,
            values: *const *const std::ffi::c_void,
            count: isize,
            key_callbacks: *const std::ffi::c_void,
            value_callbacks: *const std::ffi::c_void,
        ) -> *const std::ffi::c_void;
        static kCFBooleanTrue: *const std::ffi::c_void;
        static kCFTypeDictionaryKeyCallBacks: std::ffi::c_void;
        static kCFTypeDictionaryValueCallBacks: std::ffi::c_void;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
    }

    unsafe {
        // Create the key string "AXTrustedCheckOptionPrompt"
        let key_str = CFStringCreateWithCString(
            std::ptr::null(),
            b"AXTrustedCheckOptionPrompt\0".as_ptr() as *const i8,
            0x08000100, // kCFStringEncodingUTF8
        );

        // Create dict { AXTrustedCheckOptionPrompt: true } to show the prompt
        let keys = [key_str];
        let values = [kCFBooleanTrue];
        let options = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const _ as *const std::ffi::c_void,
            &kCFTypeDictionaryValueCallBacks as *const _ as *const std::ffi::c_void,
        );

        AXIsProcessTrustedWithOptions(options)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn check_and_prompt_accessibility() -> bool { true }

static DEAF_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_deaf_mode(deaf: bool) {
    DEAF_MODE.store(deaf, Ordering::Relaxed);
}

pub fn is_deaf() -> bool {
    DEAF_MODE.load(Ordering::Relaxed)
}

fn key_to_name(key: &Key) -> Option<&'static str> {
    match key {
        // Letters
        Key::KeyA => Some("a"), Key::KeyB => Some("b"), Key::KeyC => Some("c"),
        Key::KeyD => Some("d"), Key::KeyE => Some("e"), Key::KeyF => Some("f"),
        Key::KeyG => Some("g"), Key::KeyH => Some("h"), Key::KeyI => Some("i"),
        Key::KeyJ => Some("j"), Key::KeyK => Some("k"), Key::KeyL => Some("l"),
        Key::KeyM => Some("m"), Key::KeyN => Some("n"), Key::KeyO => Some("o"),
        Key::KeyP => Some("p"), Key::KeyQ => Some("q"), Key::KeyR => Some("r"),
        Key::KeyS => Some("s"), Key::KeyT => Some("t"), Key::KeyU => Some("u"),
        Key::KeyV => Some("v"), Key::KeyW => Some("w"), Key::KeyX => Some("x"),
        Key::KeyY => Some("y"), Key::KeyZ => Some("z"),
        // Numbers
        Key::Num0 => Some("0"), Key::Num1 => Some("1"), Key::Num2 => Some("2"),
        Key::Num3 => Some("3"), Key::Num4 => Some("4"), Key::Num5 => Some("5"),
        Key::Num6 => Some("6"), Key::Num7 => Some("7"), Key::Num8 => Some("8"),
        Key::Num9 => Some("9"),
        // Symbols
        Key::Comma => Some(","), Key::Dot => Some("."),
        Key::Slash => Some("/"), Key::SemiColon => Some(";"),
        Key::Quote => Some("'"), Key::LeftBracket => Some("["),
        Key::RightBracket => Some("]"), Key::BackSlash => Some("\\"),
        Key::Minus => Some("-"), Key::Equal => Some("="),
        Key::BackQuote => Some("`"),
        // Whitespace & editing
        Key::Space => Some("␣"), Key::Tab => Some("⇥"),
        Key::Backspace => Some("⌫"), Key::Delete => Some("⌦"),
        Key::Return => Some("⏎"),
        // Modifiers
        Key::Escape => Some("⎋"),
        Key::ShiftLeft | Key::ShiftRight => Some("⇧"),
        Key::CapsLock => Some("⇪"),
        Key::ControlLeft | Key::ControlRight => Some("⌃"),
        Key::Alt => Some("⌥"),
        Key::MetaLeft | Key::MetaRight => Some("⌘"),
        // Function keys
        Key::F1 => Some("F1"), Key::F2 => Some("F2"), Key::F3 => Some("F3"),
        Key::F4 => Some("F4"), Key::F5 => Some("F5"), Key::F6 => Some("F6"),
        Key::F7 => Some("F7"), Key::F8 => Some("F8"), Key::F9 => Some("F9"),
        Key::F10 => Some("F10"), Key::F11 => Some("F11"), Key::F12 => Some("F12"),
        // Arrow keys
        Key::UpArrow => Some("↑"), Key::DownArrow => Some("↓"),
        Key::LeftArrow => Some("←"), Key::RightArrow => Some("→"),
        // Other
        Key::Home => Some("⇱"), Key::End => Some("⇲"),
        Key::PageUp => Some("⇞"), Key::PageDown => Some("⇟"),
        _ => Some("?"),  // Record unknown keys too — every keystroke counts
    }
}

/// Start keystroke capture using raw macOS CGEventTap.
/// rdev::listen doesn't work when Tauri owns the main run loop,
/// so we create our own CGEventTap directly on this thread.
#[cfg(target_os = "macos")]
pub fn start_capture(stats: SharedStats) {
    use std::sync::atomic::AtomicU64;
    static KEY_COUNT: AtomicU64 = AtomicU64::new(0);

    // Store stats in a static so the C callback can access it
    use std::sync::OnceLock;
    static STATS: OnceLock<SharedStats> = OnceLock::new();
    STATS.set(stats).ok();

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
        if event.is_null() {
            return event;
        }
        if DEAF_MODE.load(Ordering::Relaxed) {
            return event;
        }
        // Only process kCGEventKeyDown (10) and kCGEventFlagsChanged (12)
        if event_type != 10 && event_type != 12 {
            return event;
        }
        unsafe {
            let keycode = CGEventGetIntegerValueField(event, 9) as u16;
            if let Some(name) = keycode_to_name(keycode) {
                let count = KEY_COUNT.fetch_add(1, Ordering::Relaxed);
                if count < 10 || count % 100 == 0 {
                    eprintln!("[dagashi] Key #{}: code={} -> {}", count + 1, keycode, name);
                }
                if let Some(stats) = STATS.get() {
                    crate::stats::record_key(stats, name);
                }
            }
        }
        event
    }

    unsafe {
        // kCGHIDEventTap=0, kCGHeadInsertEventTap=0, kCGEventTapOptionListenOnly=1
        // CGEventMaskBit(kCGEventKeyDown=10) = 1 << 10 = 1024
        // kCGHIDEventTap=0, kCGHeadInsertEventTap=0, kCGEventTapOptionListenOnly=1
        // CGEventMaskBit(kCGEventKeyDown=10) = 1 << 10
        // Also listen for kCGEventFlagsChanged=12 (modifier keys) = 1 << 12
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
        eprintln!("[dagashi] CGEventTap installed and enabled, listening for keys...");
        CFRunLoopRun(); // blocks forever
    }
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
        // Function keys
        122 => Some("F1"), 120 => Some("F2"), 99 => Some("F3"),
        118 => Some("F4"), 96 => Some("F5"), 97 => Some("F6"),
        98 => Some("F7"), 100 => Some("F8"), 101 => Some("F9"),
        109 => Some("F10"), 103 => Some("F11"), 111 => Some("F12"),
        // Arrows
        123 => Some("←"), 124 => Some("→"), 125 => Some("↓"), 126 => Some("↑"),
        _ => Some("?"),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn start_capture(stats: SharedStats) {
    let callback = move |event: Event| {
        if DEAF_MODE.load(Ordering::Relaxed) { return; }
        if let EventType::KeyPress(key) = event.event_type {
            if let Some(name) = key_to_name(&key) {
                stats::record_key(&stats, name);
            }
        }
    };
    if let Err(e) = listen(callback) {
        eprintln!("Failed to start key listener: {:?}", e);
    }
}
