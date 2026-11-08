CREATE TABLE historys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    active_branch_id INTEGER,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE branches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    history_id INTEGER NOT NULL REFERENCES historys(id) ON DELETE CASCADE,
    parent_branch_id INTEGER REFERENCES branches(id),
    fork_message_id INTEGER REFERENCES messages(id) ON DELETE SET NULL,
    head_message_id INTEGER REFERENCES messages(id) ON DELETE SET NULL,
    name TEXT NOT NULL DEFAULT 'Branch',
    created_at TEXT NOT NULL
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    history_id INTEGER NOT NULL REFERENCES historys(id) ON DELETE CASCADE,
    branch_id INTEGER NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    parent_message_id INTEGER REFERENCES messages(id) ON DELETE SET NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    thinking TEXT,
    created_at TEXT NOT NULL
);