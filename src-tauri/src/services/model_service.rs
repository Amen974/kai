use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::{
    database::repository,
    models::{ai_model::AiModel, chat_event::ChatEvent, error_model::ChatError},
    ollama::{delete_model as ollama_delete_model, get_tags, pull_model, show_model},
    state::ChatSession,
};

pub async fn sync_models(
    pool: &std::sync::Mutex<Connection>,
    base_url: &str,
) -> Result<(), ChatError> {
    let tags = get_tags(base_url).await?;
    let previous_active = active_model(pool)?.map(|model| model.name);

    let mut models = Vec::with_capacity(tags.models.len());
    for tag in tags.models {
        let supports_vision = show_model(base_url, &tag.name)
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

pub async fn sync_if_empty(
    pool: &std::sync::Mutex<Connection>,
    base_url: &str,
) -> Result<bool, ChatError> {
    if repository::get_models(pool)?.is_empty() {
        sync_models(pool, base_url).await?;
        return Ok(true);
    }

    Ok(false)
}

pub async fn download_model(
    session: &Mutex<ChatSession>,
    pool: &std::sync::Mutex<Connection>,
    base_url: &str,
    name: &str,
    on_event: impl Fn(ChatEvent),
) -> Result<bool, ChatError> {
    let token = session.lock().await.begin_download()?;

    let pulled = pull_model(base_url, name, &token, |progress| {
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

    sync_models(pool, base_url).await?;
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
    base_url: &str,
    name: &str,
) -> Result<(), ChatError> {
    ollama_delete_model(base_url, name).await?;
    sync_models(pool, base_url).await
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
mod tests {
    use super::*;
    use crate::{
        database::test_db::in_memory_pool,
        ollama::mock::{MockOllama, MockRoute},
    };
    use std::cell::RefCell;

    const TAGS: &str = r#"{"models":[
        {"name":"alpha:1b","modified_at":"2026-01-01T00:00:00Z","size":1000,"details":{"family":"llama","parameter_size":"1B","quantization_level":"Q4_0"}},
        {"name":"beta:2b","modified_at":"2026-01-02T00:00:00Z","size":2000,"details":{"family":"qwen","parameter_size":"2B","quantization_level":"Q4_K_M"}}]}"#;

    fn sync_routes() -> Vec<MockRoute> {
        vec![
            MockRoute::json("GET", "/api/tags", TAGS),
            MockRoute::json("POST", "/api/show", r#"{"capabilities":["completion","vision"]}"#),
        ]
    }

    fn model_names(pool: &std::sync::Mutex<Connection>) -> Vec<String> {
        repository::get_models(pool)
            .expect("get models")
            .into_iter()
            .map(|stored| stored.name)
            .collect()
    }

    #[tokio::test]
    async fn sync_models_caches_the_catalog_and_activates_the_first_model() {
        let server = MockOllama::start(sync_routes()).await;
        let pool = in_memory_pool();

        sync_models(&pool, &server.base_url)
            .await
            .expect("sync models");

        let models = repository::get_models(&pool).expect("get models");
        assert_eq!(model_names(&pool), vec!["alpha:1b", "beta:2b"]);
        assert_eq!(models[0].family, "llama");
        assert_eq!(models[0].size_bytes, 1000);
        assert!(models[0].supports_vision);
        assert!(models[0].active);
        assert!(!models[1].active);
    }

    #[tokio::test]
    async fn sync_models_reports_models_without_vision_support() {
        let server = MockOllama::start(vec![
            MockRoute::json("GET", "/api/tags", TAGS),
            MockRoute::json("POST", "/api/show", r#"{"capabilities":["completion"]}"#),
        ])
        .await;
        let pool = in_memory_pool();

        sync_models(&pool, &server.base_url)
            .await
            .expect("sync models");

        assert!(repository::get_models(&pool)
            .expect("get models")
            .iter()
            .all(|stored| !stored.supports_vision));
    }

    #[tokio::test]
    async fn sync_models_keeps_the_selected_model_active() {
        let server = MockOllama::start(sync_routes()).await;
        let pool = in_memory_pool();
        sync_models(&pool, &server.base_url)
            .await
            .expect("first sync");
        set_active_model(&pool, "beta:2b").expect("set active model");

        sync_models(&pool, &server.base_url)
            .await
            .expect("second sync");

        assert_eq!(
            active_model(&pool).expect("active model").map(|stored| stored.name),
            Some("beta:2b".to_string())
        );
    }

    #[tokio::test]
    async fn active_model_returns_none_for_an_empty_catalog() {
        let pool = in_memory_pool();

        assert!(active_model(&pool).expect("active model").is_none());
    }

    #[tokio::test]
    async fn sync_if_empty_syncs_the_catalog_when_it_is_empty() {
        let server = MockOllama::start(sync_routes()).await;
        let pool = in_memory_pool();

        let synced = sync_if_empty(&pool, &server.base_url)
            .await
            .expect("sync if empty");

        assert!(synced);
        assert_eq!(model_names(&pool), vec!["alpha:1b", "beta:2b"]);
    }

    #[tokio::test]
    async fn sync_if_empty_skips_ollama_when_the_catalog_is_populated() {
        let server = MockOllama::start(sync_routes()).await;
        let pool = in_memory_pool();
        sync_models(&pool, &server.base_url)
            .await
            .expect("sync models");

        let synced = sync_if_empty(&pool, "http://127.0.0.1:1")
            .await
            .expect("sync if empty");

        assert!(!synced);
        assert_eq!(model_names(&pool), vec!["alpha:1b", "beta:2b"]);
    }

    #[tokio::test]
    async fn download_model_reports_progress_and_resyncs_the_catalog() {
        let server = MockOllama::start(vec![
            MockRoute::json(
                "POST",
                "/api/pull",
                "{\"status\":\"pulling manifest\",\"completed\":10,\"total\":100}\n{\"status\":\"success\"}",
            ),
            MockRoute::json("GET", "/api/tags", TAGS),
            MockRoute::json("POST", "/api/show", r#"{"capabilities":[]}"#),
        ])
        .await;
        let pool = in_memory_pool();
        let session = Mutex::new(ChatSession::empty());
        let events = RefCell::new(Vec::new());

        let pulled = download_model(
            &session,
            &pool,
            &server.base_url,
            "alpha:1b",
            |event| events.borrow_mut().push(event),
        )
        .await
        .expect("download model");

        assert!(pulled);
        assert_eq!(model_names(&pool), vec!["alpha:1b", "beta:2b"]);
        assert!(session.lock().await.download.is_none());

        let events = events.into_inner();
        assert_eq!(events.len(), 3);
        match &events[0] {
            ChatEvent::ModelDownloadProgress {
                name,
                status,
                completed,
                total,
            } => {
                assert_eq!(name, "alpha:1b");
                assert_eq!(status, "pulling manifest");
                assert_eq!(*completed, 10);
                assert_eq!(*total, 100);
            }
            _ => panic!("expected a download progress event"),
        }
        assert!(matches!(events[1], ChatEvent::ModelDownloadProgress { .. }));
        assert!(matches!(events[2], ChatEvent::ModelsChanged));
    }

    #[tokio::test]
    async fn download_model_returns_false_when_cancelled() {
        let server = MockOllama::start(vec![MockRoute::hold_open(
            "POST",
            "/api/pull",
            "{\"status\":\"pulling manifest\"}\n",
        )])
        .await;
        let pool = in_memory_pool();
        let session = Mutex::new(ChatSession::empty());
        let (started, mut progress) = tokio::sync::mpsc::unbounded_channel();

        let download = async {
            download_model(
                &session,
                &pool,
                &server.base_url,
                "alpha:1b",
                move |event| {
                    if matches!(event, ChatEvent::ModelDownloadProgress { .. }) {
                        started.send(()).expect("progress channel");
                    }
                },
            )
            .await
        };
        let stop = async {
            progress.recv().await.expect("first progress");
            cancel_download(&session).await;
        };
        let (pulled, ()) = tokio::join!(download, stop);

        assert!(!pulled.expect("download model"));
        assert!(session.lock().await.download.is_none());
    }

    #[tokio::test]
    async fn download_model_reports_a_failed_pull_and_clears_the_download() {
        let server = MockOllama::start(vec![MockRoute::json(
            "POST",
            "/api/pull",
            "{\"error\":\"pull failed\"}\n",
        )])
        .await;
        let pool = in_memory_pool();
        let session = Mutex::new(ChatSession::empty());

        let result = download_model(&session, &pool, &server.base_url, "alpha:1b", |_| {}).await;

        assert!(matches!(
            result,
            Err(ChatError::ModelDownloadFailed(message)) if message == "pull failed"
        ));
        assert!(session.lock().await.download.is_none());
    }

    #[tokio::test]
    async fn download_model_rejects_a_second_concurrent_download() {
        let pool = in_memory_pool();
        let session = Mutex::new(ChatSession::empty());
        session
            .lock()
            .await
            .begin_download()
            .expect("first download");

        let result = download_model(&session, &pool, "http://127.0.0.1:1", "alpha:1b", |_| {}).await;

        assert!(matches!(result, Err(ChatError::DownloadActive)));
    }

    #[tokio::test]
    async fn delete_model_removes_the_model_and_resyncs_the_catalog() {
        let server = MockOllama::start(vec![
            MockRoute::status("DELETE", "/api/delete", 200, ""),
            MockRoute::json(
                "GET",
                "/api/tags",
                r#"{"models":[{"name":"beta:2b","modified_at":"2026-01-02T00:00:00Z","size":2000,"details":{"family":"qwen","parameter_size":"2B","quantization_level":"Q4_K_M"}}]}"#,
            ),
            MockRoute::json("POST", "/api/show", r#"{"capabilities":[]}"#),
        ])
        .await;
        let pool = in_memory_pool();

        delete_model(&pool, &server.base_url, "alpha:1b")
            .await
            .expect("delete model");

        assert_eq!(model_names(&pool), vec!["beta:2b"]);
    }
}
