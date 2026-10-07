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