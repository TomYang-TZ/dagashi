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

/// Start keystroke capture. Blocks the calling thread — run from a dedicated thread.
pub fn start_capture(stats: SharedStats) {
    let callback = move |event: Event| {
        if DEAF_MODE.load(Ordering::Relaxed) {
            return;
        }
        if let EventType::KeyPress(key) = event.event_type {
            if let Some(name) = key_to_name(&key) {
                stats::record_key(&stats, name);
            }
        }
    };

    if let Err(e) = listen(callback) {
        eprintln!("Failed to start key listener: {:?}", e);
        eprintln!("Grant Accessibility permission in System Settings > Privacy & Security > Accessibility");
    }
}
