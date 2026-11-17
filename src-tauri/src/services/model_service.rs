use rusqlite::Connection;

use crate::{
    database::repository,
    models::{
        ai_model::{AiModel, OllamaTag},
        error_model::ChatError,
    },
    ollama::{get_tags, show_model},
};

fn to_model(tag: OllamaTag, supports_vision: bool) -> AiModel {
    AiModel {
        name: tag.name,
        family: tag.details.family,
        parameter_size: tag.details.parameter_size,
        quantization_level: tag.details.quantization_level,
        size_bytes: tag.size,
        modified_at: tag.modified_at,
        supports_vision,
        active: false,
    }
}

async fn has_vision(name: &str) -> Result<bool, ChatError> {
    let capabilities = show_model(name).await?.capabilities;
    Ok(capabilities.iter().any(|capability| capability == "vision"))
}

pub async fn sync_models(
    pool: &std::sync::Mutex<Connection>,
) -> Result<Vec<AiModel>, ChatError> {
    let tags = get_tags().await?;

    let mut models = Vec::with_capacity(tags.models.len());
    for tag in tags.models {
        let supports_vision = has_vision(&tag.name).await?;
        models.push(to_model(tag, supports_vision));
    }

    let active = repository::get_active_model_name(pool)?
        .filter(|name| models.iter().any(|model| &model.name == name))
        .or_else(|| models.first().map(|model| model.name.clone()));

    repository::replace_models(pool, &models, active.as_deref())?;

    repository::get_models(pool)
}

pub async fn sync_if_empty(pool: &std::sync::Mutex<Connection>) -> Result<(), ChatError> {
    if !repository::get_models(pool)?.is_empty() {
        return Ok(());
    }

    sync_models(pool).await?;

    Ok(())
}

pub fn list_models(pool: &std::sync::Mutex<Connection>) -> Result<Vec<AiModel>, ChatError> {
    get_models(pool)
}

pub fn get_models(pool: &std::sync::Mutex<Connection>) -> Result<Vec<AiModel>, ChatError> {
    repository::get_models(pool)
}

pub fn active_model(pool: &std::sync::Mutex<Connection>) -> Result<Option<AiModel>, ChatError> {
    Ok(repository::get_models(pool)?
        .into_iter()
        .find(|model| model.active))
}

pub fn set_active_model(
    pool: &std::sync::Mutex<Connection>,
    name: &str,
) -> Result<Vec<AiModel>, ChatError> {
    repository::set_active_model(pool, name)?;
    repository::get_models(pool)
}
