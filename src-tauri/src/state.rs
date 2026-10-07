use tokio_util::sync::CancellationToken;

use crate::models::{
    chat_model::{
        ActiveStream, ChatMessage,
        Role::{Assistant, User},
        SessionSnapshot,
    },
    error_model::ChatError,
};

pub struct ChatSession  {
    pub(crate) history_id: Option<u32>,
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) active: Option<ActiveStream>,
    pub(crate) next_generation_id: u32,
}

impl ChatSession {
    pub fn begin_generation(&mut self) -> Result<(u32, CancellationToken), ChatError> {
        if self.active.is_some() {
            return Err(ChatError::Active);
        }

        let token = CancellationToken::new();

        let generation_id = self.next_generation_id;
        self.next_generation_id += 1;

        self.active = Some(ActiveStream {
            generation_id,
            token: token.clone(),
        });

        Ok((generation_id, token))
    }

    pub fn end_generation(&mut self, generation_id: u32) {
        if let Some(active) = &self.active {
            if active.generation_id == generation_id {
                self.active = None;
            }
        }
    }

    pub fn push_user(&mut self, content: String) {
        let user_message = ChatMessage {
            role: User,
            content,
            thinking: None,
        };

        self.messages.push(user_message);
    }

    pub fn push_assistant_placeholder(&mut self) {
        let assistant_placeholder = ChatMessage {
            role: Assistant,
            content: String::new(),
            thinking: None,
        };

        self.messages.push(assistant_placeholder);
    }

    pub fn append_content(&mut self, generation_id: u32, chunk: String) -> bool {
        if let Some(active) = &self.active {
            if active.generation_id != generation_id {
                return false;
            }

            if let Some(message) = self.messages.last_mut() {
                message.content.push_str(&chunk);
                return true;
            }
        }

        false
    }

    pub fn append_thinking(
    &mut self,
    generation_id: u32,
    chunk: String,
    ) -> bool {
        if let Some(active) = &self.active {
            if active.generation_id != generation_id {
                return false;
            }

            if let Some(message) = self.messages.last_mut() {
                match message.thinking.as_mut() {
                    Some(thinking) => thinking.push_str(&chunk),
                    None => message.thinking = Some(chunk),
                }
                return true;
            }
        }

        false
    }

    pub fn pop_last_assistant(&mut self) -> Result<(), ChatError> {
        if let Some(message) = self.messages.last_mut() {
            if !matches!(message.role, Assistant) {
                return Err(ChatError::NotAssistant);
            }
        }

        self.messages.pop();

        Ok(())
    }

    pub fn truncate(&mut self, index: u32) -> Result<(), ChatError> {
        if index as usize >= self.messages.len() {
            return Err(ChatError::InvalidMessageIndex);
        }

        if let Some(message) = self.messages.get(index as usize) {
            if !matches!(message.role, User) {
                return Err(ChatError::NotUser);
            }
        }

        self.messages.truncate(index as usize);

        Ok(())
    }

    pub fn replace(&mut self, history_id: u32, messages: Vec<ChatMessage>) {
        self.history_id = Some(history_id);
        self.messages = messages;
    }

    pub fn reset(&mut self) {
        self.history_id = None;
        self.messages.clear();
        self.active = None;
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            history_id: self.history_id,
            messages: self.messages.clone(),
            generating: self.active.is_some(),
        }
   }
}
