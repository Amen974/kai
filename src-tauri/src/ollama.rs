use reqwest::Response;

use crate::models::chat::OllamaRequest;


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