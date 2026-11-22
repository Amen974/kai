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

    #[error("no active model selected")]
    NoActiveModel,

    #[error("failed to download model: {0}")]
    ModelDownloadFailed(String),

    #[error("a model download is active")]
    DownloadActive,

    #[error("unsupported attachment type '{0}'")]
    UnsupportedAttachmentType(String),

    #[error("cannot read attachment path")]
    InvalidAttachmentPath,

    #[error("attachment '{name}' is {size_bytes} bytes, above the {limit} byte limit")]
    AttachmentTooLarge {
        name: String,
        size_bytes: u64,
        limit: u64,
    },

    #[error("attachment '{0}' is empty")]
    EmptyAttachment(String),

    #[error("model '{0}' cannot read images")]
    AttachmentNeedsVisionModel(String),

    #[error(transparent)]
    Db(#[from] rusqlite::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("invalid UTF-8 in stream: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl serde::Serialize for ChatError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}