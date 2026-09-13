use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};

#[derive(serde::Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(serde::Deserialize)]
struct OllamaResponse {
    response: Option<String>,
    done: bool,
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    message: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let payload = OllamaRequest {
        model: "qwen3:4b".to_string(),
        prompt: message,
        stream: true,
    };

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

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].to_string();
            buffer.drain(..=pos);

            if line.trim().is_empty() {
                continue;
            }

            let data: OllamaResponse = 
                serde_json::from_str(&line).map_err(|error| error.to_string())?;

            if let Some(response) = data.response {
                app.emit("chat-chunk", response)
                    .map_err(|error| error.to_string())?;
            }

            if data.done {
                app.emit("chat-done", ())
                    .map_err(|error| error.to_string())?;

                return Ok(());
            }
        }
    }

    Ok(())
}