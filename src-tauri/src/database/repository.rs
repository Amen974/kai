use rusqlite::Connection;

use crate::models::{
    ai_model::AiModel,
    attachment_model::{Attachment, AttachmentKind},
    chat_model::{ChatMessage, GetHistory, MessageVariant, Role},
    error_model::ChatError,
};

pub fn create_history(pool: &std::sync::Mutex<Connection>) -> Result<u32, ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let id: u32 = conn.query_row(
        "INSERT INTO histories (title, created_at, updated_at) VALUES (?1, ?2, ?3) RETURNING id",
        (&now, &now, &now),
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn touch_history(pool: &std::sync::Mutex<Connection>, id: u32) -> Result<(), ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute(
        "UPDATE histories SET updated_at = ?1 WHERE id = ?2",
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
    conn.execute("UPDATE histories SET title = ?1 WHERE id = ?2", (title, id))?;
    Ok(())
}

pub fn delete_history(id: u32, pool: &std::sync::Mutex<Connection>) -> Result<(), ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    conn.execute("DELETE FROM histories WHERE id = ?1", [id])?;
    Ok(())
}

pub fn get_history(pool: &std::sync::Mutex<Connection>) -> Result<Vec<GetHistory>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare("SELECT id, title FROM histories ORDER BY updated_at DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(GetHistory {
            id: row.get(0)?,
            title: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn search_history(
    pool: &std::sync::Mutex<Connection>,
    query: &str,
) -> Result<Vec<GetHistory>, ChatError> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| t.chars().filter(|c| !c.is_control() && *c != '"').collect::<String>())
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{t}\"*"))
        .collect();

    if tokens.is_empty() {
        return get_history(pool);
    }

    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare(
        "SELECT h.id, h.title
         FROM histories h
         JOIN (
             SELECT history_id, MIN(rank) AS min_rank
             FROM history_fts5
             WHERE history_fts5 MATCH ?1
             GROUP BY history_id
         ) matched ON h.id = matched.history_id
         ORDER BY matched.min_rank ASC, h.updated_at DESC",
    )?;
    let rows = stmt.query_map([tokens.join(" ")], |row| {
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
        "UPDATE histories SET active_branch_id = ?1 WHERE id = ?2",
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
        "SELECT active_branch_id FROM histories WHERE id = ?1",
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
    conn.execute(
        "UPDATE branches
         SET head_message_id = (
             SELECT id FROM messages WHERE branch_id = ?1 ORDER BY id DESC LIMIT 1
         )
         WHERE id = ?1",
        [branch_id],
    )?;
    Ok(())
}

pub fn attachment_paths_for_history(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
) -> Result<Vec<String>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare(
        "SELECT a.path
         FROM attachments a
         JOIN messages m ON m.id = a.message_id
         WHERE m.history_id = ?1",
    )?;

    let paths = stmt
        .query_map([history_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(paths)
}

pub fn insert_attachments(
    pool: &std::sync::Mutex<Connection>,
    message_id: u32,
    attachments: &[Attachment],
) -> Result<Vec<Attachment>, ChatError> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stored = Vec::with_capacity(attachments.len());

    for attachment in attachments {
        let id: u32 = conn.query_row(
            "INSERT INTO attachments (message_id, name, kind, mime_type, path, size_bytes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING id",
            (
                message_id,
                &attachment.name,
                attachment.kind.as_str(),
                &attachment.mime_type,
                &attachment.path,
                attachment.size_bytes,
                &now,
            ),
            |row| row.get(0),
        )?;

        stored.push(Attachment {
            id: Some(id),
            ..attachment.clone()
        });
    }

    Ok(stored)
}

pub fn load_branch(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
) -> Result<Vec<ChatMessage>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;

    let mut stmt = conn.prepare(
        "WITH RECURSIVE branch_tree(id, parent_message_id, role, content, thinking) AS (

            SELECT m.id, m.parent_message_id, m.role, m.content, m.thinking
            FROM messages m
            JOIN branches b ON b.head_message_id = m.id
            WHERE b.id = ?1
            UNION ALL

            SELECT m.id, m.parent_message_id, m.role, m.content, m.thinking
            FROM messages m
            JOIN branch_tree t ON m.id = t.parent_message_id
        )
        SELECT id, parent_message_id, role, content, thinking FROM branch_tree ORDER BY id ASC",
    )?;

    let chat_messages = stmt
        .query_map([branch_id], |row| {
            let role_str: String = row.get(2)?;
            let role = Role::from_str(&role_str).ok_or(rusqlite::Error::InvalidQuery)?;
            Ok(ChatMessage {
                id: Some(row.get(0)?),
                parent_message_id: row.get(1)?,
                role,
                content: row.get(3)?,
                thinking: row.get(4)?,
                attachments: Vec::new(),
                images: None,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    load_attachments(&conn, chat_messages)
}

fn load_attachments(
    conn: &Connection,
    mut messages: Vec<ChatMessage>,
) -> Result<Vec<ChatMessage>, ChatError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, kind, mime_type, path, size_bytes
         FROM attachments
         WHERE message_id = ?1
         ORDER BY id ASC",
    )?;

    for message in &mut messages {
        let Some(message_id) = message.id else {
            continue;
        };

        message.attachments = stmt
            .query_map([message_id], |row| {
                let kind_str: String = row.get(2)?;
                let kind =
                    AttachmentKind::from_str(&kind_str).ok_or(rusqlite::Error::InvalidQuery)?;
                Ok(Attachment {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    kind,
                    mime_type: row.get(3)?,
                    path: row.get(4)?,
                    size_bytes: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(messages)
}

pub fn get_variants(
    pool: &std::sync::Mutex<Connection>,
    history_id: u32,
    parent_message_id: Option<u32>,
) -> Result<Vec<MessageVariant>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare(
        "SELECT id, branch_id
         FROM messages
         WHERE history_id = ?1
           AND ((parent_message_id IS NULL AND ?2 IS NULL) OR parent_message_id = ?2)
         ORDER BY id ASC",
    )?;

    let rows = stmt.query_map(rusqlite::params![history_id, parent_message_id], |row| {
        Ok(MessageVariant {
            id: row.get(0)?,
            branch_id: row.get(1)?,
        })
    })?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_nth_message_id(
    pool: &std::sync::Mutex<Connection>,
    branch_id: u32,
    n: usize,
) -> Result<Option<u32>, ChatError> {
    Ok(load_branch(pool, branch_id)?.get(n).and_then(|message| message.id))
}

pub fn get_models(pool: &std::sync::Mutex<Connection>) -> Result<Vec<AiModel>, ChatError> {
    let conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let mut stmt = conn.prepare(
        "SELECT name, family, parameter_size, quantization_level, size_bytes, modified_at, supports_vision, active
         FROM models ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(AiModel {
            name: row.get(0)?,
            family: row.get(1)?,
            parameter_size: row.get(2)?,
            quantization_level: row.get(3)?,
            size_bytes: row.get(4)?,
            modified_at: row.get(5)?,
            supports_vision: row.get(6)?,
            active: row.get(7)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn replace_models(
    pool: &std::sync::Mutex<Connection>,
    models: &[AiModel],
) -> Result<(), ChatError> {
    let mut conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM models", [])?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO models (name, family, parameter_size, quantization_level, size_bytes, modified_at, supports_vision, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        for model in models {
            stmt.execute(rusqlite::params![
                model.name,
                model.family,
                model.parameter_size,
                model.quantization_level,
                model.size_bytes,
                model.modified_at,
                model.supports_vision,
                model.active,
            ])?;
        }
    }

    tx.commit()?;
    Ok(())
}

pub fn set_active_model(
    pool: &std::sync::Mutex<Connection>,
    name: &str,
) -> Result<(), ChatError> {
    let mut conn = pool.lock().map_err(|_| ChatError::LockPoisoned)?;
    let tx = conn.transaction()?;

    tx.execute("UPDATE models SET active = 0 WHERE active = 1", [])?;

    if tx.execute("UPDATE models SET active = 1 WHERE name = ?1", [name])? == 0 {
        return Err(ChatError::ModelNotFound(name.to_string()));
    }

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn in_memory_pool() -> std::sync::Mutex<Connection> {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .unwrap();
        crate::database::init::migrate(&mut connection).unwrap();
        std::sync::Mutex::new(connection)
    }

    fn history_with_branch(pool: &std::sync::Mutex<Connection>) -> (u32, u32) {
        let history_id = create_history(pool).unwrap();
        let branch_id = create_branch(pool, history_id, None, None).unwrap();
        set_active_branch(pool, branch_id, history_id).unwrap();
        (history_id, branch_id)
    }

    fn append(
        pool: &std::sync::Mutex<Connection>,
        history_id: u32,
        branch_id: u32,
        parent_message_id: Option<u32>,
        role: Role,
        content: &str,
    ) -> u32 {
        let id = insert_message(
            pool,
            history_id,
            branch_id,
            parent_message_id,
            role.as_str(),
            content,
            None,
        )
        .unwrap();
        update_branch_head(pool, branch_id, id).unwrap();
        id
    }

    fn model(name: &str, active: bool) -> AiModel {
        AiModel {
            name: name.to_string(),
            family: "qwen3".to_string(),
            parameter_size: "4B".to_string(),
            quantization_level: "Q4_K_M".to_string(),
            size_bytes: 2_500_000_000,
            modified_at: "2025-01-01T00:00:00Z".to_string(),
            supports_vision: false,
            active,
        }
    }

    fn set_updated_at(pool: &std::sync::Mutex<Connection>, id: u32, timestamp: &str) {
        pool.lock()
            .unwrap()
            .execute(
                "UPDATE histories SET updated_at = ?1 WHERE id = ?2",
                (timestamp, id),
            )
            .unwrap();
    }

    #[test]
    fn create_history_inserts_a_row_that_get_history_lists() {
        let pool = in_memory_pool();

        let id = create_history(&pool).unwrap();

        let rows = get_history(&pool).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, id);
        assert!(!rows[0].title.is_empty());
    }

    #[test]
    fn get_history_orders_by_updated_at_descending() {
        let pool = in_memory_pool();
        let first = create_history(&pool).unwrap();
        let second = create_history(&pool).unwrap();
        set_updated_at(&pool, first, "2021-01-01T00:00:00Z");
        set_updated_at(&pool, second, "2020-01-01T00:00:00Z");

        let ids: Vec<u32> = get_history(&pool).unwrap().iter().map(|h| h.id).collect();

        assert_eq!(ids, vec![first, second]);
    }

    #[test]
    fn touch_history_moves_a_history_to_the_top() {
        let pool = in_memory_pool();
        let first = create_history(&pool).unwrap();
        let second = create_history(&pool).unwrap();
        set_updated_at(&pool, first, "2021-01-01T00:00:00Z");
        set_updated_at(&pool, second, "2020-01-01T00:00:00Z");

        touch_history(&pool, second).unwrap();

        assert_eq!(get_history(&pool).unwrap()[0].id, second);
    }

    #[test]
    fn update_title_overwrites_the_title() {
        let pool = in_memory_pool();
        let id = create_history(&pool).unwrap();

        update_title(&pool, &"Renamed chat".to_string(), &id).unwrap();

        assert_eq!(get_history(&pool).unwrap()[0].title, "Renamed chat");
    }

    #[test]
    fn set_active_branch_persists_the_selection() {
        let pool = in_memory_pool();
        let history_id = create_history(&pool).unwrap();
        let first = create_branch(&pool, history_id, None, None).unwrap();
        let second = create_branch(&pool, history_id, Some(first), None).unwrap();

        set_active_branch(&pool, second, history_id).unwrap();

        assert_eq!(get_active_branch(&pool, history_id).unwrap(), second);
    }

    #[test]
    fn create_branch_rejects_a_missing_fork_message() {
        let pool = in_memory_pool();
        let history_id = create_history(&pool).unwrap();

        let result = create_branch(&pool, history_id, None, Some(99));

        assert!(matches!(result, Err(ChatError::Db(_))));
    }

    #[test]
    fn get_head_message_id_is_none_until_update_branch_head_sets_it() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        assert_eq!(get_head_message_id(&pool, branch_id).unwrap(), None);

        let id = append(&pool, history_id, branch_id, None, Role::User, "hi");

        assert_eq!(get_head_message_id(&pool, branch_id).unwrap(), Some(id));
    }

    #[test]
    fn load_branch_returns_the_chain_from_the_head_back_to_the_root() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        let user_id = insert_message(
            &pool,
            history_id,
            branch_id,
            None,
            Role::User.as_str(),
            "hello",
            None,
        )
        .unwrap();
        let assistant_id = insert_message(
            &pool,
            history_id,
            branch_id,
            Some(user_id),
            Role::Assistant.as_str(),
            "hi there",
            Some("thinking"),
        )
        .unwrap();
        update_branch_head(&pool, branch_id, assistant_id).unwrap();

        let messages = load_branch(&pool, branch_id).unwrap();

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].id, Some(user_id));
        assert_eq!(messages[0].parent_message_id, None);
        assert_eq!(messages[0].content, "hello");
        assert!(matches!(messages[0].role, Role::User));
        assert_eq!(messages[1].id, Some(assistant_id));
        assert_eq!(messages[1].parent_message_id, Some(user_id));
        assert_eq!(messages[1].thinking.as_deref(), Some("thinking"));
        assert!(matches!(messages[1].role, Role::Assistant));
    }

    #[test]
    fn load_branch_returns_the_attachments_stored_for_each_message() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        let user_id = append(&pool, history_id, branch_id, None, Role::User, "look");
        let attachment = Attachment {
            id: None,
            name: "notes.txt".to_string(),
            kind: AttachmentKind::Text,
            mime_type: "text/plain".to_string(),
            path: "/tmp/notes.txt".to_string(),
            size_bytes: 12,
        };

        let stored = insert_attachments(&pool, user_id, &[attachment]).unwrap();
        let persisted = load_branch(&pool, branch_id).unwrap();

        assert_eq!(stored.len(), 1);
        assert!(stored[0].id.is_some());
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].attachments.len(), 1);
        assert_eq!(persisted[0].attachments[0].id, stored[0].id);
        assert_eq!(persisted[0].attachments[0].name, "notes.txt");
        assert_eq!(persisted[0].attachments[0].kind, AttachmentKind::Text);
        assert_eq!(persisted[0].attachments[0].mime_type, "text/plain");
        assert_eq!(persisted[0].attachments[0].size_bytes, 12);
    }

    #[test]
    fn load_branch_returns_nothing_when_the_head_is_unset() {
        let pool = in_memory_pool();
        let (_, branch_id) = history_with_branch(&pool);

        assert!(load_branch(&pool, branch_id).unwrap().is_empty());
    }

    #[test]
    fn load_branch_ignores_messages_outside_the_chain() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        append(&pool, history_id, branch_id, None, Role::User, "kept");
        let sibling = create_branch(&pool, history_id, Some(branch_id), None).unwrap();
        append(&pool, history_id, sibling, None, Role::User, "other");

        let messages = load_branch(&pool, branch_id).unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "kept");
    }

    #[test]
    fn delete_last_message_moves_the_head_back_to_the_previous_message() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        let first = append(&pool, history_id, branch_id, None, Role::User, "one");
        append(&pool, history_id, branch_id, Some(first), Role::Assistant, "two");

        delete_last_message(&pool, history_id, branch_id).unwrap();

        let messages = load_branch(&pool, branch_id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, Some(first));
        assert_eq!(get_head_message_id(&pool, branch_id).unwrap(), Some(first));
    }

    #[test]
    fn delete_last_message_on_an_empty_branch_leaves_the_head_unset() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);

        delete_last_message(&pool, history_id, branch_id).unwrap();

        assert_eq!(get_head_message_id(&pool, branch_id).unwrap(), None);
    }

    #[test]
    fn get_variants_lists_root_messages_across_branches() {
        let pool = in_memory_pool();
        let history_id = create_history(&pool).unwrap();
        let first_branch = create_branch(&pool, history_id, None, None).unwrap();
        let second_branch = create_branch(&pool, history_id, Some(first_branch), None).unwrap();
        let first = append(&pool, history_id, first_branch, None, Role::User, "a");
        let second = append(&pool, history_id, second_branch, None, Role::User, "b");

        let variants = get_variants(&pool, history_id, None).unwrap();

        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].id, first);
        assert_eq!(variants[1].id, second);
        assert_eq!(variants[1].branch_id, second_branch);
    }

    #[test]
    fn get_variants_returns_the_children_of_a_message() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        let root = append(&pool, history_id, branch_id, None, Role::User, "root");
        let other_branch = create_branch(&pool, history_id, Some(branch_id), Some(root)).unwrap();
        let first = append(&pool, history_id, branch_id, Some(root), Role::Assistant, "a");
        let second = append(&pool, history_id, other_branch, Some(root), Role::Assistant, "b");

        let variants = get_variants(&pool, history_id, Some(root)).unwrap();

        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].id, first);
        assert_eq!(variants[1].id, second);
        assert!(get_variants(&pool, history_id, Some(second)).unwrap().is_empty());
    }

    #[test]
    fn get_nth_message_id_indexes_the_branch_chain() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        let first = append(&pool, history_id, branch_id, None, Role::User, "one");
        let second = append(&pool, history_id, branch_id, Some(first), Role::Assistant, "two");

        assert_eq!(get_nth_message_id(&pool, branch_id, 0).unwrap(), Some(first));
        assert_eq!(get_nth_message_id(&pool, branch_id, 1).unwrap(), Some(second));
        assert_eq!(get_nth_message_id(&pool, branch_id, 2).unwrap(), None);
    }

    #[test]
    fn replace_models_stores_the_catalog_in_name_order() {
        let pool = in_memory_pool();
        let mut vision = model("llama3:8b", false);
        vision.supports_vision = true;

        replace_models(&pool, &[vision, model("qwen3:4b", true)]).unwrap();

        let models = get_models(&pool).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "llama3:8b");
        assert!(models[0].supports_vision);
        assert_eq!(models[1].name, "qwen3:4b");
        assert_eq!(models[1].family, "qwen3");
        assert_eq!(models[1].size_bytes, 2_500_000_000);
        assert!(models[1].active);
        assert!(!models[0].active);
    }

    #[test]
    fn replace_models_removes_the_previous_rows() {
        let pool = in_memory_pool();
        replace_models(&pool, &[model("qwen3:4b", true), model("llama3:8b", false)]).unwrap();

        replace_models(&pool, &[model("llama3:8b", true)]).unwrap();

        let models = get_models(&pool).unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "llama3:8b");
    }

    #[test]
    fn replace_models_rejects_two_active_models() {
        let pool = in_memory_pool();

        let result = replace_models(&pool, &[model("qwen3:4b", true), model("llama3:8b", true)]);

        assert!(matches!(result, Err(ChatError::Db(_))));
        assert!(get_models(&pool).unwrap().is_empty());
    }

    #[test]
    fn search_history_matches_titles_and_user_messages_but_not_assistant_messages() {
        let pool = in_memory_pool();
        let titled = create_history(&pool).unwrap();
        update_title(&pool, &"Async Runtime".to_string(), &titled).unwrap();
        let (history_id, branch_id) = history_with_branch(&pool);
        append(&pool, history_id, branch_id, None, Role::User, "ferret facts");
        append(&pool, history_id, branch_id, None, Role::Assistant, "zebra trivia");

        assert_eq!(search_history(&pool, "async").unwrap()[0].id, titled);
        assert_eq!(search_history(&pool, "fer").unwrap()[0].id, history_id);
        assert!(search_history(&pool, "zebra").unwrap().is_empty());
    }

    #[test]
    fn search_history_requires_every_token_to_match() {
        let pool = in_memory_pool();
        let (first_history, first_branch) = history_with_branch(&pool);
        let (second_history, second_branch) = history_with_branch(&pool);
        append(&pool, first_history, first_branch, None, Role::User, "rust async runtime");
        append(&pool, second_history, second_branch, None, Role::User, "rust ownership rules");

        let results = search_history(&pool, "rust async").unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, first_history);
        assert_eq!(search_history(&pool, "rust").unwrap().len(), 2);
    }

    #[test]
    fn search_history_strips_quotes_from_terms() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = history_with_branch(&pool);
        append(&pool, history_id, branch_id, None, Role::User, "quoted words");

        assert_eq!(search_history(&pool, "\"quoted\"").unwrap().len(), 1);
    }

    #[test]
    fn search_history_falls_back_to_get_history_for_blank_queries() {
        let pool = in_memory_pool();
        create_history(&pool).unwrap();
        create_history(&pool).unwrap();

        assert_eq!(search_history(&pool, "   ").unwrap().len(), 2);
    }

    #[test]
    fn search_history_drops_the_entries_of_a_deleted_history() {
        let pool = in_memory_pool();
        let id = create_history(&pool).unwrap();
        update_title(&pool, &"Deleted topic".to_string(), &id).unwrap();

        delete_history(id, &pool).unwrap();

        assert!(search_history(&pool, "deleted").unwrap().is_empty());
    }
}


