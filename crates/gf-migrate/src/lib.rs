//! GlassForge migration pipeline: parse scan artifacts, plan migrations, execute via Claude Code agents.
//!
//! Takes a GlassForge JSON artifact (deprecated APIs + Liquid Glass findings),
//! groups findings by file, prioritizes by severity, and spawns Claude Code agents
//! in isolated git worktrees to apply fixes.

pub mod artifact;
pub mod planner;
pub mod prompt;
pub mod runner;
pub mod task;

pub use artifact::{parse_artifact, parse_artifact_str, ScanArtifact};
pub use planner::{plan_migration, MigrationPlan};
pub use prompt::generate_prompt;
pub use runner::MigrationRunner;
pub use task::{MigrationTask, TaskStatus};
