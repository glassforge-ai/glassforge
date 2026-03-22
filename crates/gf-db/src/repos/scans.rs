//! Scan result repository.

use gf_core::error::{ForgeError, ForgeResult};
use crate::pool::lock_conn;
use rusqlite::{Connection, params};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct Scan {
    pub id: String,
    pub repo_path: String,
    pub status: String,
    pub score: Option<i64>,
    pub finding_count: Option<i64>,
    pub artifact_json: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

pub struct ScanRepo {
    conn: Arc<Mutex<Connection>>,
}

impl ScanRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, repo_path: &str) -> ForgeResult<Scan> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "INSERT INTO scans (id, repo_path, status) VALUES (?1, ?2, 'pending')",
            params![id, repo_path],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(&id)
    }

    pub fn get(&self, id: &str) -> ForgeResult<Scan> {
        let conn = lock_conn(&self.conn)?;
        conn.query_row(
            "SELECT id, repo_path, status, score, finding_count, artifact_json, error_message, created_at, completed_at FROM scans WHERE id = ?1",
            params![id],
            |row| {
                Ok(Scan {
                    id: row.get(0)?,
                    repo_path: row.get(1)?,
                    status: row.get(2)?,
                    score: row.get(3)?,
                    finding_count: row.get(4)?,
                    artifact_json: row.get(5)?,
                    error_message: row.get(6)?,
                    created_at: row.get(7)?,
                    completed_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ForgeError::NotFound {
                entity: "scan",
                id: id.to_string(),
            },
            other => ForgeError::Database(Box::new(other)),
        })
    }

    pub fn list(&self) -> ForgeResult<Vec<Scan>> {
        let conn = lock_conn(&self.conn)?;
        let mut stmt = conn
            .prepare(
                "SELECT id, repo_path, status, score, finding_count, artifact_json, error_message, created_at, completed_at FROM scans ORDER BY created_at DESC LIMIT 100",
            )
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Scan {
                    id: row.get(0)?,
                    repo_path: row.get(1)?,
                    status: row.get(2)?,
                    score: row.get(3)?,
                    finding_count: row.get(4)?,
                    artifact_json: row.get(5)?,
                    error_message: row.get(6)?,
                    created_at: row.get(7)?,
                    completed_at: row.get(8)?,
                })
            })
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let mut scans = Vec::new();
        for row in rows {
            scans.push(row.map_err(|e| ForgeError::Database(Box::new(e)))?);
        }
        Ok(scans)
    }

    pub fn update_completed(
        &self,
        id: &str,
        score: Option<i64>,
        finding_count: Option<i64>,
        artifact_json: &str,
    ) -> ForgeResult<Scan> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE scans SET status = 'completed', score = ?2, finding_count = ?3, artifact_json = ?4, completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, score, finding_count, artifact_json],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(id)
    }

    pub fn update_status(&self, id: &str, status: &str) -> ForgeResult<Scan> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE scans SET status = ?2 WHERE id = ?1",
            params![id, status],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(id)
    }

    pub fn update_failed(&self, id: &str, error_message: &str) -> ForgeResult<Scan> {
        let conn = lock_conn(&self.conn)?;
        conn.execute(
            "UPDATE scans SET status = 'failed', error_message = ?2, completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
            params![id, error_message],
        )
        .map_err(|e| ForgeError::Database(Box::new(e)))?;
        drop(conn);
        self.get(id)
    }
}
