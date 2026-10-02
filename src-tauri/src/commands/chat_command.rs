use rusqlite::Connection;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

use crate::{
    models::chat_model::GetHistory, services::chat_service::{self, StreamEvent, recend_handel}, state::{CancelState, ChatHistory},
};

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
    message_arr: State<'_, Mutex<ChatHistory>>,
    cancel_state: State<'_, Mutex<CancelState>>,
    id: Option<u32>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), String> {
    chat_service::send(message, message_arr, cancel_state, id, pool, |event| {
        match event {
            StreamEvent::Message(message) => {
                let _ = app.emit("update_message", message)
                    .map_err(|error| error.to_string());
            }

            StreamEvent::ThinkingChunk(chunk) => {
                let _ = app.emit("update_thinking_content", chunk)
                    .map_err(|error| error.to_string());
            }

            StreamEvent::MessageChunk(message) => {
                let _ = app.emit("update_message_content", message)
                    .map_err(|error| error.to_string());
            }

            StreamEvent::Done => {
                let _ = app.emit("emit_done", ())
                    .map_err(|error| error.to_string());
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn cancel_token(token: State<'_, Mutex<CancelState>>) -> Result<(), String> {
    chat_service::cancel_token(&*token).await
}

#[tauri::command]
pub async fn edit_message(
    app: AppHandle,
    message_arr: State<'_, Mutex<ChatHistory>>,
    message: String,
    index: usize,
) -> Result<(), String> {
    let updated_messages = chat_service::edit_message(&*message_arr, message, index).await?;
    app.emit("update_messages", updated_messages)
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_history(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Vec<GetHistory>, String> {
    chat_service::get_history(&*pool)
}

#[tauri::command]
pub async fn get_messages(
    pool: State<'_, std::sync::Mutex<Connection>>,
    app: AppHandle,
    message_arr: State<'_, Mutex<ChatHistory>>,
    id: i32,
) -> Result<(), String> {
    let fetched_messages = chat_service::get_messages(&*pool, &*message_arr, id).await?;

    app.emit("update_messages", fetched_messages)
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_history (id: u32, pool: State<'_, std::sync::Mutex<Connection>>,) -> Result<(), String> {
    let conn = pool.lock().map_err(|error| error.to_string())?;

    conn.execute("DELETE FROM history WHERE id = ?1", [id]).map_err(|error| error.to_string())?;

    return Ok(());
}

#[tauri::command]
pub async fn recend_message (
    app: AppHandle,
    message: String,
    message_arr: State<'_, Mutex<ChatHistory>>,
    cancel_state: State<'_, Mutex<CancelState>>,
    id: Option<u32>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), String> {
    let id = id.ok_or_else(|| "need id".to_string())?;
    recend_handel(id, &pool, &message_arr).await?;

    send_message(app, message, message_arr, cancel_state, Some(id), pool).await?;
    return Ok(());
}