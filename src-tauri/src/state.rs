use tokio_util::sync::CancellationToken;

use crate::models::{
    chat_model::{
        ActiveStream, ChatMessage,
        Role::{Assistant, User},
        SessionSnapshot,
    },
    error_model::ChatError,
};

#[derive(Default)]
pub struct ChatSession  {
    pub(crate) history_id: Option<u32>,
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) active: Option<ActiveStream>,
    pub(crate) next_generation_id: u32,
    pub(crate) active_branch_id: Option<u32>,
    pub(crate) download: Option<CancellationToken>,
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

    pub fn begin_download(&mut self) -> Result<CancellationToken, ChatError> {
        if self.download.is_some() {
            return Err(ChatError::DownloadActive);
        }

       let token = CancellationToken::new(); 
        self.download = Some(token.clone());

        Ok(token)
    }

    pub fn end_download(&mut self) {
        self.download = None;
    }

    pub fn end_generation(&mut self, generation_id: u32) {
        if let Some(active) = &self.active {
            if active.generation_id == generation_id {
                self.active = None;
            }
        }
    }

    pub fn push_user(
        &mut self,
        content: String,
        id: Option<u32>,
        parent_message_id: Option<u32>,
    ) {
        let user_message = ChatMessage {
            id,
            parent_message_id,
            role: User,
            content,
            thinking: None,
        };

        self.messages.push(user_message);
    }

    pub fn push_assistant_placeholder(&mut self) {
        let assistant_placeholder = ChatMessage {
            id: None,
            parent_message_id: None,
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

    pub fn replace(&mut self, history_id: u32, messages: Vec<ChatMessage>, active_branch_id: u32) {
        self.history_id = Some(history_id);
        self.messages = messages;
        self.active_branch_id = Some(active_branch_id);
    }

    pub fn reset(&mut self) {
        self.history_id = None;
        self.messages.clear();
        self.active = None;
        self.active_branch_id = None;
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            history_id: self.history_id,
            messages: self.messages.clone(),
            generating: self.active.is_some(),
            active_branch_id: self.active_branch_id,
        }
   }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_generation_succeeds_on_idle_session() {
        let mut session = ChatSession::default();

        let (_, token) = session.begin_generation().unwrap();

        assert!(!token.is_cancelled());
        assert!(session.active.is_some());
    }

    #[test]
    fn begin_generation_fails_when_one_is_already_active() {
        let mut session = ChatSession::default();
        session.begin_generation().unwrap();

        let result = session.begin_generation();

        assert!(matches!(result, Err(ChatError::Active)));
    }

    #[test]
    fn generation_id_increases_by_one_and_rejected_call_does_not_use_one() {
        let mut session = ChatSession::default();
        let (first_id, _) = session.begin_generation().unwrap();
        let _ = session.begin_generation();

        session.end_generation(first_id);
        let (second_id, _) = session.begin_generation().unwrap();

        assert_eq!(second_id, first_id + 1);
    }

    #[test]
    fn returned_token_is_the_one_stored_in_active() {
        let mut session = ChatSession::default();
        let (_, token) = session.begin_generation().unwrap();

        token.cancel();

        assert!(session.active.as_ref().unwrap().token.is_cancelled());
    }

    #[test]
    fn end_generation_on_idle_session_is_a_noop() {
        let mut session = ChatSession::default();

        session.end_generation(0);

        assert!(session.active.is_none());
    }

    #[test]
    fn end_generation_succeeds_on_correct_generation_id() {
        let mut session = ChatSession::default();
        
        let (generation_id, _) = session.begin_generation().unwrap();

        session.end_generation(generation_id);

        assert!(session.active.is_none());
    }

    #[test]
    fn end_generation_ignores_stale_generation_id() {
        let mut session = ChatSession::default();

        let (first_generation, _) = session.begin_generation().unwrap();
        session.end_generation(first_generation);

        
        let (second_generation, _) = session.begin_generation().unwrap();
        session.end_generation(first_generation);

        assert_eq!(session.active.as_ref().unwrap().generation_id, second_generation);
    }

    #[test]
    fn begin_download_succeeds_on_idle_session() {
        let mut session = ChatSession::default();

        let token = session.begin_download().unwrap();

        assert!(!token.is_cancelled());
        assert!(session.download.is_some());
    }

    #[test]
    fn begin_download_fails_when_one_is_already_active() {
        let mut session = ChatSession::default();
        session.begin_download().unwrap();

        let result = session.begin_download();

        assert!(matches!(result, Err(ChatError::DownloadActive)));
    }

    #[test]
    fn returned_download_token_is_the_one_stored_in_download() {
        let mut session = ChatSession::default();
        let token = session.begin_download().unwrap();

        token.cancel();

        assert!(session.download.as_ref().unwrap().is_cancelled());
    }

    #[test]
    fn end_download_clears_the_active_download() {
        let mut session = ChatSession::default();
        session.begin_download().unwrap();

        session.end_download();

        assert!(session.download.is_none());
    }

    #[test]
    fn end_download_on_idle_session_is_a_noop() {
        let mut session = ChatSession::default();

        session.end_download();

        assert!(session.download.is_none());
    }

    #[test]
    fn begin_download_succeeds_again_after_end_download() {
        let mut session = ChatSession::default();
        session.begin_download().unwrap();
        session.end_download();

        let token = session.begin_download().unwrap();

        assert!(!token.is_cancelled());
    }

    #[test]
    fn push_user_appends_message_with_given_fields() {
        let mut session = ChatSession::default();

        session.push_user("hello".to_string(), Some(1), Some(9));

        assert_eq!(session.messages.len(), 1);
        let message = &session.messages[0];
        assert_eq!(message.id, Some(1));
        assert_eq!(message.parent_message_id, Some(9));
        assert!(matches!(message.role, User));
        assert_eq!(message.content, "hello");
        assert!(message.thinking.is_none());
    }

    #[test]
    fn push_user_appends_after_existing_messages() {
        let mut session = ChatSession::default();
        session.push_user("first".to_string(), Some(1), None);

        session.push_user("second".to_string(), Some(2), Some(1));

        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].content, "first");
        assert_eq!(session.messages[1].content, "second");
    }

    #[test]
    fn push_assistant_placeholder_appends_empty_assistant_message() {
        let mut session = ChatSession::default();
        session.push_user("hello".to_string(), Some(1), None);

        session.push_assistant_placeholder();

        assert_eq!(session.messages.len(), 2);
        let message = &session.messages[1];
        assert_eq!(message.id, None);
        assert_eq!(message.parent_message_id, None);
        assert!(matches!(message.role, Assistant));
        assert!(message.content.is_empty());
        assert!(message.thinking.is_none());
    }

    #[test]
    fn append_content_returns_false_without_active_generation() {
        let mut session = ChatSession::default();
        session.push_assistant_placeholder();

        let appended = session.append_content(0, "chunk".to_string());

        assert!(!appended);
        assert!(session.messages[0].content.is_empty());
    }

    #[test]
    fn append_content_returns_false_for_stale_generation_id() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_assistant_placeholder();

        let appended = session.append_content(generation_id + 1, "chunk".to_string());

        assert!(!appended);
        assert!(session.messages[0].content.is_empty());
    }

    #[test]
    fn append_content_returns_false_without_messages() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();

        let appended = session.append_content(generation_id, "chunk".to_string());

        assert!(!appended);
        assert!(session.messages.is_empty());
    }

    #[test]
    fn append_content_appends_chunks_to_last_message() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_assistant_placeholder();

        let first = session.append_content(generation_id, "Hello".to_string());
        let second = session.append_content(generation_id, " world".to_string());

        assert!(first);
        assert!(second);
        assert_eq!(session.messages[0].content, "Hello world");
    }

    #[test]
    fn append_content_only_touches_the_last_message() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_user("question".to_string(), Some(1), None);
        session.push_assistant_placeholder();

        session.append_content(generation_id, "answer".to_string());

        assert_eq!(session.messages[0].content, "question");
        assert_eq!(session.messages[1].content, "answer");
    }

    #[test]
    fn append_thinking_returns_false_without_active_generation() {
        let mut session = ChatSession::default();
        session.push_assistant_placeholder();

        let appended = session.append_thinking(0, "thought".to_string());

        assert!(!appended);
        assert!(session.messages[0].thinking.is_none());
    }

    #[test]
    fn append_thinking_returns_false_for_stale_generation_id() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_assistant_placeholder();

        let appended = session.append_thinking(generation_id + 1, "thought".to_string());

        assert!(!appended);
        assert!(session.messages[0].thinking.is_none());
    }

    #[test]
    fn append_thinking_returns_false_without_messages() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();

        let appended = session.append_thinking(generation_id, "thought".to_string());

        assert!(!appended);
        assert!(session.messages.is_empty());
    }

    #[test]
    fn append_thinking_sets_thinking_when_none() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_assistant_placeholder();

        let appended = session.append_thinking(generation_id, "thought".to_string());

        assert!(appended);
        assert_eq!(session.messages[0].thinking.as_deref(), Some("thought"));
    }

    #[test]
    fn append_thinking_appends_to_existing_thinking() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_assistant_placeholder();

        session.append_thinking(generation_id, "first".to_string());
        session.append_thinking(generation_id, " second".to_string());

        assert_eq!(session.messages[0].thinking.as_deref(), Some("first second"));
    }

    #[test]
    fn pop_last_assistant_removes_last_assistant_message() {
        let mut session = ChatSession::default();
        session.push_user("hello".to_string(), Some(1), None);
        session.push_assistant_placeholder();

        let result = session.pop_last_assistant();

        assert!(result.is_ok());
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "hello");
    }

    #[test]
    fn pop_last_assistant_fails_when_last_message_is_user() {
        let mut session = ChatSession::default();
        session.push_user("hello".to_string(), Some(1), None);

        let result = session.pop_last_assistant();

        assert!(matches!(result, Err(ChatError::NotAssistant)));
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn pop_last_assistant_on_empty_session_is_a_noop() {
        let mut session = ChatSession::default();

        let result = session.pop_last_assistant();

        assert!(result.is_ok());
        assert!(session.messages.is_empty());
    }

    #[test]
    fn truncate_fails_when_index_is_out_of_bounds() {
        let mut session = ChatSession::default();
        session.push_user("hello".to_string(), Some(1), None);

        let result = session.truncate(1);

        assert!(matches!(result, Err(ChatError::InvalidMessageIndex)));
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn truncate_on_empty_session_fails() {
        let mut session = ChatSession::default();

        let result = session.truncate(0);

        assert!(matches!(result, Err(ChatError::InvalidMessageIndex)));
    }

    #[test]
    fn truncate_fails_when_message_at_index_is_not_user() {
        let mut session = ChatSession::default();
        session.push_user("hello".to_string(), Some(1), None);
        session.push_assistant_placeholder();

        let result = session.truncate(1);

        assert!(matches!(result, Err(ChatError::NotUser)));
        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn truncate_removes_index_and_everything_after_it() {
        let mut session = ChatSession::default();
        session.push_user("first".to_string(), Some(1), None);
        session.push_assistant_placeholder();
        session.push_user("second".to_string(), Some(2), Some(1));
        session.push_assistant_placeholder();

        let result = session.truncate(2);

        assert!(result.is_ok());
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].content, "first");
        assert!(matches!(session.messages[1].role, Assistant));
    }

    #[test]
    fn truncate_at_zero_clears_all_messages() {
        let mut session = ChatSession::default();
        session.push_user("first".to_string(), Some(1), None);
        session.push_assistant_placeholder();

        let result = session.truncate(0);

        assert!(result.is_ok());
        assert!(session.messages.is_empty());
    }

    #[test]
    fn replace_sets_history_messages_and_branch() {
        let mut source = ChatSession::default();
        source.push_user("hello".to_string(), Some(1), None);
        source.push_assistant_placeholder();
        let messages = source.snapshot().messages;

        let mut session = ChatSession::default();
        session.push_user("old".to_string(), Some(7), None);

        session.replace(3, messages, 5);

        assert_eq!(session.history_id, Some(3));
        assert_eq!(session.active_branch_id, Some(5));
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.messages[0].content, "hello");
        assert!(matches!(session.messages[1].role, Assistant));
    }

    #[test]
    fn reset_clears_session_state() {
        let mut session = ChatSession::default();
        session.begin_generation().unwrap();
        session.push_user("hello".to_string(), Some(1), None);
        session.replace(3, Vec::new(), 5);

        session.reset();

        assert!(session.history_id.is_none());
        assert!(session.messages.is_empty());
        assert!(session.active.is_none());
        assert!(session.active_branch_id.is_none());
    }

    #[test]
    fn snapshot_of_empty_session_is_empty() {
        let session = ChatSession::default();

        let snapshot = session.snapshot();

        assert!(snapshot.history_id.is_none());
        assert!(snapshot.messages.is_empty());
        assert!(!snapshot.generating);
        assert!(snapshot.active_branch_id.is_none());
    }

    #[test]
    fn snapshot_reflects_current_state() {
        let mut session = ChatSession::default();
        session.replace(3, Vec::new(), 5);
        session.push_user("hello".to_string(), Some(1), None);

        let snapshot = session.snapshot();

        assert_eq!(snapshot.history_id, Some(3));
        assert_eq!(snapshot.messages.len(), 1);
        assert_eq!(snapshot.messages[0].content, "hello");
        assert!(!snapshot.generating);
        assert_eq!(snapshot.active_branch_id, Some(5));
    }

    #[test]
    fn snapshot_reports_generating_while_a_generation_is_active() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();

        assert!(session.snapshot().generating);

        session.end_generation(generation_id);

        assert!(!session.snapshot().generating);
    }

    #[test]
    fn snapshot_messages_are_detached_from_session() {
        let mut session = ChatSession::default();
        let (generation_id, _) = session.begin_generation().unwrap();
        session.push_assistant_placeholder();

        let snapshot = session.snapshot();
        session.append_content(generation_id, "later".to_string());

        assert!(snapshot.messages[0].content.is_empty());
        assert_eq!(session.messages[0].content, "later");
    }
}