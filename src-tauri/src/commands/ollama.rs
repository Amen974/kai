use futures_util::StreamExt;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::{Message, MessageState, Role::{ Assistant, User}};

#[derive(serde::Serialize)]
#[derive(Clone)]
struct OllamaRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(serde::Deserialize)]
#[derive(Debug)]
struct OllamaMessage {
    content: String,
}

#[derive(serde::Deserialize)]
struct OllamaResponse {
    message: Option<OllamaMessage>,
    done: bool,
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
    message_arr: State<'_, Mutex<MessageState>>,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let messages = {
        let mut state = message_arr.lock().await;

        state.messages.push(Message {
            role: User.as_str(),
            content: message,
        });

        state.messages.clone()
    };

    let payload = OllamaRequest {
        model: "qwen3:4b".to_string(),
        messages,
        stream: true,
    };

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Ollama returned status: {}",
            response.status()
        ));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut full_message = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;

        let text = String::from_utf8_lossy(&chunk);
        println!("RAW CHUNK: {}", text);

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].to_string();
            buffer.drain(..=pos);

            if line.trim().is_empty() {
                continue;
            }

            let data: OllamaResponse =
                serde_json::from_str(&line).map_err(|error| error.to_string())?;
            println!("PARSED: {:?}", line);
            println!("AI RESPONSE: {:?}", data.message);
            println!("DONE: {}", data.done);

            if let Some(message) = data.message {
                full_message.push_str(&message.content);

                app.emit("chat-chunk", message.content)
                    .map_err(|error| error.to_string())?;
            }

            if data.done {
                let mut state = message_arr.lock().await;

                state.messages.push(Message {
                    role: Assistant.as_str(),
                    content: full_message,
                });

                app.emit("chat-done", ())
                    .map_err(|error| error.to_string())?;

                return Ok(());
            }
        }
    }

    Ok(())
}