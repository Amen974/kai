use futures_util::StreamExt;
use tokio::sync::Mutex;
use tauri::State;

use crate::{MessageState, models::chat::{Message, OllamaRequest, OllamaResponse, Role::{Assistant, User}}, ollama::post_chat};

pub enum Callback {
    Message(String),
    Done
}

pub async fn send(
    message: String,
    message_arr: State<'_, Mutex<MessageState>>,
    callback: impl Fn(Callback)
) -> Result<(), String> {
    let messages = {
        let mut state = message_arr.lock().await;

        state.messages.push(Message {
            role: User,
            content: message,
        });

        state.messages.clone()
    };

    let payload = OllamaRequest {
        model: "qwen3:4b".to_string(),
        messages,
        stream: true,
    };

    let response = post_chat(payload)
        .await
        .map_err(|error| error.to_string())?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut full_message = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].to_string();
            buffer.drain(..=pos);

            if line.trim().is_empty() {
                continue;
            }

            let data: OllamaResponse =
                serde_json::from_str(&line).map_err(|error| error.to_string())?;

            if let Some(message) = data.message {
                full_message.push_str(&message.content);

                callback(Callback::Message(message.content));
            }

            if data.done {
                let mut state = message_arr.lock().await;

                state.messages.push(Message {
                    role: Assistant,
                    content: full_message,
                });

                callback(Callback::Done);

                return Ok(());
            }
        }
    }

    Ok(())
}