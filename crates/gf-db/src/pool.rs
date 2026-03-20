//! Database connection pool with r2d2 and busy_timeout.

use gf_core::error::{ForgeError, ForgeResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const FILE_PRAGMAS: &str = "\
    PRAGMA journal_mode = WAL;\
    PRAGMA synchronous  = NORMAL;\
    PRAGMA foreign_keys = ON;\
    PRAGMA cache_size   = -8000;\
    PRAGMA busy_timeout = 5000;";

const MEMORY_PRAGMAS: &str = "\
    PRAGMA foreign_keys = ON;\
    PRAGMA busy_timeout = 5000;";

pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
    write_pool: Pool<SqliteConnectionManager>,
    read_pool: Pool<SqliteConnectionManager>,
    path: Option<PathBuf>,
}

impl DbPool {
    pub fn new(path: &Path) -> ForgeResult<Self> {
        let conn = Connection::open(path).map_err(|e| ForgeError::Database(Box::new(e)))?;
        conn.execute_batch(FILE_PRAGMAS)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let write_mgr = SqliteConnectionManager::file(path).with_init(|c| {
            c.execute_batch(FILE_PRAGMAS)?;
            Ok(())
        });
        let write_pool = Pool::builder()
            .max_size(1)
            .build(write_mgr)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let num_readers = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(4)
            .max(2);
        let read_mgr = SqliteConnectionManager::file(path).with_init(|c| {
            c.execute_batch(FILE_PRAGMAS)?;
            Ok(())
        });
        let read_pool = Pool::builder()
            .max_size(num_readers)
            .build(read_mgr)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            write_pool,
            read_pool,
            path: Some(path.to_path_buf()),
        })
    }

    pub fn in_memory() -> ForgeResult<Self> {
        let conn = Connection::open_in_memory().map_err(|e| ForgeError::Database(Box::new(e)))?;
        conn.execute_batch(MEMORY_PRAGMAS)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;

        let write_mgr = SqliteConnectionManager::memory().with_init(|c| {
            c.execute_batch(MEMORY_PRAGMAS)?;
            Ok(())
        });
        let write_pool = Pool::builder()
            .max_size(1)
            .build(write_mgr)
            .map_err(|e| ForgeError::Database(Box::new(e)))?;
        let read_pool = write_pool.clone();

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            write_pool,
            read_pool,
            path: None,
        })
    }

    pub fn writer(&self) -> ForgeResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.write_pool
            .get()
            .map_err(|e| ForgeError::Database(Box::new(e)))
    }

    pub fn reader(&self) -> ForgeResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.read_pool
            .get()
            .map_err(|e| ForgeError::Database(Box::new(e)))
    }

    pub fn connection(&self) -> ForgeResult<std::sync::MutexGuard<'_, Connection>> {
        lock_conn(&self.conn)
    }

    pub fn conn_arc(&self) -> ForgeResult<Arc<Mutex<Connection>>> {
        match &self.path {
            Some(p) => {
                let c = Connection::open(p).map_err(|e| ForgeError::Database(Box::new(e)))?;
                c.execute_batch(FILE_PRAGMAS).ok();
                Ok(Arc::new(Mutex::new(c)))
            }
            None => Ok(Arc::clone(&self.conn)),
        }
    }
}

pub fn lock_conn(conn: &Arc<Mutex<Connection>>) -> ForgeResult<std::sync::MutexGuard<'_, Connection>> {
    conn.lock().map_err(|_| ForgeError::Internal("database mutex poisoned".into()))
}
