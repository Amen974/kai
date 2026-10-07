use tauri::Manager;
use tokio::sync::Mutex;

use crate::commands::chat_command::{
    cancel_generation, delete_history, edit_message, get_history, get_session, load_chat,
    new_chat, resend_message, send_message,
};
use crate::state::ChatSession;

mod commands;
mod database;
mod models;
mod ollama;
mod services;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let pool = database::init::init(app.handle()).expect("Failed to initialize database");
            app.manage(std::sync::Mutex::new(pool));
            Ok(())
        })
        .manage(Mutex::new(ChatSession {
            history_id: None,
            messages: Vec::new(),
            active: None,
            next_generation_id: 0,
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            send_message,
            resend_message,
            edit_message,
            load_chat,
            new_chat,
            delete_history,
            cancel_generation,
            get_session,
            get_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}