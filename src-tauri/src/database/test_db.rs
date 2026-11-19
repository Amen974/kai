use std::sync::Mutex;

use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

const TEST_MIGRATIONS: &[M] = &[
    M::up(include_str!("migrations/001_initial_schema.sql")),
    M::up(include_str!("migrations/002_add_fts5.sql")),
    M::up(include_str!("migrations/003_add_models.sql")),
    M::up(include_str!("migrations/004_rename_historys.sql")),
];

pub(crate) fn in_memory_pool() -> Mutex<Connection> {
    let mut conn = Connection::open_in_memory().expect("open in-memory database");
    conn.pragma_update(None, "foreign_keys", "ON")
        .expect("enable foreign keys");
    Migrations::from_slice(TEST_MIGRATIONS)
        .to_latest(&mut conn)
        .expect("apply migrations");
    Mutex::new(conn)
}
