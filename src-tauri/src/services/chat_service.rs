use futures_util::{StreamExt};
use tokio::sync::Mutex;
use tauri::{State};
use tokio_util::{bytes, sync::CancellationToken};
use tokio::select;

use crate::{
    ChatHistory, models::chat_model::{ChatMessage, OllamaRequest, OllamaResponse, Role::{Assistant, User}}, ollama::post_chat, state::CancelState,
};

pub enum StreamEvent {
    Messages(Vec<ChatMessage>),
    MessageChunk(String),
    Done
}

pub async fn send(
    message: String,
    message_arr: State<'_, Mutex<ChatHistory>>,
    cancel_state: State<'_, Mutex<CancelState>>,
    update_callback: impl Fn(StreamEvent)
) -> Result<(), String> {
    let messages = {
        let mut state = message_arr.lock().await;

        state.messages.push(ChatMessage {
            role: User,
            content: message,
        });

        state.messages.clone()
    };

    update_callback(StreamEvent::Messages(messages.clone()));

    let payload = OllamaRequest {
        model: "qwen3:4b".to_string(),
        messages,
        stream: true,
    };

    let response = post_chat(payload)
        .await
        .map_err(|error| error.to_string())?;

    let stream = response.bytes_stream();

    let token = CancellationToken::new();
    let cloned_token = token.clone();

    cancel_state.lock().await.token = Some(token);

    let result = process_stream(
        stream,
        &*message_arr,
        cloned_token,
        update_callback,
    ).await;

    cancel_state.lock().await.token = None;

    result
}

async fn process_stream(
    mut stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    message_arr: &Mutex<ChatHistory>,
    token: CancellationToken,
    update_callback: impl Fn(StreamEvent),
) -> Result<(), String> {
    let mut buffer = String::new();

    let messages = {
        let mut state = message_arr.lock().await;

        state.messages.push(ChatMessage {
            role: Assistant,
            content: "".to_string(),
        });

        state.messages.clone()
    };

    let message_index = messages.len() - 1;

    loop {
        select! {
            chunk = stream.next() => {
                let chunk = match chunk {
                    Some(Ok(chunk)) => chunk,
                    Some(Err(error)) => return Err(error.to_string()),
                    None => return Ok(()),
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer.drain(..=pos);

                    if line.trim().is_empty() {
                        continue;
                    }

                    let data: OllamaResponse =
                        serde_json::from_str(&line)
                            .map_err(|error| error.to_string())?;

                    if let Some(message) = data.message {
                        let mut state = message_arr.lock().await;

                        state.messages[message_index].content.push_str(&message.content);

                        update_callback(StreamEvent::MessageChunk(message.content));
                    }

                    if data.done { 
                        update_callback(StreamEvent::Done);
                        return Ok(());
                    }

                }
            }

            _ = token.cancelled() => {
                return Ok(());
            }
        }
    }
}