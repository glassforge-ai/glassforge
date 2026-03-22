//! Schema migration runner.

use gf_core::error::{ForgeError, ForgeResult};
use rusqlite::Connection;
use tracing::info;

const MIGRATION_001: &str = include_str!("../../../migrations/0001_init.sql");
const MIGRATION_002: &str = include_str!("../../../migrations/0002_add_cost.sql");
const MIGRATION_003: &str = include_str!("../../../migrations/0003_add_memory.sql");
const MIGRATION_004: &str = include_str!("../../../migrations/0004_add_hooks.sql");
const MIGRATION_005: &str = include_str!("../../../migrations/0005_scheduler_analytics.sql");
const MIGRATION_006: &str = include_str!("../../../migrations/0006_add_compactions.sql");
const MIGRATION_007: &str = include_str!("../../../migrations/0007_add_workflow_columns.sql");
const MIGRATION_008: &str = include_str!("../../../migrations/0008_memory_types_and_skill_rules.sql");
const MIGRATION_009: &str = include_str!("../../../migrations/0009_personas.sql");
const MIGRATION_011: &str = include_str!("../../../migrations/0011_org_charts.sql");
const MIGRATION_012: &str = include_str!("../../../migrations/0012_agents_persona_id.sql");
const MIGRATION_013: &str = include_str!("../../../migrations/0013_safety_state.sql");
const MIGRATION_014: &str = include_str!("../../../migrations/0014_agents_backend_type.sql");
const MIGRATION_015: &str = include_str!("../../../migrations/0015_scans.sql");
const MIGRATION_016: &str = include_str!("../../../migrations/0016_migrations.sql");

pub struct Migrator<'a> {
    conn: &'a Connection,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn current_version(&self) -> ForgeResult<u32> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        if !exists {
            return Ok(0);
        }

        let version: u32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(version)
    }

    pub fn apply_pending(&self) -> ForgeResult<u32> {
        let mut current = self.current_version()?;
        let mut applied = 0;

        let migrations: &[(u32, &str, &str)] = &[
            (1, "0001_init.sql", MIGRATION_001),
            (2, "0002_add_cost.sql", MIGRATION_002),
            (3, "0003_add_memory.sql", MIGRATION_003),
            (4, "0004_add_hooks.sql", MIGRATION_004),
            (5, "0005_scheduler_analytics.sql", MIGRATION_005),
            (6, "0006_add_compactions.sql", MIGRATION_006),
            (7, "0007_add_workflow_columns.sql", MIGRATION_007),
            (8, "0008_memory_types_and_skill_rules.sql", MIGRATION_008),
            (9, "0009_personas.sql", MIGRATION_009),
            (11, "0011_org_charts.sql", MIGRATION_011),
            (12, "0012_agents_persona_id.sql", MIGRATION_012),
            (13, "0013_safety_state.sql", MIGRATION_013),
            (14, "0014_agents_backend_type.sql", MIGRATION_014),
            (15, "0015_scans.sql", MIGRATION_015),
            (16, "0016_migrations.sql", MIGRATION_016),
        ];

        for &(version, name, sql) in migrations {
            if current < version {
                info!("applying migration {}", name);
                self.conn
                    .execute_batch(sql)
                    .map_err(|e| ForgeError::Database(Box::new(e)))?;
                current = version;
                applied += 1;
                info!("migration {} applied, now at version {}", name, version);
            }
        }

        if applied == 0 {
            info!(version = current, "schema already at latest version");
        }

        Ok(applied)
    }
}
