use tauri::Manager;
use std::{fs, path::PathBuf};

use rusqlite::Connection;
use tauri::AppHandle;

pub fn init(app: &AppHandle) -> Result<Connection,  Box<dyn std::error::Error>> {
    let app_data_dir: PathBuf = app.path().app_data_dir()?;

    fs::create_dir_all(&app_data_dir)?;

    let db_path: PathBuf = app_data_dir.join("kai.db");

    let pool = rusqlite::Connection::open(db_path)?;

    pool.execute("PRAGMA foreign_keys = ON", [])?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;

    pool.execute(
        "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                history_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                thinking TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(history_id) REFERENCES history(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(pool)
}