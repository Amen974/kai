use std::{fs, path::Path};

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::{
    models::{
        attachment_model::{Attachment, AttachmentKind},
        error_model::ChatError,
    },
    services::model_service,
    state::ChatSession,
};

const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_TEXT_BYTES: u64 = 1024 * 1024;

const ATTACHMENTS_DIR: &str = "attachments";

fn classify(path: &Path) -> Result<(AttachmentKind, &'static str), ChatError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "png" => Ok((AttachmentKind::Image, "image/png")),
        "jpg" | "jpeg" => Ok((AttachmentKind::Image, "image/jpeg")),
        "webp" => Ok((AttachmentKind::Image, "image/webp")),
        "gif" => Ok((AttachmentKind::Image, "image/gif")),
        "txt" => Ok((AttachmentKind::Text, "text/plain")),
        _ => Err(ChatError::UnsupportedAttachmentType(extension)),
    }
}

fn file_name(path: &Path) -> Result<String, ChatError> {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or(ChatError::InvalidAttachmentPath)
}

pub fn require_vision(model_name: &str, supports_vision: bool) -> Result<(), ChatError> {
    if !supports_vision {
        return Err(ChatError::AttachmentNeedsVisionModel(
            model_name.to_string(),
        ));
    }

    Ok(())
}

fn require_vision_model(pool: &std::sync::Mutex<Connection>) -> Result<(), ChatError> {
    let model = model_service::active_model(pool)?.ok_or(ChatError::NoActiveModel)?;

    require_vision(&model.name, model.supports_vision)
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let third = chunk.get(2).copied().unwrap_or_default();
        let second = chunk.get(1).copied().unwrap_or_default();
        let value = (u32::from(chunk[0]) << 16) | (u32::from(second) << 8) | u32::from(third);

        encoded.push(TABLE[(value >> 18) as usize & 63] as char);
        encoded.push(TABLE[(value >> 12) as usize & 63] as char);
        encoded.push(if chunk.len() > 1 {
            TABLE[(value >> 6) as usize & 63] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            TABLE[value as usize & 63] as char
        } else {
            '='
        });
    }

    encoded
}

pub async fn attach(
    app_data_dir: &Path,
    source: &Path,
    session: &Mutex<ChatSession>,
    pool: &std::sync::Mutex<Connection>,
) -> Result<Attachment, ChatError> {
    let (kind, mime_type) = classify(source)?;
    let name = file_name(source)?;

    let metadata = fs::metadata(source)?;
    if !metadata.is_file() {
        return Err(ChatError::InvalidAttachmentPath);
    }

    let size_bytes = metadata.len();
    let limit = match kind {
        AttachmentKind::Image => MAX_IMAGE_BYTES,
        AttachmentKind::Text => MAX_TEXT_BYTES,
    };

    if size_bytes > limit {
        return Err(ChatError::AttachmentTooLarge {
            name,
            size_bytes,
            limit,
        });
    }

    match kind {
        AttachmentKind::Image => require_vision_model(pool)?,
        AttachmentKind::Text => {
            if fs::read_to_string(source)?.trim().is_empty() {
                return Err(ChatError::EmptyAttachment(name));
            }
        }
    }

    let directory = app_data_dir.join(ATTACHMENTS_DIR);
    fs::create_dir_all(&directory)?;

    let destination = directory.join(format!("{}-{name}", chrono::Utc::now().timestamp_micros()));
    fs::copy(source, &destination)?;

    let attachment = Attachment {
        id: None,
        name,
        kind,
        mime_type: mime_type.to_string(),
        path: destination.to_string_lossy().to_string(),
        size_bytes: size_bytes as i64,
    };

    session.lock().await.add_pending_attachment(attachment.clone());

    Ok(attachment)
}

fn remove_copy(path: &str) -> Result<(), ChatError> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result.map_err(ChatError::from),
    }
}

