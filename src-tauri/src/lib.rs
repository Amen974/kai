use tokio::sync::Mutex;
use crate::commands::chat_command::{cancel_token, send_message};
use crate::state::{MessageArray, CancelToken};


mod commands;
mod models;
mod services;
mod ollama;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(MessageArray {
            message_array: Vec::new(),
        }))
        .manage(Mutex::new(CancelToken {
            token: None,
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_message, cancel_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}