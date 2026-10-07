use reqwest::Response;

use crate::models::{chat_model::{OllamaGenerateResponse, OllamaRequest, OllamaRequestGenerate}, error_model::ChatError};


pub async fn post_chat(payload: OllamaRequest) -> Result<Response, ChatError> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    return Ok(response);
}

pub async fn post_generate(payload: OllamaRequestGenerate) -> Result<String, ChatError> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let data: OllamaGenerateResponse = response
        .json()
        .await?;

    return Ok(data.response);
}