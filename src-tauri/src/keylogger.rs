// Keylogger is now a separate daemon process (dagashi-daemon).
// This module is kept for the deaf mode toggle which the UI uses.

use std::sync::atomic::{AtomicBool, Ordering};

static DEAF_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_deaf_mode(deaf: bool) {
    DEAF_MODE.store(deaf, Ordering::Relaxed);
}

pub fn is_deaf() -> bool {
    DEAF_MODE.load(Ordering::Relaxed)
}
