use rusqlite::Connection;

use crate::models::{
    ai_model::AiModel,
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
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(chat_messages)
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
mod tests {
    use super::*;
    use crate::database::test_db::in_memory_pool;

    fn seed_history(pool: &std::sync::Mutex<Connection>) -> (u32, u32) {
        let history_id = create_history(pool).expect("create history");
        let branch_id = create_branch(pool, history_id, None, None).expect("create branch");
        set_active_branch(pool, branch_id, history_id).expect("set active branch");
        (history_id, branch_id)
    }

    fn append(
        pool: &std::sync::Mutex<Connection>,
        history_id: u32,
        branch_id: u32,
        role: &str,
        content: &str,
    ) -> u32 {
        let parent = get_head_message_id(pool, branch_id).expect("read head");
        let id = insert_message(pool, history_id, branch_id, parent, role, content, None)
            .expect("insert message");
        update_branch_head(pool, branch_id, id).expect("update head");
        id
    }

    fn fork(
        pool: &std::sync::Mutex<Connection>,
        history_id: u32,
        parent_branch_id: u32,
        fork_message_id: u32,
    ) -> u32 {
        let branch_id = create_branch(
            pool,
            history_id,
            Some(parent_branch_id),
            Some(fork_message_id),
        )
        .expect("create fork");
        update_branch_head(pool, branch_id, fork_message_id).expect("update head");
        branch_id
    }

    fn ids(messages: &[ChatMessage]) -> Vec<Option<u32>> {
        messages.iter().map(|message| message.id).collect()
    }

    #[test]
    fn new_branch_has_no_head_and_loads_no_messages() {
        let pool = in_memory_pool();
        let (_history_id, branch_id) = seed_history(&pool);

        assert_eq!(get_head_message_id(&pool, branch_id).expect("head"), None);
        assert!(load_branch(&pool, branch_id).expect("branch").is_empty());
    }

    #[test]
    fn update_branch_head_points_the_branch_at_a_message() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let message_id = insert_message(&pool, history_id, branch_id, None, "user", "hello", None)
            .expect("insert message");

        update_branch_head(&pool, branch_id, message_id).expect("update head");

        assert_eq!(
            get_head_message_id(&pool, branch_id).expect("head"),
            Some(message_id)
        );
    }

    #[test]
    fn set_active_branch_persists_the_selection() {
        let pool = in_memory_pool();
        let (history_id, first_branch_id) = seed_history(&pool);
        let second_branch_id = create_branch(&pool, history_id, Some(first_branch_id), None)
            .expect("create branch");

        set_active_branch(&pool, second_branch_id, history_id).expect("set active branch");

        assert_eq!(
            get_active_branch(&pool, history_id).expect("active branch"),
            second_branch_id
        );
    }

    #[test]
    fn load_branch_returns_messages_in_parent_order() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let question = append(&pool, history_id, branch_id, "user", "why is the sky blue");
        let answer = append(&pool, history_id, branch_id, "assistant", "rayleigh scattering");

        let messages = load_branch(&pool, branch_id).expect("load branch");

        assert_eq!(ids(&messages), vec![Some(question), Some(answer)]);
        assert!(matches!(messages[0].role, Role::User));
        assert!(matches!(messages[1].role, Role::Assistant));
        assert_eq!(messages[0].content, "why is the sky blue");
        assert_eq!(messages[0].parent_message_id, None);
        assert_eq!(messages[1].parent_message_id, Some(question));
        assert_eq!(messages[1].thinking, None);
    }

    #[test]
    fn load_branch_walks_the_parent_chain_from_a_fork() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let question = append(&pool, history_id, branch_id, "user", "first question");
        append(&pool, history_id, branch_id, "assistant", "first answer");

        let fork_id = fork(&pool, history_id, branch_id, question);
        let edited = append(&pool, history_id, fork_id, "user", "edited question");

        let messages = load_branch(&pool, fork_id).expect("load fork");

        assert_eq!(ids(&messages), vec![Some(question), Some(edited)]);
    }

    #[test]
    fn load_branch_fails_on_an_unknown_role() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let message_id = insert_message(&pool, history_id, branch_id, None, "system", "prompt", None)
            .expect("insert message");
        update_branch_head(&pool, branch_id, message_id).expect("update head");

        let result = load_branch(&pool, branch_id);

        assert!(matches!(result, Err(ChatError::Db(_))));
    }

    #[test]
    fn get_nth_message_id_returns_the_message_at_an_index() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let question = append(&pool, history_id, branch_id, "user", "one");
        let answer = append(&pool, history_id, branch_id, "assistant", "two");

        assert_eq!(get_nth_message_id(&pool, branch_id, 0).expect("index"), Some(question));
        assert_eq!(get_nth_message_id(&pool, branch_id, 1).expect("index"), Some(answer));
        assert_eq!(get_nth_message_id(&pool, branch_id, 2).expect("index"), None);
    }

    #[test]
    fn delete_last_message_moves_head_to_the_previous_message() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let question = append(&pool, history_id, branch_id, "user", "question");
        append(&pool, history_id, branch_id, "assistant", "answer");

        delete_last_message(&pool, history_id, branch_id).expect("delete last message");

        assert_eq!(get_head_message_id(&pool, branch_id).expect("head"), Some(question));
        let messages = load_branch(&pool, branch_id).expect("load branch");
        assert_eq!(ids(&messages), vec![Some(question)]);
        assert_eq!(messages[0].content, "question");
    }

    #[test]
    fn delete_last_message_on_a_single_message_branch_clears_the_head() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        append(&pool, history_id, branch_id, "user", "only message");

        delete_last_message(&pool, history_id, branch_id).expect("delete last message");

        assert_eq!(get_head_message_id(&pool, branch_id).expect("head"), None);
        assert!(load_branch(&pool, branch_id).expect("load branch").is_empty());
    }

    #[test]
    fn get_variants_returns_messages_sharing_the_same_parent() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        let question = append(&pool, history_id, branch_id, "user", "question");
        let first_answer = append(&pool, history_id, branch_id, "assistant", "first answer");
        let fork_id = fork(&pool, history_id, branch_id, question);
        let second_answer = append(&pool, history_id, fork_id, "assistant", "second answer");

        let variants = get_variants(&pool, history_id, Some(question)).expect("variants");

        assert_eq!(
            variants.iter().map(|variant| variant.id).collect::<Vec<_>>(),
            vec![first_answer, second_answer]
        );
        assert_eq!(variants[0].branch_id, branch_id);
        assert_eq!(variants[1].branch_id, fork_id);
    }

    #[test]
    fn search_history_matches_user_message_content() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        append(&pool, history_id, branch_id, "user", "how do I bake sourdough");

        let results = search_history(&pool, "sourdough").expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, history_id);
    }

    #[test]
    fn search_history_matches_a_renamed_title() {
        let pool = in_memory_pool();
        let (history_id, _branch_id) = seed_history(&pool);
        update_title(&pool, &"Old title".to_string(), &history_id).expect("update title");
        update_title(&pool, &"New title".to_string(), &history_id).expect("update title");

        assert!(search_history(&pool, "old").expect("search").is_empty());
        assert_eq!(search_history(&pool, "new").expect("search").len(), 1);
    }

    #[test]
    fn search_history_ignores_assistant_messages() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        append(&pool, history_id, branch_id, "assistant", "rayleigh scattering");

        assert!(search_history(&pool, "rayleigh").expect("search").is_empty());
    }

    #[test]
    fn search_history_matches_partial_words() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        append(&pool, history_id, branch_id, "user", "authentication flow");

        assert_eq!(search_history(&pool, "auth").expect("search").len(), 1);
    }

    #[test]
    fn search_history_returns_every_history_for_a_blank_query() {
        let pool = in_memory_pool();
        let (first, _first_branch) = seed_history(&pool);
        let (second, _second_branch) = seed_history(&pool);

        let mut found: Vec<u32> = search_history(&pool, "   ")
            .expect("search")
            .iter()
            .map(|history| history.id)
            .collect();
        found.sort();

        assert_eq!(found, vec![first, second]);
    }

    #[test]
    fn search_history_drops_a_deleted_user_message() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        append(&pool, history_id, branch_id, "user", "pineapple");
        assert_eq!(search_history(&pool, "pineapple").expect("search").len(), 1);

        delete_last_message(&pool, history_id, branch_id).expect("delete last message");

        assert!(search_history(&pool, "pineapple").expect("search").is_empty());
    }

    #[test]
    fn delete_history_removes_its_messages_and_search_hits() {
        let pool = in_memory_pool();
        let (history_id, branch_id) = seed_history(&pool);
        append(&pool, history_id, branch_id, "user", "ephemeral");

        delete_history(history_id, &pool).expect("delete history");

        assert!(search_history(&pool, "ephemeral").expect("search").is_empty());
        assert!(load_branch(&pool, branch_id).expect("load branch").is_empty());
        assert!(get_history(&pool).expect("get history").is_empty());
    }
}


