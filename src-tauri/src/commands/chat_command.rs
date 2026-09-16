use tokio::sync::Mutex;
use tauri::{Emitter, State, AppHandle};

use crate::{
    services::chat_service::{Callback, send}, state::{CancelToken, MessageArray},
};

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
    message_arr: State<'_, Mutex<MessageArray>>,
    cancel_state: State<'_, Mutex<CancelToken>>,
) -> Result<(), String> {
    send(message, message_arr, cancel_state, |event| {
        match event {
            Callback::Message(message) => {
                app.emit("chat-chunk", message)
                    .expect("failed to emit chat-chunk");
            }

            Callback::Done => {
                app.emit("chat-done", ())
                    .expect("failed to emit chat-done");
            }
        }
    })
    .await
}

#[tauri::command]
pub async fn cancel_token(token: State<'_, Mutex<CancelToken>>) -> Result<(), String> {
    let token = token.lock().await;

    if let Some(token) = token.token.as_ref() {
        token.cancel();
    }

    Ok(())
}