use tokio::sync::Mutex;
use tauri::{Emitter, State, AppHandle};

use crate::{
    services::chat_service::{StreamEvent, send}, state::{CancelState, ChatHistory},
};

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
    message_arr: State<'_, Mutex<ChatHistory>>,
    cancel_state: State<'_, Mutex<CancelState>>,
) -> Result<(), String> {
    send(message, message_arr, cancel_state, |event| {
        match event {
            StreamEvent::Message(message) => {
                let _ = app.emit("update_message", message)
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
    }).await
}

#[tauri::command]
pub async fn cancel_token(token: State<'_, Mutex<CancelState>>) -> Result<(), String> {
    let token = token.lock().await;

    if let Some(token) = token.token.as_ref() {
        token.cancel();
    }

    Ok(())
}

#[tauri::command]
pub async fn edit_message(
    app: AppHandle,
    message_arr: State<'_, Mutex<ChatHistory>>,
    message: String,
    index: usize,
) -> Result<(), String> {
    let mut messages = message_arr.lock().await;

    if index >= messages.messages.len() {
        return Err("Message index out of bounds".to_string());
    }

    messages.messages[index].content = message;
    messages.messages.truncate(index);

    let clone = messages.messages.clone();

    app.emit("update_messages", clone)
        .map_err(|error| error.to_string())?;

    Ok(())
}