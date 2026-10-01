use reqwest::Response;

use crate::models::chat_model::{OllamaGenerateResponse, OllamaRequest, OllamaRequestGenerate};


pub async fn post_chat(payload: OllamaRequest) -> Result<Response, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Ollama returned status: {}",
            response.status()
        ));
    }

    return Ok(response);
}

pub async fn post_generate(payload: OllamaRequestGenerate) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Ollama returned status: {}",
            response.status()
        ));
    }

    let data: OllamaGenerateResponse = response
        .json()
        .await
        .map_err(|e| e.to_string())?;

    return Ok(data.response);
}