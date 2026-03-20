CREATE TABLE IF NOT EXISTS hooks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    event_type TEXT NOT NULL,
    timing TEXT NOT NULL CHECK (timing IN ('pre', 'post')),
    command TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_hooks_event_timing ON hooks(event_type, timing);
INSERT INTO schema_version (version) VALUES (4);
