use crate::models::chat::Message;

pub struct MessageState {
    pub(crate) messages: Vec<Message>,
}