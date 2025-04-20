use key_block::{start, stop};
use tauri::AppHandle;

pub mod key_block;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_command() {
    start();
}

#[tauri::command]
fn stop_command(app: AppHandle) {
    stop(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_command, stop_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
