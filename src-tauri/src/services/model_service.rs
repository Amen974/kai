use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::{
    database::repository,
    models::{ai_model::AiModel, chat_event::ChatEvent, error_model::ChatError},
    ollama::{delete_model as ollama_delete_model, get_tags, pull_model, show_model},
    state::ChatSession,
};

pub async fn sync_models(pool: &std::sync::Mutex<Connection>) -> Result<(), ChatError> {
    let tags = get_tags().await?;
    let previous_active = active_model(pool)?.map(|model| model.name);

    let mut models = Vec::with_capacity(tags.models.len());
    for tag in tags.models {
        let supports_vision = show_model(&tag.name)
            .await?
            .capabilities
            .iter()
            .any(|capability| capability == "vision");

        models.push(AiModel {
            active: previous_active.as_deref() == Some(tag.name.as_str()),
            name: tag.name,
            family: tag.details.family,
            parameter_size: tag.details.parameter_size,
            quantization_level: tag.details.quantization_level,
            size_bytes: tag.size,
            modified_at: tag.modified_at,
            supports_vision,
        });
    }

    if !models.iter().any(|model| model.active) {
        if let Some(first) = models.first_mut() {
            first.active = true;
        }
    }

    repository::replace_models(pool, &models)
}

pub async fn sync_if_empty(pool: &std::sync::Mutex<Connection>) -> Result<bool, ChatError> {
    if repository::get_models(pool)?.is_empty() {
        sync_models(pool).await?;
        return Ok(true);
    }

    Ok(false)
}

pub async fn download_model(
    session: &Mutex<ChatSession>,
    pool: &std::sync::Mutex<Connection>,
    name: &str,
    on_event: impl Fn(ChatEvent),
) -> Result<bool, ChatError> {
    let token = session.lock().await.begin_download()?;

    let pulled = pull_model(name, &token, |progress| {
        on_event(ChatEvent::ModelDownloadProgress {
            name: name.to_string(),
            status: progress.status,
            completed: progress.completed,
            total: progress.total,
        });
    })
    .await;

    session.lock().await.end_download();

    if !pulled? {
        return Ok(false);
    }

    sync_models(pool).await?;
    on_event(ChatEvent::ModelsChanged);

    Ok(true)
}

pub async fn cancel_download(session: &Mutex<ChatSession>) {
    let state = session.lock().await;
    if let Some(token) = &state.download {
        token.cancel();
    }
}

pub async fn delete_model(
    pool: &std::sync::Mutex<Connection>,
    name: &str,
) -> Result<(), ChatError> {
    ollama_delete_model(name).await?;
    sync_models(pool).await
}

pub fn active_model(pool: &std::sync::Mutex<Connection>) -> Result<Option<AiModel>, ChatError> {
    Ok(repository::get_models(pool)?
        .into_iter()
        .find(|model| model.active))
}

pub fn set_active_model(pool: &std::sync::Mutex<Connection>, name: &str) -> Result<(), ChatError> {
    repository::set_active_model(pool, name)
}

#[cfg(test)]
mod test {
    use super::*;

    fn in_memory_pool() -> std::sync::Mutex<Connection> {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        crate::database::init::migrate(&mut connection).unwrap();
        std::sync::Mutex::new(connection)
    }

    fn model(name: &str, active: bool) -> AiModel {
        AiModel {
            name: name.to_string(),
            family: "qwen3".to_string(),
            parameter_size: "4B".to_string(),
            quantization_level: "Q4_K_M".to_string(),
            size_bytes: 2_500_000_000,
            modified_at: "2025-01-01T00:00:00Z".to_string(),
            supports_vision: false,
            active,
        }
    }

    fn pool_with(models: &[AiModel]) -> std::sync::Mutex<Connection> {
        let pool = in_memory_pool();
        repository::replace_models(&pool, models).unwrap();
        pool
    }

    #[test]
    fn active_model_returns_none_for_an_empty_catalog() {
        let pool = in_memory_pool();

        assert!(active_model(&pool).unwrap().is_none());
    }

    #[test]
    fn active_model_returns_the_active_row() {
        let pool = pool_with(&[model("qwen3:4b", false), model("llama3:8b", true)]);

        let active = active_model(&pool).unwrap().unwrap();

        assert_eq!(active.name, "llama3:8b");
        assert!(active.active);
    }

    #[test]
    fn active_model_returns_none_when_no_row_is_active() {
        let pool = pool_with(&[model("qwen3:4b", false)]);

        assert!(active_model(&pool).unwrap().is_none());
    }

    #[test]
    fn set_active_model_switches_the_active_row() {
        let pool = pool_with(&[model("qwen3:4b", true), model("llama3:8b", false)]);

        set_active_model(&pool, "llama3:8b").unwrap();

        let models = repository::get_models(&pool).unwrap();
        assert!(models.iter().filter(|model| model.active).count() == 1);
        assert_eq!(active_model(&pool).unwrap().unwrap().name, "llama3:8b");
    }

    #[test]
    fn set_active_model_rejects_an_unknown_model_and_keeps_the_previous_selection() {
        let pool = pool_with(&[model("qwen3:4b", true), model("llama3:8b", false)]);

        let result = set_active_model(&pool, "missing:1b");

        assert!(matches!(result, Err(ChatError::ModelNotFound(name)) if name == "missing:1b"));
        assert_eq!(active_model(&pool).unwrap().unwrap().name, "qwen3:4b");
    }

    #[tokio::test]
    async fn sync_if_empty_skips_the_sync_when_the_catalog_is_populated() {
        let pool = pool_with(&[model("qwen3:4b", true)]);

        let synced = sync_if_empty(&pool).await.unwrap();

        assert!(!synced);
        assert_eq!(repository::get_models(&pool).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cancel_download_cancels_the_active_token() {
        let session = Mutex::new(ChatSession::default());
        let token = session.lock().await.begin_download().unwrap();

        cancel_download(&session).await;

        assert!(token.is_cancelled());
        assert!(session.lock().await.download.is_some());
    }

    #[tokio::test]
    async fn cancel_download_on_idle_session_is_a_noop() {
        let session = Mutex::new(ChatSession::default());

        cancel_download(&session).await;

        assert!(session.lock().await.download.is_none());
    }
}
