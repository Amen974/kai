use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use tauri::{AppHandle, Manager};
use std::{fs, path::PathBuf};

const MIGRATIONS: &[M] = &[
    M::up(include_str!("migrations/001_initial_schema.sql")),
    M::up(include_str!("migrations/002_add_fts5.sql")),
    M::up(include_str!("migrations/003_add_models.sql")),
    M::up(include_str!("migrations/004_rename_historys.sql")),
    M::up(include_str!("migrations/005_add_attachments.sql")),
];

const MIGRATOR: Migrations = Migrations::from_slice(MIGRATIONS);

pub fn migrate(conn: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    MIGRATOR.to_latest(conn)
}

pub fn init(app: &AppHandle) -> Result<Connection, Box<dyn std::error::Error>> {
    let app_data_dir: PathBuf = app.path().app_data_dir()?;

    fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("kai.db");

    let mut conn = Connection::open(db_path)?;

    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    migrate(&mut conn)?;

    Ok(conn)
}