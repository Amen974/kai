#[derive(serde::Serialize)]
#[derive(Clone)]
pub struct Message {
    pub(crate) role: Role,
    pub(crate) content: String,
}

#[derive(serde::Serialize)]
#[derive(Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant
}

#[derive(serde::Serialize)]
#[derive(Clone)]
pub struct OllamaRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<Message>,
    pub(crate) stream: bool,
}

#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct OllamaMessage {
    pub(crate) content: String,
}

#[derive(serde::Deserialize)]
#[derive(serde::Serialize)]
pub struct OllamaResponse {
    pub(crate) message: Option<OllamaMessage>,
    pub(crate) done: bool,
}