use tokio_util::sync::CancellationToken;

use crate::models::chat_model::Message;

pub struct MessageArray {
    pub(crate) message_array: Vec<Message>
}

pub struct CancelToken {
    pub(crate) token: Option<CancellationToken>
}