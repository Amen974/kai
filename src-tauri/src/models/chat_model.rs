use tokio_util::sync::CancellationToken;

#[derive(serde::Serialize)]
#[derive(Clone)]
pub struct ChatMessage {
    pub(crate) role: Role,
    pub(crate) content: String,
    pub(crate) thinking: Option<String>,
}

#[derive(serde::Serialize)]
#[derive(Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Role::User),
            "assistant" => Some(Role::Assistant),
            _ => None,
        }
    }
}

#[derive(serde::Serialize)]
#[derive(Clone)]
pub struct OllamaRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) stream: bool,
}

#[derive(serde::Serialize)]
#[derive(Clone)]
pub struct OllamaRequestGenerate {
    pub(crate) model: String,
    pub(crate) prompt: String,
    pub(crate) stream: bool,
}

#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct OllamaMessage {
    pub(crate) content: String,
    #[serde(default)]
    pub(crate) thinking: Option<String>,
}

#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct OllamaResponse {
    pub(crate) message: Option<OllamaMessage>,
    pub(crate) done: bool,
}

#[derive(serde::Deserialize)]
pub struct OllamaGenerateResponse {
    pub(crate) response: String,
}

#[derive(serde::Serialize)]
#[derive(Clone)]
pub struct GetHistory {
    pub(crate) id: u32,
    pub(crate) title: String,
}

pub struct ActiveStream {
    pub generation_id: u32,
    pub token: CancellationToken,
}

#[derive(serde::Serialize, Clone)]
pub struct SessionSnapshot {
    pub history_id: Option<u32>,
    pub messages: Vec<ChatMessage>,
    pub generating: bool,
}