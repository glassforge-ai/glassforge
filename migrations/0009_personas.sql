CREATE TABLE IF NOT EXISTS persona_divisions (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    agent_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS personas (
    id TEXT PRIMARY KEY,
    division_slug TEXT NOT NULL REFERENCES persona_divisions(slug) ON DELETE RESTRICT,
    slug TEXT NOT NULL,
    name TEXT NOT NULL,
    short_description TEXT NOT NULL,
    personality TEXT,
    deliverables TEXT,
    success_metrics TEXT,
    workflow TEXT,
    tags_json TEXT,
    source_file TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_personas_source_file ON personas(source_file);
INSERT INTO schema_version (version) VALUES (9);
