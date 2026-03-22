//! Migration run and task repository.

use gf_core::error::{ForgeError, ForgeResult};
use crate::pool::lock_conn;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct MigrationRun {
    pub id: String,
    pub scan_id: String,
    pub repo_path: String,
    pub status: String,
    pub task_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationTask {
    pub id: String,
    pub migration_id: String,
    pub file_path: String,
    pub priority: String,
    pub status: String,
    pub persona_id: Option<String>,
    pub finding_count: i64,
    pub agent_output: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

pub struct MigrationRunRepo {
    conn: Arc<Mutex<Connection>>,
}

impl MigrationRunRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, scan_id: &str, repo_path: &str, task_count: i64) -> ForgeResult<MigrationRun> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "INSERT INTO migration_runs (id, scan_id, repo_path, status, task_count) VALUES (?1, ?2, ?3, 'planning', ?4)",
            params![id, scan_id, repo_path, task_count],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<MigrationRun> {
        let conn = lock_conn(&self.conn)?;
        conn.query_row(
            "SELECT id, scan_id, repo_path, status, task_count, completed_count, failed_count, created_at, completed_at FROM migration_runs WHERE id = ?1",
            params![id],
            |row| {
                Ok(MigrationRun {
                    id: row.get(0)?,
                    scan_id: row.get(1)?,
                    repo_path: row.get(2)?,
                    status: row.get(3)?,
                    task_count: row.get(4)?,
                    completed_count: row.get(5)?,
                    failed_count: row.get(6)?,
                    created_at: row.get(7)?,
                    completed_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ForgeError::NotFound {
                entity: "migration_run",
                id: id.to_string(),
            },
            other => ForgeError::Database(Box::new(other)),
        })
    }

    pub fn update_status(&self, id: &str, status: &str) -> ForgeResult<MigrationRun> {
        let conn = lock_conn(&self.conn)?;
        let completed_at = if status == "completed" || status == "failed" {
            "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')"
        } else {
            "completed_at"
        };
        conn.execute(
            &format!("UPDATE migration_runs SET status = ?2, completed_at = {completed_at} WHERE id = ?1"),
            params![id, status],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(id)
    }

    pub fn increment_completed(&self, id: &str) -> ForgeResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE migration_runs SET completed_count = completed_count + 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn increment_failed(&self, id: &str) -> ForgeResult<()> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE migration_runs SET failed_count = failed_count + 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn create_task(
        &self,
        migration_id: &str,
        file_path: &str,
        priority: &str,
        persona_id: Option<&str>,
        finding_count: i64,
    ) -> ForgeResult<MigrationTask> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "INSERT INTO migration_tasks (id, migration_id, file_path, priority, status, persona_id, finding_count) VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6)",
            params![id, migration_id, file_path, priority, persona_id, finding_count],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get_task(&id)
    }

    pub fn get_task(&self, id: &str) -> ForgeResult<MigrationTask> {
        let conn = lock_conn(&self.conn)?;
        conn.query_row(
            "SELECT id, migration_id, file_path, priority, status, persona_id, finding_count, agent_output, error_message, created_at, completed_at FROM migration_tasks WHERE id = ?1",
            params![id],
            |row| {
                Ok(MigrationTask {
                    id: row.get(0)?,
                    migration_id: row.get(1)?,
                    file_path: row.get(2)?,
                    priority: row.get(3)?,
                    status: row.get(4)?,
                    persona_id: row.get(5)?,
                    finding_count: row.get(6)?,
                    agent_output: row.get(7)?,
                    error_message: row.get(8)?,
                    created_at: row.get(9)?,
                    completed_at: row.get(10)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ForgeError::NotFound {
                entity: "migration_task",
                id: id.to_string(),
            },
            other => ForgeError::Database(Box::new(other)),
        })
    }

    pub fn list_tasks(&self, migration_id: &str) -> ForgeResult<Vec<MigrationTask>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, migration_id, file_path, priority, status, persona_id, finding_count, agent_output, error_message, created_at, completed_at FROM migration_tasks WHERE migration_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![migration_id], |row| {
                Ok(MigrationTask {
                    id: row.get(0)?,
                    migration_id: row.get(1)?,
                    file_path: row.get(2)?,
                    priority: row.get(3)?,
                    status: row.get(4)?,
                    persona_id: row.get(5)?,
                    finding_count: row.get(6)?,
                    agent_output: row.get(7)?,
                    error_message: row.get(8)?,
                    created_at: row.get(9)?,
                    completed_at: row.get(10)?,
                })
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(|e| ForgeError::Database(Box::new(e)))?);
        }
        Ok(tasks)
    }

    pub fn update_task_completed(&self, id: &str, agent_output: &str) -> ForgeResult<MigrationTask> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE migration_tasks SET status = 'completed', agent_output = ?2, completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, agent_output],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get_task(id)
    }

    pub fn update_task_failed(&self, id: &str, error_message: &str) -> ForgeResult<MigrationTask> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE migration_tasks SET status = 'failed', error_message = ?2, completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, error_message],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get_task(id)
    }

    pub fn update_task_status(&self, id: &str, status: &str) -> ForgeResult<MigrationTask> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE migration_tasks SET status = ?2 WHERE id = ?1",
            params![id, status],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get_task(id)
    }
}
