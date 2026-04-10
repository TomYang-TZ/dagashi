use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::stats::{self, SharedStats};

static DEAF_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_deaf_mode(deaf: bool) {
    DEAF_MODE.store(deaf, Ordering::Relaxed);
}

pub fn is_deaf() -> bool {
    DEAF_MODE.load(Ordering::Relaxed)
}

fn key_to_name(key: &Key) -> Option<&'static str> {
    match key {
        Key::KeyA => Some("a"), Key::KeyB => Some("b"), Key::KeyC => Some("c"),
        Key::KeyD => Some("d"), Key::KeyE => Some("e"), Key::KeyF => Some("f"),
        Key::KeyG => Some("g"), Key::KeyH => Some("h"), Key::KeyI => Some("i"),
        Key::KeyJ => Some("j"), Key::KeyK => Some("k"), Key::KeyL => Some("l"),
        Key::KeyM => Some("m"), Key::KeyN => Some("n"), Key::KeyO => Some("o"),
        Key::KeyP => Some("p"), Key::KeyQ => Some("q"), Key::KeyR => Some("r"),
        Key::KeyS => Some("s"), Key::KeyT => Some("t"), Key::KeyU => Some("u"),
        Key::KeyV => Some("v"), Key::KeyW => Some("w"), Key::KeyX => Some("x"),
        Key::KeyY => Some("y"), Key::KeyZ => Some("z"),
        Key::Num0 => Some("0"), Key::Num1 => Some("1"), Key::Num2 => Some("2"),
        Key::Num3 => Some("3"), Key::Num4 => Some("4"), Key::Num5 => Some("5"),
        Key::Num6 => Some("6"), Key::Num7 => Some("7"), Key::Num8 => Some("8"),
        Key::Num9 => Some("9"),
        Key::Space => Some("space"), Key::Tab => Some("tab"),
        Key::Backspace => Some("backspace"), Key::Return => Some("return"),
        Key::Escape => Some("escape"),
        Key::ShiftLeft | Key::ShiftRight => Some("shift"),
        Key::CapsLock => Some("capslock"),
        Key::ControlLeft | Key::ControlRight => Some("ctrl"),
        Key::Alt => Some("option"),
        Key::MetaLeft | Key::MetaRight => Some("cmd"),
        Key::Comma => Some(","), Key::Dot => Some("."),
        Key::Slash => Some("/"), Key::SemiColon => Some(";"),
        Key::Quote => Some("'"), Key::LeftBracket => Some("["),
        Key::RightBracket => Some("]"), Key::BackSlash => Some("\\"),
        Key::Minus => Some("-"), Key::Equal => Some("="),
        Key::BackQuote => Some("`"),
        _ => None,
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