pub fn discard<'a>(paths: impl IntoIterator<Item = &'a str>) -> Result<(), ChatError> {
    for path in paths {
        remove_copy(path)?;
    }

    Ok(())
}

pub async fn detach(session: &Mutex<ChatSession>, path: &str) -> Result<(), ChatError> {
    let attachment = session.lock().await.remove_pending_attachment(path)?;

    discard(std::iter::once(attachment.path.as_str()))
}

pub fn read_text(attachment: &Attachment) -> Result<String, ChatError> {
    Ok(fs::read_to_string(&attachment.path)?)
}

pub fn read_image_base64(attachment: &Attachment) -> Result<String, ChatError> {
    Ok(encode_base64(&fs::read(&attachment.path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn classify_accepts_the_supported_extensions_in_any_case() {
        assert_eq!(
            classify(&PathBuf::from("photo.PNG")).unwrap(),
            (AttachmentKind::Image, "image/png")
        );
        assert_eq!(
            classify(&PathBuf::from("photo.jpeg")).unwrap(),
            (AttachmentKind::Image, "image/jpeg")
        );
        assert_eq!(
            classify(&PathBuf::from("notes.Txt")).unwrap(),
            (AttachmentKind::Text, "text/plain")
        );
    }

    #[test]
    fn classify_rejects_an_unsupported_extension() {
        let result = classify(&PathBuf::from("archive.zip"));

        assert!(matches!(result, Err(ChatError::UnsupportedAttachmentType(extension)) if extension == "zip"));
    }

    #[test]
    fn require_vision_accepts_a_vision_model() {
        assert!(require_vision("llava:7b", true).is_ok());
    }

    #[test]
    fn require_vision_rejects_a_text_only_model() {
        let result = require_vision("qwen3:4b", false);

        assert!(matches!(result, Err(ChatError::AttachmentNeedsVisionModel(name)) if name == "qwen3:4b"));
    }

    #[test]
    fn encode_base64_pads_each_incomplete_group() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(b"foob"), "Zm9vYg==");
        assert_eq!(encode_base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn encode_base64_keeps_high_bytes() {
        assert_eq!(encode_base64(&[0xff, 0xd8, 0xff]), "/9j/");
    }

    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("kai-detach-{}-{name}", std::process::id()));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn queued_attachment(path: &Path) -> Attachment {
        Attachment {
            id: None,
            name: "notes.txt".to_string(),
            kind: AttachmentKind::Text,
            mime_type: "text/plain".to_string(),
            path: path.to_string_lossy().to_string(),
            size_bytes: 4,
        }
    }

    #[tokio::test]
    async fn detach_removes_the_queued_attachment_and_its_copy() {
        let path = temp_file("detach.txt", b"body");
        let queued_path = path.to_string_lossy().to_string();
        let session = Mutex::new(ChatSession::default());
        session
            .lock()
            .await
            .add_pending_attachment(queued_attachment(&path));

        detach(&session, &queued_path).await.unwrap();

        assert!(!path.exists());
        assert!(session.lock().await.pending_attachments.is_empty());
    }

    #[tokio::test]
    async fn detach_leaves_a_file_that_is_not_queued_alone() {
        let path = temp_file("unqueued.txt", b"body");
        let session = Mutex::new(ChatSession::default());

        let result = detach(&session, &path.to_string_lossy()).await;

        assert!(matches!(result, Err(ChatError::AttachmentNotFound(_))));
        assert!(path.exists());
        std::fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn detach_tolerates_a_copy_that_is_already_gone() {
        let path = temp_file("gone.txt", b"body");
        let queued_path = path.to_string_lossy().to_string();
        let session = Mutex::new(ChatSession::default());
        session
            .lock()
            .await
            .add_pending_attachment(queued_attachment(&path));
        std::fs::remove_file(&path).unwrap();

        let result = detach(&session, &queued_path).await;

        assert!(result.is_ok());
        assert!(session.lock().await.pending_attachments.is_empty());
    }
}
