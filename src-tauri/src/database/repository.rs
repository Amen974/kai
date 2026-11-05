use rusqlite::{Connection, OptionalExtension};

use crate::models::{
    chat_model::{ChatMessage, GetHistory, Role},
    error_model::ChatError,
};

pub fn create_history(pool: &std::sync::Mutex<Connection>) -> Result<u32, ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let id: u32 = conn.query_row(
        "INSERT INTO historys (title, created_at, updated_at) VALUES (?1, ?2, ?3) RETURNING id",
        (&now, &now, &now),
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn touch_history(pool: &std::sync::Mutex<Connection>, id: u32) -> Result<(), ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "UPDATE historys SET updated_at = ?1 WHERE id = ?2",
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
    conn.execute("UPDATE historys SET title = ?1 WHERE id = ?2", (title, id))?;
    Ok(())
}

pub fn delete_history(id: u32, pool: &std::sync::Mutex<Connection>) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute("DELETE FROM historys WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_history(pool: &std::sync::Mutex<Connection>) -> Result<Vec<GetHistory>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare("SELECT id, title FROM historys ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(GetHistory {
            id: row.get(0)?,
            title: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_branch(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    parent_branch_id: Option<u32>,
    fork_message_id: Option<u32>,
) -> Result<u32, ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;

    let branch_id: u32 = conn.query_row(
        "INSERT INTO branches (history_id, parent_branch_id, fork_message_id, head_message_id, created_at)
         VALUES (?1, ?2, ?3, NULL, ?4) RETURNING id",
        (history_id, parent_branch_id, fork_message_id, &now),
        |row| row.get(0),
    )?;

    Ok(branch_id)
}

pub fn set_active_branch(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
    history_id: u32,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "UPDATE historys SET active_branch_id = ?1 WHERE id = ?2",
        [branch_id, history_id],
    )?;
    Ok(())
}

pub fn get_active_branch(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
) -> Result<u32, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let active_branch_id: u32 = conn.query_row(
        "SELECT active_branch_id FROM historys WHERE id = ?1",
        [history_id],
        |row| row.get(0),
    )?;
    Ok(active_branch_id)
}

pub fn update_branch_head(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
    head_message_id: u32,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "UPDATE branches SET head_message_id = ?1 WHERE id = ?2",
        [head_message_id, branch_id],
    )?;
    Ok(())
}

pub fn get_head_message_id(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
) -> Result<Option<u32>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let head_message_id: Option<u32> = conn.query_row(
        "SELECT head_message_id FROM branches WHERE id = ?1",
        [branch_id],
        |row| row.get(0),
    )?;
    Ok(head_message_id)
}

pub fn insert_message(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    branch_id: u32,
    parent_message_id: Option<u32>,
    role: &str,
    content: &str,
    thinking: Option<&str>,
) -> Result<u32, ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let new_id: u32 = conn.query_row(
        "INSERT INTO messages (history_id, branch_id, parent_message_id, role, content, thinking, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING id",
        (history_id, branch_id, parent_message_id, role, content, thinking, &now),
        |row| row.get(0),
    )?;
    Ok(new_id)
}

pub fn delete_last_message(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    branch_id: u32,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "DELETE FROM messages
         WHERE id = (
             SELECT id FROM messages
             WHERE history_id = ?1 AND branch_id = ?2
             ORDER BY id DESC
             LIMIT 1
         )",
        [history_id, branch_id],
    )?;
    Ok(())
}

pub fn load_branch(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
) -> Result<Vec<ChatMessage>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;

    let mut stmt = conn.prepare(
        "WITH RECURSIVE branch_tree(id, parent_message_id, role, content, thinking) AS (
            -- Start from the head message of the branch
            SELECT m.id, m.parent_message_id, m.role, m.content, m.thinking
            FROM messages m
            JOIN branches b ON b.head_message_id = m.id
            WHERE b.id = ?1
            UNION ALL
            -- Walk backwards through parent links
            SELECT m.id, m.parent_message_id, m.role, m.content, m.thinking
            FROM messages m
            JOIN branch_tree t ON m.id = t.parent_message_id
        )
        SELECT id, role, content, thinking FROM branch_tree ORDER BY id ASC",
    )?;

    let chat_messages = stmt
        .query_map([branch_id], |row| {
            let role_str: String = row.get(1)?;
            let role = Role::from_str(&role_str).ok_or(rusqlite::Error::InvalidQuery)?;
            Ok(ChatMessage {
                role,
                content: row.get(2)?,
                thinking: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(chat_messages)
}

pub fn get_nth_message_id(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
    n: usize,
) -> Result<Option<u32>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare(
        "WITH RECURSIVE branch_tree(id, parent_message_id, depth) AS (
            SELECT m.id, m.parent_message_id, 0
            FROM messages m
            JOIN branches b ON b.head_message_id = m.id
            WHERE b.id = ?1
            UNION ALL
            SELECT m.id, m.parent_message_id, t.depth + 1
            FROM messages m
            JOIN branch_tree t ON m.id = t.parent_message_id
        )
        SELECT id FROM branch_tree ORDER BY id ASC LIMIT 1 OFFSET ?2",
    )?;
    let result: Option<u32> = stmt
        .query_row([branch_id as i64, n as i64], |row| row.get(0))
        .optional()?;
    Ok(result)
}

#[allow(dead_code)]
pub fn truncate_messages(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    branch_id: u32,
    keep_count: usize,
) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "DELETE FROM messages
         WHERE history_id = ?1 AND branch_id = ?2
           AND id NOT IN (
               SELECT id FROM messages
               WHERE history_id = ?1 AND branch_id = ?2
               ORDER BY id ASC
               LIMIT ?3
           )",
        rusqlite::params![history_id, branch_id, keep_count as i64],
    )?;
    Ok(())
}