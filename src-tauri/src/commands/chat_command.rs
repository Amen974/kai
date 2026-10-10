use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

use crate::{
    database::repository,
    models::{
        chat_event::ChatEvent,
        chat_model::{GetHistory, SessionSnapshot},
        error_model::ChatError,
    },
    services::chat_service,
    state::ChatSession,
};

fn emit_event<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    if let Err(error) = app.emit(event, payload) {
        eprintln!("Failed to emit Tauri event '{event}': {error}");
    }
}

fn make_callback(app: AppHandle) -> impl Fn(ChatEvent) {
    move |event| match event {
        ChatEvent::Snapshot { messages, .. } => {
            emit_event(&app, "update_messages", messages);
        }
        ChatEvent::ThinkingDelta { thinking } => {
            emit_event(&app, "update_thinking_content", thinking);
        }
        ChatEvent::ContentDelta { content } => {
            emit_event(&app, "update_message_content", content);
        }
        ChatEvent::GenerationFinished { .. } => {
            emit_event(&app, "emit_done", ());
        }
        ChatEvent::Error { message } => {
            emit_event(&app, "chat_error", message);
            emit_event(&app, "emit_done", ());
        }
        ChatEvent::HistoryChanged => {
            emit_event(&app, "history_changed", ());
        }
    }
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    chat_service::send(message, &chat_session, &pool, make_callback(app)).await
}

#[tauri::command]
pub async fn resend_message(
    app: AppHandle,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    chat_service::resend(&chat_session, &pool, make_callback(app)).await
}

#[tauri::command]
pub async fn edit_message(
    app: AppHandle,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
    index: usize,
    message: String,
) -> Result<(), ChatError> {
    chat_service::edit(index, message, &chat_session, &pool, make_callback(app)).await
}

#[tauri::command]
pub async fn load_chat(
    id: u32,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<SessionSnapshot, ChatError> {
    chat_service::load_chat(id, &chat_session, &pool).await
}

#[tauri::command]
pub async fn new_chat(
    chat_session: State<'_, Mutex<ChatSession>>,
) -> Result<SessionSnapshot, ChatError> {
    Ok(chat_service::new_chat(&chat_session).await)
}

#[tauri::command]
pub async fn delete_history(
    id: u32,
    chat_session: State<'_, Mutex<ChatSession>>,
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<(), ChatError> {
    chat_service::delete_chat(id, &chat_session, &pool).await
}

#[tauri::command]
pub async fn cancel_generation(
    chat_session: State<'_, Mutex<ChatSession>>,
) -> Result<(), ChatError> {
    chat_service::cancel_generation(&chat_session).await
}

#[tauri::command]
pub async fn get_session(
    chat_session: State<'_, Mutex<ChatSession>>,
) -> Result<SessionSnapshot, ChatError> {
    Ok(chat_service::get_session(&chat_session).await)
}

#[tauri::command]
pub fn get_history(
    pool: State<'_, std::sync::Mutex<Connection>>,
) -> Result<Vec<GetHistory>, ChatError> {
    repository::get_history(&pool)
}