use futures_util::{StreamExt};
use tokio::sync::Mutex;
use tauri::{State};
use tokio_util::{bytes, sync::CancellationToken};
use tokio::select;

use crate::{
    MessageArray,
    models::chat_model::{Message, OllamaRequest, OllamaResponse, Role::{Assistant, User}},
    ollama::post_chat,
    state::CancelToken,
};

pub enum Callback {
    Message(String),
    Done
}

pub async fn send(
    message: String,
    message_arr: State<'_, Mutex<MessageArray>>,
    cancel_state: State<'_, Mutex<CancelToken>>,
    callback: impl Fn(Callback)
) -> Result<(), String> {
    let messages = {
        let mut state = message_arr.lock().await;

        state.message_array.push(Message {
            role: User,
            content: message,
        });

        state.message_array.clone()
    };

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

process_stream(
    stream,
    message_arr,
    cloned_token,
    callback,
).await?;

cancel_state.lock().await.token = None;

Ok(())
}

async fn process_stream(
    mut stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    message_arr: State<'_, Mutex<MessageArray>>,
    token: CancellationToken,
    callback: impl Fn(Callback),
) -> Result<(), String> {
    let mut buffer = String::new();
    let mut full_message = String::new();

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
                        full_message.push_str(&message.content);
                        callback(Callback::Message(message.content));
                    }

                    if data.done {
                        let mut state = message_arr.lock().await;

                        state.message_array.push(Message {
                            role: Assistant,
                            content: full_message,
                        });

                        callback(Callback::Done);
                        return Ok(());
                    }
                }
            }

            _ = token.cancelled() => {
                drop(stream);
                return Ok(());
            }
        }
    }
}