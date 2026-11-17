use rusqlite::Connection;
use tauri::State;

use crate::{
    models::{ai_model::AiModel, error_model::ChatError},
    services::model_service,
};

#[tauri::command]
pub async fn sync_models(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Vec<AiModel>, ChatError> {
    model_service::sync_models(&pool).await
}

#[tauri::command]
pub fn get_models(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Vec<AiModel>, ChatError> {
    model_service::list_models(&pool)
}

#[tauri::command]
pub fn get_active_model(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Option<AiModel>, ChatError> {
    model_service::active_model(&pool)
}

#[tauri::command]
pub fn set_active_model(
    name: String,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Vec<AiModel>, ChatError> {
    model_service::set_active_model(&pool, &name)
}
