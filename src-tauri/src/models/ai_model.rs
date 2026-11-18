#[derive(serde::Serialize)]
pub struct AiModel {
    pub(crate) name: String,
    pub(crate) family: String,
    pub(crate) parameter_size: String,
    pub(crate) quantization_level: String,
    pub(crate) size_bytes: i64,
    pub(crate) modified_at: String,
    pub(crate) supports_vision: bool,
    pub(crate) active: bool,
}

#[derive(serde::Deserialize)]
pub struct OllamaTagsResponse {
    pub(crate) models: Vec<OllamaTag>,
}

#[derive(serde::Deserialize)]
pub struct OllamaTag {
    pub(crate) name: String,
    pub(crate) modified_at: String,
    pub(crate) size: i64,
    pub(crate) details: OllamaModelDetails,
}

#[derive(serde::Deserialize)]
pub struct OllamaModelDetails {
    #[serde(default)]
    pub(crate) family: String,
    #[serde(default)]
    pub(crate) parameter_size: String,
    #[serde(default)]
    pub(crate) quantization_level: String,
}

#[derive(serde::Serialize)]
pub struct OllamaShowRequest {
    pub(crate) model: String,
}

#[derive(serde::Deserialize)]
pub struct OllamaShowResponse {
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct OllamaPullRequest {
    pub(crate) model: String,
    pub(crate) stream: bool,
}

#[derive(serde::Deserialize)]
pub struct OllamaPullResponse {
    #[serde(default)]
    pub(crate) error: Option<String>,
}
