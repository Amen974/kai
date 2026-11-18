use rusqlite::Connection;

use crate::{
    database::repository,
    models::{ai_model::AiModel, chat_event::ChatEvent, error_model::ChatError},
    ollama::{get_tags, pull_model, show_model},
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
    pool: &std::sync::Mutex<Connection>,
    name: &str,
    on_event: impl Fn(ChatEvent),
) -> Result<(), ChatError> {
    pull_model(name, |progress| {
        on_event(ChatEvent::ModelDownloadProgress {
            name: name.to_string(),
            status: progress.status,
            completed: progress.completed,
            total: progress.total,
        });
    })
    .await?;

    sync_models(pool).await?;
    on_event(ChatEvent::ModelsChanged);
    on_event(ChatEvent::ModelDownloadFinished);

    Ok(())
}

pub fn active_model(pool: &std::sync::Mutex<Connection>) -> Result<Option<AiModel>, ChatError> {
    Ok(repository::get_models(pool)?
        .into_iter()
        .find(|model| model.active))
}

pub fn set_active_model(pool: &std::sync::Mutex<Connection>, name: &str) -> Result<(), ChatError> {
    repository::set_active_model(pool, name)
}
