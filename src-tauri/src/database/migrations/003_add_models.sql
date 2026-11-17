CREATE TABLE models (
    name TEXT PRIMARY KEY,
    family TEXT NOT NULL,
    parameter_size TEXT NOT NULL,
    quantization_level TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    modified_at TEXT NOT NULL,
    supports_vision INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX models_single_active ON models (active) WHERE active = 1;
