use std::path::Path;

use rusqlite::Connection;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::{
    models::{attachment_model::Attachment, error_model::ChatError},
    services::attachment_service,
    state::ChatSession,
};

#[tauri::command]
pub async fn attach_file(
    app: AppHandle,
    path: String,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Attachment, ChatError> {
    let app_data_dir = app.path().app_data_dir()?;

    attachment_service::attach(&app_data_dir, Path::new(&path), &chat_session, &pool).await
}

#[tauri::command]
pub async fn remove_pending_attachment(
    path: String,
    chat_session: State<'_, Mutex<ChatSession>>,
) -> Result<(), ChatError> {
    attachment_service::detach(&chat_session, &path).await
}
