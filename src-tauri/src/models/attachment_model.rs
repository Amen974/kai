#[derive(serde::Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentKind {
    Image,
    Text,
}

impl AttachmentKind {
    pub fn as_str(&self) -> &str {
        match self {
            AttachmentKind::Image => "image",
            AttachmentKind::Text => "text",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "image" => Some(AttachmentKind::Image),
            "text" => Some(AttachmentKind::Text),
            _ => None,
        }
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct Attachment {
    pub(crate) id: Option<u32>,
    pub(crate) name: String,
    pub(crate) kind: AttachmentKind,
    pub(crate) mime_type: String,
    pub(crate) path: String,
    pub(crate) size_bytes: i64,
}
