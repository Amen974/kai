use tauri::Manager;
use tokio::sync::Mutex;
use crate::commands::chat_command::{cancel_token, edit_message, get_history, get_messages, send_message};
use crate::state::{ChatHistory, CancelState};


mod commands;
mod models;
mod services;
mod ollama;
mod state;
mod database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let pool =  database::init::init(app.handle())
                .expect("Failed to initialize database");

            app.manage(std::sync::Mutex::new(pool));

            Ok(())
        })
        .manage(Mutex::new(ChatHistory {
            messages: Vec::new(),
        }))
        .manage(Mutex::new(CancelState {
            token: None,
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_message, cancel_token, edit_message, get_history, get_messages])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}