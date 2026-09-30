use futures_util::StreamExt;
use rusqlite::Connection;
use serde_json;
use tauri::State;
use tokio::select;
use tokio::sync::Mutex;
use tokio_util::{bytes::Bytes, sync::CancellationToken};

use crate::{
    models::chat_model::{
        ChatMessage, GetHistory, OllamaRequest, OllamaResponse,
        Role::{Assistant, User},
    },
    ollama::post_chat,
    state::{CancelState, ChatHistory},
};

pub enum StreamEvent {
    Message(ChatMessage),
    ThinkingChunk(String),
    MessageChunk(String),
    Done,
}

pub async fn send(
    message: String,
    message_arr: State<'_, Mutex<ChatHistory>>,
    cancel_state: State<'_, Mutex<CancelState>>,
    id: Option<u32>,
    pool: State<'_, std::sync::Mutex<Connection>>,
    update_callback: impl Fn(StreamEvent),
) -> Result<(), String> {
    let (messages, last_user_message) = {
        let mut state = message_arr.lock().await;
        let user_msg = ChatMessage {
            role: User,
            content: message,
        };
        state.messages.push(user_msg.clone());
        (state.messages.clone(), user_msg)
    };

    handle_sql_update(&pool, id, &last_user_message)?;
    update_callback(StreamEvent::Message(last_user_message));

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
    cancel_state.lock().await.token = Some(token.clone());

    let result = process_stream(
        stream,
        &*message_arr,
        token,
        id,
        &pool,
        update_callback,
    )
    .await;

    cancel_state.lock().await.token = None;

    result
}

pub async fn cancel_token(cancel_state: &Mutex<CancelState>) -> Result<(), String> {
    let cancel_state = cancel_state.lock().await;

    if let Some(token) = cancel_state.token.as_ref() {
        token.cancel();
    }

    Ok(())
}

pub async fn edit_message(
    message_arr: &Mutex<ChatHistory>,
    message: String,
    index: usize,
) -> Result<Vec<ChatMessage>, String> {
    let mut messages = message_arr.lock().await;

    if index >= messages.messages.len() {
        return Err("Message index out of bounds".to_string());
    }

    messages.messages[index].content = message;
    messages.messages.truncate(index);

    Ok(messages.messages.clone())
}

pub fn get_history(pool: &std::sync::Mutex<Connection>, offset: i32) -> Result<Vec<GetHistory>, String> {
    let conn = pool.lock().map_err(|_| "database lock poisoned".to_string())?;

    let mut result = conn
        .prepare("SELECT id, title FROM history ORDER BY updated_at LIMIT 20 OFFSET ?1")
        .map_err(|error| error.to_string())?;

    let row_map = result
        .query_map([offset], |row| {
            Ok(GetHistory {
                id: row.get(0)?,
                title: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let history: Vec<GetHistory> = row_map
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(history)
}

pub async fn get_messages(
    pool: &std::sync::Mutex<Connection>,
    message_arr: &Mutex<ChatHistory>,
    id: i32,
) -> Result<Vec<ChatMessage>, String> {
    let fetched_messages = {
        let conn = pool.lock().map_err(|_| "database lock poisoned".to_string())?;

        let mut stmt = conn
            .prepare("SELECT role, content FROM messages WHERE history_id = ?1 ORDER BY id")
            .map_err(|error| error.to_string())?;

        let mapped_rows = stmt
            .query_map([id], |row: &rusqlite::Row<'_>| {
                let role_str: String = row.get(0)?;
                let role = match role_str.as_str() {
                    "user" => User,
                    "assistant" => Assistant,
                    _ => {
                        return Err(rusqlite::Error::InvalidColumnType(
                            0,
                            "role".to_string(),
                            rusqlite::types::Type::Text,
                        ));
                    }
                };

                Ok(ChatMessage {
                    role,
                    content: row.get(1)?,
                })
            })
            .map_err(|error| error.to_string())?;

        mapped_rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };

    let mut messages = message_arr.lock().await;
    messages.messages = fetched_messages.clone();

    Ok(fetched_messages)
}

async fn process_stream(
    mut stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
    message_arr: &Mutex<ChatHistory>,
    token: CancellationToken,
    id: Option<u32>,
    pool: &std::sync::Mutex<Connection>,
    update_callback: impl Fn(StreamEvent),
) -> Result<(), String> {
    let mut buffer = String::new();

    let message_index = {
        let mut state = message_arr.lock().await;
        state.messages.push(ChatMessage {
            role: Assistant,
            content: String::new(),
        });
        let idx = state.messages.len() - 1;
        update_callback(StreamEvent::Message(state.messages[idx].clone()));
        idx
    };

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

                    let data: OllamaResponse = serde_json::from_str(&line)
                        .map_err(|error| error.to_string())?;

                    if let Some(message) = data.message {
                        if let Some(thinking) = message.thinking {
                            if !thinking.is_empty() {
                                update_callback(StreamEvent::ThinkingChunk(thinking));
                            }
                        }

                        if !message.content.is_empty() {
                            let mut state = message_arr.lock().await;
                            state.messages[message_index].content.push_str(&message.content);
                            update_callback(StreamEvent::MessageChunk(message.content));
                        }
                    }

                    if data.done {
                        let final_assistant_message = {
                            let state = message_arr.lock().await;
                            state.messages[message_index].clone()
                        };

                        handle_sql_update(pool, id, &final_assistant_message)?;

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

fn handle_sql_update(
    pool: &std::sync::Mutex<Connection>,
    id: Option<u32>,
    message: &ChatMessage,
) -> Result<(), String> {
    let conn = pool.lock().map_err(|_| "database lock poisoned".to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    let history_id = match id {
        Some(id) => id,
        None => conn
            .query_row(
                "INSERT INTO history (title, created_at, updated_at) VALUES (?1, ?2, ?3) RETURNING id",
                ["test1", &now, &now],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?,
    };

    let role_str = match message.role {
        User => "user",
        Assistant => "assistant",
    };

    conn.execute(
        "INSERT INTO messages (history_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
        (history_id, role_str, &message.content, &now),
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}