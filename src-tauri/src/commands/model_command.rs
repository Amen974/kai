use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::{
    commands::chat_command::emit_chat_event,
    database::repository,
    models::{ai_model::AiModel, chat_event::ChatEvent, error_model::ChatError},
    services::model_service,
};

#[tauri::command]
pub async fn sync_models(
    app: AppHandle,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    model_service::sync_models(&pool).await?;
    emit_chat_event(&app, ChatEvent::ModelsChanged);
    Ok(())
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    name: String,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    model_service::download_model(&pool, &name, |event| emit_chat_event(&app, event)).await
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
