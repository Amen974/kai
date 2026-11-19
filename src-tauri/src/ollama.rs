use futures_util::StreamExt;
use reqwest::Response;
use tokio_util::sync::CancellationToken;

use crate::models::{ai_model::{OllamaDeleteRequest, OllamaPullProgress, OllamaPullRequest, OllamaShowRequest, OllamaShowResponse, OllamaTagsResponse}, chat_model::{OllamaGenerateResponse, OllamaRequest, OllamaRequestGenerate}, error_model::ChatError};

pub const DEFAULT_BASE_URL: &str = "http://localhost:11434";

fn endpoint(base_url: &str, path: &str) -> String {
    format!("{}{}", base_url.trim_end_matches('/'), path)
}

pub async fn post_chat(base_url: &str, payload: OllamaRequest) -> Result<Response, ChatError> {
    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint(base_url, "/api/chat"))
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    return Ok(response);
}

pub async fn get_tags(base_url: &str) -> Result<OllamaTagsResponse, ChatError> {
    let client = reqwest::Client::new();

    let response = client
        .get(&endpoint(base_url, "/api/tags"))
        .send()
        .await?
        .error_for_status()?;

    let data: OllamaTagsResponse = response.json().await?;

    return Ok(data);
}

pub async fn show_model(base_url: &str, name: &str) -> Result<OllamaShowResponse, ChatError> {
    let client = reqwest::Client::new();

    let payload = OllamaShowRequest {
        model: name.to_string(),
    };

    let response = client
        .post(&endpoint(base_url, "/api/show"))
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
    base_url: &str,
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
        .post(&endpoint(base_url, "/api/pull"))
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

pub async fn delete_model(base_url: &str, name: &str) -> Result<(), ChatError> {
    let client = reqwest::Client::new();

    let payload = OllamaDeleteRequest {
        model: name.to_string(),
    };

    client
        .delete(&endpoint(base_url, "/api/delete"))
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn post_generate(base_url: &str, payload: OllamaRequestGenerate) -> Result<String, ChatError> {
    let client = reqwest::Client::new();

    let response = client
        .post(&endpoint(base_url, "/api/generate"))
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let data: OllamaGenerateResponse = response
        .json()
        .await?;

    return Ok(data.response);
}

#[cfg(test)]
pub(crate) mod mock {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Clone)]
    pub(crate) struct MockRoute {
        pub(crate) method: &'static str,
        pub(crate) path: &'static str,
        pub(crate) status: u16,
        pub(crate) body: &'static str,
        hold_open: bool,
    }

    impl MockRoute {
        pub(crate) fn json(method: &'static str, path: &'static str, body: &'static str) -> Self {
            Self {
                method,
                path,
                status: 200,
                body,
                hold_open: false,
            }
        }

        pub(crate) fn status(
            method: &'static str,
            path: &'static str,
            status: u16,
            body: &'static str,
        ) -> Self {
            Self {
                method,
                path,
                status,
                body,
                hold_open: false,
            }
        }

        pub(crate) fn hold_open(method: &'static str, path: &'static str, body: &'static str) -> Self {
            Self {
                method,
                path,
                status: 200,
                body,
                hold_open: true,
            }
        }
    }

    /// Loopback HTTP server that answers the configured routes and nothing else.
    pub(crate) struct MockOllama {
        pub(crate) base_url: String,
        task: tokio::task::JoinHandle<()>,
    }

    impl MockOllama {
        pub(crate) async fn start(routes: Vec<MockRoute>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind mock ollama");
            let port = listener.local_addr().expect("mock address").port();

            let task = tokio::spawn(async move {
                loop {
                    let Ok((mut socket, _)) = listener.accept().await else {
                        return;
                    };
                    let routes = routes.clone();

                    tokio::spawn(async move {
                        let mut buffer = Vec::new();
                        let mut chunk = [0u8; 1024];

                        while find_headers_end(&buffer).is_none() {
                            match socket.read(&mut chunk).await {
                                Ok(0) | Err(_) => return,
                                Ok(read) => buffer.extend_from_slice(&chunk[..read]),
                            }
                        }

                        let header_end = find_headers_end(&buffer).unwrap_or(0);
                        let headers = String::from_utf8_lossy(&buffer[..header_end]).to_lowercase();
                        let content_length: usize = headers
                            .lines()
                            .find_map(|line| line.strip_prefix("content-length:"))
                            .and_then(|value| value.trim().parse().ok())
                            .unwrap_or(0);

                        while buffer.len() < header_end + content_length {
                            match socket.read(&mut chunk).await {
                                Ok(0) | Err(_) => break,
                                Ok(read) => buffer.extend_from_slice(&chunk[..read]),
                            }
                        }

                        let request_line = String::from_utf8_lossy(&buffer[..header_end])
                            .lines()
                            .next()
                            .unwrap_or_default()
                            .to_string();
                        let mut request_parts = request_line.split_whitespace();
                        let method = request_parts.next().unwrap_or_default();
                        let path = request_parts.next().unwrap_or_default();

                        let route = routes
                            .iter()
                            .find(|route| route.method == method && route.path == path);
                        let (status, body, hold_open) = match route {
                            Some(route) => (route.status, route.body, route.hold_open),
                            None => (404, "", false),
                        };

                        let response = if hold_open {
                            format!(
                                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\nconnection: close\r\n\r\n{body}"
                            )
                        } else {
                            format!(
                                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                                body.len()
                            )
                        };
                        let _ = socket.write_all(response.as_bytes()).await;

                        if hold_open {
                            let mut scratch = [0u8; 1];
                            let _ = socket.read(&mut scratch).await;
                        }
                        let _ = socket.shutdown().await;
                    });
                }
            });

            Self {
                base_url: format!("http://127.0.0.1:{port}"),
                task,
            }
        }
    }

    impl Drop for MockOllama {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    fn find_headers_end(buffer: &[u8]) -> Option<usize> {
        buffer
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|position| position + 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ollama::mock::{MockOllama, MockRoute};

    fn chat_payload() -> OllamaRequest {
        OllamaRequest {
            model: "qwen3:4b".to_string(),
            messages: Vec::new(),
            stream: true,
        }
    }

    fn generate_payload() -> OllamaRequestGenerate {
        OllamaRequestGenerate {
            model: "qwen3:4b".to_string(),
            prompt: "Summarize".to_string(),
            stream: false,
        }
    }

    #[test]
    fn endpoint_ignores_a_trailing_slash() {
        assert_eq!(
            endpoint("http://127.0.0.1:11434/", "/api/tags"),
            "http://127.0.0.1:11434/api/tags"
        );
    }

    #[tokio::test]
    async fn get_tags_reads_the_model_list() {
        let server = MockOllama::start(vec![MockRoute::json(
            "GET",
            "/api/tags",
            r#"{"models":[{"name":"qwen3:4b","modified_at":"2026-01-02T03:04:05Z","size":2500000000,"details":{"family":"qwen3","parameter_size":"4B","quantization_level":"Q4_K_M"}}]}"#,
        )])
        .await;

        let tags = get_tags(&server.base_url).await.expect("get tags");

        assert_eq!(tags.models.len(), 1);
        assert_eq!(tags.models[0].name, "qwen3:4b");
        assert_eq!(tags.models[0].size, 2_500_000_000);
        assert_eq!(tags.models[0].modified_at, "2026-01-02T03:04:05Z");
        assert_eq!(tags.models[0].details.family, "qwen3");
        assert_eq!(tags.models[0].details.quantization_level, "Q4_K_M");
    }

    #[tokio::test]
    async fn get_tags_fails_on_an_error_status() {
        let server =
            MockOllama::start(vec![MockRoute::status("GET", "/api/tags", 500, "boom")]).await;

        let result = get_tags(&server.base_url).await;

        assert!(matches!(result, Err(ChatError::Http(_))));
    }

    #[tokio::test]
    async fn show_model_reports_the_vision_capability() {
        let server = MockOllama::start(vec![MockRoute::json(
            "POST",
            "/api/show",
            r#"{"capabilities":["completion","vision"]}"#,
        )])
        .await;

        let show = show_model(&server.base_url, "qwen3:4b")
            .await
            .expect("show model");

        assert!(show
            .capabilities
            .iter()
            .any(|capability| capability == "vision"));
    }

    #[tokio::test]
    async fn post_chat_returns_the_streaming_response() {
        let body = "{\"message\":{\"content\":\"hi\"},\"done\":true}\n";
        let server = MockOllama::start(vec![MockRoute::json("POST", "/api/chat", body)]).await;

        let response = post_chat(&server.base_url, chat_payload())
            .await
            .expect("post chat");
        let text = response.text().await.expect("chat body");

        assert_eq!(text, body);
    }

    #[tokio::test]
    async fn post_generate_returns_the_generated_text() {
        let server = MockOllama::start(vec![MockRoute::json(
            "POST",
            "/api/generate",
            r#"{"response":"Sourdough starter"}"#,
        )])
        .await;

        let title = post_generate(&server.base_url, generate_payload())
            .await
            .expect("post generate");

        assert_eq!(title, "Sourdough starter");
    }

    #[tokio::test]
    async fn pull_model_reports_progress_and_succeeds() {
        let server = MockOllama::start(vec![MockRoute::json(
            "POST",
            "/api/pull",
            "{\"status\":\"pulling manifest\",\"completed\":0,\"total\":100}\n{\"status\":\"success\"}",
        )])
        .await;
        let mut statuses = Vec::new();

        let pulled = pull_model(
            &server.base_url,
            "qwen3:4b",
            &CancellationToken::new(),
            |progress| statuses.push(progress.status),
        )
        .await
        .expect("pull model");

        assert!(pulled);
        assert_eq!(statuses, vec!["pulling manifest", "success"]);
    }

    #[tokio::test]
    async fn pull_model_returns_false_when_cancelled() {
        let server = MockOllama::start(vec![MockRoute::hold_open(
            "POST",
            "/api/pull",
            "{\"status\":\"pulling manifest\"}\n",
        )])
        .await;
        let (started, mut progress) = tokio::sync::mpsc::unbounded_channel();
        let cancel = CancellationToken::new();
        let base_url = server.base_url.clone();
        let token = cancel.clone();

        let pull = tokio::spawn(async move {
            pull_model(&base_url, "qwen3:4b", &token, move |event| {
                started.send(event.status).expect("progress channel")
            })
            .await
        });
        progress.recv().await.expect("first progress");

        cancel.cancel();
        let pulled = pull.await.expect("pull task").expect("pull model");

        assert!(!pulled);
    }

    #[tokio::test]
    async fn pull_model_fails_when_the_stream_ends_without_success() {
        let server = MockOllama::start(vec![MockRoute::json(
            "POST",
            "/api/pull",
            "{\"status\":\"pulling manifest\"}\n",
        )])
        .await;

        let result = pull_model(
            &server.base_url,
            "qwen3:4b",
            &CancellationToken::new(),
            |_| {},
        )
        .await;

        assert!(matches!(
            result,
            Err(ChatError::ModelDownloadFailed(message))
                if message == "connection closed before the download finished"
        ));
    }

    #[tokio::test]
    async fn pull_model_reports_an_error_line() {
        let server = MockOllama::start(vec![MockRoute::json(
            "POST",
            "/api/pull",
            r#"{"error":"model not found"}"#,
        )])
        .await;

        let result = pull_model(
            &server.base_url,
            "missing:1b",
            &CancellationToken::new(),
            |_| {},
        )
        .await;

        assert!(matches!(
            result,
            Err(ChatError::ModelDownloadFailed(message)) if message == "model not found"
        ));
    }

    #[tokio::test]
    async fn pull_model_reports_the_error_body_of_a_failed_request() {
        let server = MockOllama::start(vec![MockRoute::status(
            "POST",
            "/api/pull",
            404,
            r#"{"error":"model 'missing:1b' not found"}"#,
        )])
        .await;

        let result = pull_model(
            &server.base_url,
            "missing:1b",
            &CancellationToken::new(),
            |_| {},
        )
        .await;

        assert!(matches!(
            result,
            Err(ChatError::ModelDownloadFailed(message)) if message == "model 'missing:1b' not found"
        ));
    }

    #[tokio::test]
    async fn delete_model_sends_the_delete_request() {
        let server =
            MockOllama::start(vec![MockRoute::status("DELETE", "/api/delete", 200, "")]).await;

        delete_model(&server.base_url, "qwen3:4b")
            .await
            .expect("delete model");
    }
}