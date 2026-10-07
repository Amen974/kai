use serde::Serialize;

use crate::models::chat_model::ChatMessage;

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum ChatEvent {
    Snapshot {
        history_id: Option<u32>,
        messages: Vec<ChatMessage>,
        generating: bool,
    },

    ContentDelta {
        content: String,
    },

    ThinkingDelta {
        thinking: String,
    },

    GenerationFinished {
        reason: String,
    },

    Error {
        message: String,
    },

    HistoryChanged,
}