use tokio_util::sync::CancellationToken;

use crate::models::chat_model::ChatMessage;

pub struct ChatHistory {
    pub(crate) messages: Vec<ChatMessage>
}

pub struct CancelState {
    pub(crate) token: Option<CancellationToken>
}