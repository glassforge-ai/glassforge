//! GlassForge database layer: pool, migrations, batch writer, and repositories.
#![forbid(unsafe_code)]

pub mod batch_writer;
pub mod migrations;
pub mod pool;
pub mod repos;

pub use batch_writer::BatchWriter;
pub use migrations::Migrator;
pub use pool::DbPool;
pub use pool::lock_conn;
pub use repos::agents::AgentRepo;
pub use repos::events::{EventRepo, StoredEvent};
pub use repos::sessions::{NewSession, Session, SessionRepo};
pub use repos::skills::{Skill, SkillRepo, SkillRule};
pub use repos::scans::{Scan, ScanRepo};
pub use repos::migration_runs::{MigrationRun, MigrationRunRepo, MigrationTask as DbMigrationTask};
pub use repos::workflows::{Workflow, WorkflowRepo};
