use tauri::Manager;
use std::{fs, path::PathBuf};

use rusqlite::Connection;
use tauri::AppHandle;

pub fn init(app: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_data_dir: PathBuf = app.path().app_data_dir()?;

    fs::create_dir_all(&app_data_dir)?;

    let db_path: PathBuf = app_data_dir.join("kai.db");

    let pool = rusqlite::Connection::open(db_path)?;

    pool.execute("PRAGMA foreign_keys = ON", [])?;
    pool.execute("PRAGMA journal_mode = WAL", [])?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS historys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            active_branch_id INTEGER,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS branches (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            history_id INTEGER NOT NULL REFERENCES historys(id) ON DELETE CASCADE,
            parent_branch_id INTEGER REFERENCES branches(id),
            fork_message_id INTEGER REFERENCES messages(id) ON DELETE SET NULL,
            head_message_id INTEGER REFERENCES messages(id) ON DELETE SET NULL,
            name TEXT NOT NULL DEFAULT 'Branch',
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            history_id INTEGER NOT NULL REFERENCES historys(id) ON DELETE CASCADE,
            branch_id INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
            parent_message_id INTEGER REFERENCES messages(id) ON DELETE SET NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            thinking TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(pool)
}
