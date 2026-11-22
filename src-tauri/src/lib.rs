use rusqlite::Connection;
use tauri::Manager;
use tokio::sync::Mutex;

use crate::commands::chat_command::{
    cancel_generation, delete_history, edit_message, emit_chat_event, get_history, get_session,
    get_variants, load_chat, new_chat, resend_message, search_history, send_message,
    switch_branch,
};
use crate::commands::attachment_command::{attach_file, remove_pending_attachment};
use crate::models::chat_event::ChatEvent;
use crate::commands::model_command::{
    cancel_model_download, delete_model, download_model, get_active_model, get_models,
    set_active_model, sync_models,
};
use crate::services::model_service;
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

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let pool = handle.state::<std::sync::Mutex<Connection>>();
                match model_service::sync_if_empty(&pool).await {
                    Ok(true) => emit_chat_event(&handle, ChatEvent::ModelsChanged),
                    Ok(false) => {}
                    Err(error) => eprintln!("Failed to sync Ollama models: {error}"),
                }
            });

            Ok(())
        })
        .manage(Mutex::new(ChatSession::default()))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            send_message,
            resend_message,
            edit_message,
            load_chat,
            switch_branch,
            new_chat,
            delete_history,
            cancel_generation,
            get_session,
            get_history,
            search_history,
            get_variants,
            sync_models,
            get_models,
            get_active_model,
            set_active_model,
            download_model,
            cancel_model_download,
            delete_model,
            attach_file,
            remove_pending_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}