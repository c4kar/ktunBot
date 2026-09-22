-- ktunbot init schema (idempotent, SQLite/libSQL compatible)

-- Persistent L2 cache blobs (insert-or-replace from the Rust app).
-- Keys mirror the moka L1 keys (e.g. "announcements_10", "menus_2025-12").
CREATE TABLE IF NOT EXISTS cache_blob (
    key TEXT PRIMARY KEY,
    value BLOB NOT NULL,
    expires_at INTEGER NOT NULL
);

-- Lightweight bot state (room for future use: per-chat flags, counters).
-- Kept minimal now (YAGNI); the platform plan will formalise this.
CREATE TABLE IF NOT EXISTS bot_state (
    chat_id INTEGER NOT NULL,
    key TEXT NOT NULL,
    value TEXT,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (chat_id, key)
);