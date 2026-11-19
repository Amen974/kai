use rusqlite::Connection;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::{
    commands::chat_command::emit_chat_event,
    database::repository,
    models::{ai_model::AiModel, chat_event::ChatEvent, error_model::ChatError},
    ollama::DEFAULT_BASE_URL,
    services::model_service,
    state::ChatSession,
};

#[tauri::command]
pub async fn sync_models(
    app: AppHandle,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    model_service::sync_models(&pool, DEFAULT_BASE_URL).await?;
    emit_chat_event(&app, ChatEvent::ModelsChanged);
    Ok(())
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    name: String,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    model_service::download_model(&chat_session, &pool, DEFAULT_BASE_URL, &name, |event| {
        emit_chat_event(&app, event)
    })
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_model_download(
    chat_session: State<'_, Mutex<ChatSession>>,
) -> Result<(), ChatError> {
    model_service::cancel_download(&chat_session).await;
    Ok(())
}

#[tauri::command]
pub async fn delete_model(
    app: AppHandle,
    name: String,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    model_service::delete_model(&pool, DEFAULT_BASE_URL, &name).await?;
    emit_chat_event(&app, ChatEvent::ModelsChanged);
    Ok(())
}

#[tauri::command]
pub fn get_models(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Vec<AiModel>, ChatError> {
    repository::get_models(&pool)
}

#[tauri::command]
pub fn get_active_model(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Option<AiModel>, ChatError> {
    model_service::active_model(&pool)
}

#[tauri::command]
pub fn set_active_model(
    app: AppHandle,
    name: String,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    model_service::set_active_model(&pool, &name)?;
    emit_chat_event(&app, ChatEvent::ModelsChanged);
    Ok(())
}
