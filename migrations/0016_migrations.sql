-- Migration runs table
CREATE TABLE IF NOT EXISTS migration_runs (
    id TEXT PRIMARY KEY,
    scan_id TEXT NOT NULL REFERENCES scans(id),
    repo_path TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, planning, running, completed, failed
    task_count INTEGER NOT NULL DEFAULT 0,
    completed_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_migration_runs_scan_id ON migration_runs(scan_id);

-- Individual migration tasks
CREATE TABLE IF NOT EXISTS migration_tasks (
    id TEXT PRIMARY KEY,
    migration_id TEXT NOT NULL REFERENCES migration_runs(id),
    file_path TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'low',
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, running, completed, failed, skipped
    persona_id TEXT,
    finding_count INTEGER NOT NULL DEFAULT 0,
    agent_output TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_migration_tasks_migration_id ON migration_tasks(migration_id);
