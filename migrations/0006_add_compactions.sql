CREATE TABLE IF NOT EXISTS compactions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    original_token_count INTEGER NOT NULL,
    compacted_token_count INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_compactions_session ON compactions(session_id);
INSERT OR IGNORE INTO schema_version (version) VALUES (6);
