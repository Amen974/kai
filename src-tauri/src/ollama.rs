use futures_util::StreamExt;
use reqwest::Response;
use tokio_util::sync::CancellationToken;

use crate::models::{ai_model::{OllamaDeleteRequest, OllamaPullProgress, OllamaPullRequest, OllamaShowRequest, OllamaShowResponse, OllamaTagsResponse}, chat_model::{OllamaGenerateResponse, OllamaRequest, OllamaRequestGenerate}, error_model::ChatError};


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

pub async fn get_tags() -> Result<OllamaTagsResponse, ChatError> {
    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:11434/api/tags")
        .send()
        .await?
        .error_for_status()?;

    let data: OllamaTagsResponse = response.json().await?;

    return Ok(data);
}

pub async fn show_model(name: &str) -> Result<OllamaShowResponse, ChatError> {
    let client = reqwest::Client::new();

    let payload = OllamaShowRequest {
        model: name.to_string(),
    };

    let response = client
        .post("http://localhost:11434/api/show")
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let data: OllamaShowResponse = response.json().await?;

    return Ok(data);
}

fn report_pull_line<F>(line: &str, on_progress: &mut F) -> Result<bool, ChatError>
where
    F: FnMut(OllamaPullProgress),
{
    let line = line.trim();
    if line.is_empty() {
        return Ok(false);
    }

    let mut progress: OllamaPullProgress = serde_json::from_str(line)?;
    if let Some(error) = progress.error.take() {
        return Err(ChatError::ModelDownloadFailed(error));
    }

    let succeeded = progress.status == "success";
    on_progress(progress);

    Ok(succeeded)
}

pub async fn pull_model<F>(
    name: &str,
    cancel: &CancellationToken,
    mut on_progress: F,
) -> Result<bool, ChatError>
where
    F: FnMut(OllamaPullProgress),
{
    let client = reqwest::Client::new();

    let payload = OllamaPullRequest {
        model: name.to_string(),
    };

    let response = client
        .post("http://localhost:11434/api/pull")
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let data: OllamaPullProgress = response.json().await?;
        return Err(ChatError::ModelDownloadFailed(
            data.error.unwrap_or_else(|| format!("HTTP {status}")),
        ));
    }

    let mut buffer: Vec<u8> = Vec::new();
    let mut stream = Box::pin(response.bytes_stream());
    let mut succeeded = false;

    loop {
        let chunk = tokio::select! {
            chunk = stream.next() => chunk,
            _ = cancel.cancelled() => return Ok(false),
        };

        let Some(chunk) = chunk else {
            break;
        };

        buffer.extend_from_slice(&chunk?);

        while let Some(end) = buffer.iter().position(|byte| *byte == b'\n') {
            let line = String::from_utf8(buffer.drain(..=end).collect::<Vec<u8>>())?;
            succeeded |= report_pull_line(&line, &mut on_progress)?;
        }
    }

    let tail = String::from_utf8(buffer)?;
    succeeded |= report_pull_line(&tail, &mut on_progress)?;

    if !succeeded {
        return Err(ChatError::ModelDownloadFailed(
            "connection closed before the download finished".to_string(),
        ));
    }

    Ok(true)
}

pub async fn delete_model(name: &str) -> Result<(), ChatError> {
    let client = reqwest::Client::new();

    let payload = OllamaDeleteRequest {
        model: name.to_string(),
    };

    client
        .delete("http://localhost:11434/api/delete")
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
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