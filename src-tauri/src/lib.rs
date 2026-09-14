use tokio::sync::Mutex;
use crate::commands::ollama::send_message;

mod commands;

struct MessageState {
    messages: Vec<Message>,
}

#[derive(serde::Serialize)]
#[derive(Clone)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(serde::Serialize)]
#[derive(Clone)]
enum Role {
    User,
    Assistant,
}

impl Role {
    fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(MessageState {
            messages: Vec::new(),
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}