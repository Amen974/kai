use tokio::sync::Mutex;
use crate::commands::chat::send_message;
use crate::state::MessageState;

mod commands;
mod models;
mod services;
mod ollama;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(MessageState {
            messages: Vec::new(),
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}