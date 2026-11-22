#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("database lock poisoned")]
    LockPoisoned,

    #[error("message index out of bounds")]
    InvalidMessageIndex,

    #[error("a generation is active")]
    Active,

    #[error("last message is not Assistant")]
    NotAssistant,

    #[error("message is not User")]
    NotUser,

    #[error("message content must not be empty")]
    EmptyMessage,

    #[error("no active branch")]
    NoActiveBranch,

    #[error("no messages available")]
    NoMessage,

    #[error("model '{0}' not found")]
    ModelNotFound(String),

    #[error("model '{0}' is not installed. Pull it with `ollama pull {0}` or from the model panel (ctrl+d)")]
    ModelNotInstalled(String),

    #[error("no model available. Pull one with `ollama pull <model>` or from the model panel (ctrl+d)")]
    NoActiveModel,

    #[error("Ollama is not installed. Install it from https://ollama.com, then start it with `ollama serve`")]
    OllamaNotInstalled,

    #[error("Ollama is not responding on 127.0.0.1:11434. Start it with `ollama serve` and try again")]
    OllamaNotRunning,

    #[error("failed to download model: {0}")]
    ModelDownloadFailed(String),

    #[error("a model download is active")]
    DownloadActive,

    #[error(transparent)]
    Db(#[from] rusqlite::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("invalid UTF-8 in stream: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl serde::Serialize for ChatError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}