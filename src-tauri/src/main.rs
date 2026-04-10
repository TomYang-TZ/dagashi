#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod giphy;
mod wiki;
mod image_pipeline;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
