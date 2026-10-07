use rusqlite::Connection;

use crate::models::{
    chat_model::{ChatMessage, GetHistory, Role},
    error_model::ChatError,
};

pub fn create_history(pool: &std::sync::Mutex<Connection>) -> Result<u32, ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let id: u32 = conn.query_row(
        "INSERT INTO history (title, created_at, updated_at) VALUES (?1, ?2, ?3) RETURNING id",
        (&now, &now, &now),
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn touch_history(pool: &std::sync::Mutex<Connection>, id: u32) -> Result<(), ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "UPDATE history SET updated_at = ?1 WHERE id = ?2",
        (&now, &id),
    )?;
    Ok(())
}

pub fn update_title(
    pool: &std::sync::Mutex<Connection>,
    title: &String,
    id: &u32,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute("UPDATE history SET title = ?1 WHERE id = ?2", (title, id))?;
    Ok(())
}

pub fn delete_history(id: u32, pool: &std::sync::Mutex<Connection>) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute("DELETE FROM history WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_history(pool: &std::sync::Mutex<Connection>) -> Result<Vec<GetHistory>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare("SELECT id, title FROM history ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(GetHistory {
            id: row.get(0)?,
            title: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn insert_message(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    role: &str,
    content: &str,
) -> Result<(), ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "INSERT INTO messages (history_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
        (&history_id, role, content, &now),
    )?;
    Ok(())
}

pub fn truncate_messages(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    keep_count: usize,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "DELETE FROM messages
         WHERE history_id = ?1
           AND id NOT IN (
               SELECT id FROM messages
               WHERE history_id = ?1
               ORDER BY id ASC
               LIMIT ?2
           )",
        rusqlite::params![history_id, keep_count as i64],
    )?;
    Ok(())
}

pub fn delete_last_message(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "DELETE FROM messages
         WHERE id = (
             SELECT id FROM messages
             WHERE history_id = ?1
             ORDER BY id DESC
             LIMIT 1
         )",
        [history_id],
    )?;
    Ok(())
}

pub fn load_messages(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
) -> Result<Vec<ChatMessage>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt =
        conn.prepare("SELECT role, content FROM messages WHERE history_id = ?1 ORDER BY id")?;
    let rows = stmt.query_map([history_id], |row| {
        let role_str: String = row.get(0)?;
        let role = Role::from_str(&role_str).ok_or(rusqlite::Error::InvalidQuery)?;
        Ok(ChatMessage {
            role,
            content: row.get(1)?,
            thinking: None,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}
