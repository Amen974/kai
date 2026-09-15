use tokio::sync::Mutex;
use tauri::{Emitter, State, AppHandle};

use crate::{
    services::chat_service::{Callback, send}, state::MessageState,
};

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
    message_arr: State<'_, Mutex<MessageState>>,
) -> Result<(), String> {
    send(message, message_arr, |event| {
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