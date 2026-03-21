//! Migration pipeline: parse artifact, plan tasks, execute via Claude Code agents.
//!
//! Implements the runner inline since `gf_migrate::runner` is not yet available.
//! Once that module lands, this can delegate to it directly.

use std::path::{Path, PathBuf};

use gf_core::ForgeError;
use gf_migrate::task::{FindingSource, MigrationTask, TaskStatus};
use gf_migrate::{parse_artifact, plan_migration, MigrationPlan};
use gf_persona::{PersonaLoader, persona_for_finding_source};
use gf_process::{SpawnConfig, spawn};
use tokio::io::AsyncBufReadExt;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// TaskResult
// ---------------------------------------------------------------------------

/// Result of executing a single migration task.
#[derive(Debug)]
pub struct TaskResult {
    /// File that was migrated.
    pub file_path: String,
    /// Updated status after execution.
    pub status: TaskStatus,
    /// Worktree path where changes were applied (if any).
    pub worktree_path: Option<PathBuf>,
    /// Raw stdout output from the agent.
    pub agent_output: String,
}

// ---------------------------------------------------------------------------
// RunnerConfig
// ---------------------------------------------------------------------------

/// Configuration for the inline migration runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Path to the iOS repo to modernize.
    pub repo_path: PathBuf,
    /// Whether to use git worktrees for isolation.
    pub use_worktrees: bool,
    /// SpawnConfig override. Uses SpawnConfig::from_env() if None.
    pub spawn_config: Option<SpawnConfig>,
    /// Maximum bytes of agent output to capture per task.
    pub max_output_bytes: usize,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Inline migration runner that spawns Claude Code agents per task.
pub struct MigrationRunner {
    config: RunnerConfig,
}

impl MigrationRunner {
    pub fn new(config: RunnerConfig) -> Self {
        Self { config }
    }

    /// Execute a single migration task.
    pub async fn execute_task(&self, task: &mut MigrationTask) -> Result<TaskResult, ForgeError> {
        // Skip already-resolved tasks.
        if matches!(
            task.status,
            TaskStatus::Completed { .. } | TaskStatus::Skipped { .. }
        ) {
            return Ok(TaskResult {
                file_path: task.file_path.clone(),
                status: task.status.clone(),
                worktree_path: None,
                agent_output: String::new(),
            });
        }

        task.status = TaskStatus::Running;
        let session_id = uuid::Uuid::new_v4().to_string();

        // Determine working directory.
        let working_dir = if self.config.use_worktrees {
            match gf_git::create_worktree(&self.config.repo_path, &session_id) {
                Ok(wt_path) => {
                    info!(
                        file = %task.file_path,
                        worktree = %wt_path.display(),
                        "created worktree for migration task"
                    );
                    wt_path
                }
                Err(e) => {
                    warn!(
                        file = %task.file_path,
                        error = %e,
                        "failed to create worktree, falling back to repo root"
                    );
                    self.config.repo_path.clone()
                }
            }
        } else {
            self.config.repo_path.clone()
        };

        // Build spawn config.
        let spawn_config = self
            .config
            .spawn_config
            .clone()
            .unwrap_or_else(SpawnConfig::from_env)
            .with_working_dir(&working_dir);

        // Spawn agent and collect output.
        let result = self
            .run_agent(&spawn_config, &task.prompt, &task.file_path)
            .await;

        let worktree_path = if self.config.use_worktrees && working_dir != self.config.repo_path {
            Some(working_dir)
        } else {
            None
        };

        match result {
            Ok(output) => {
                task.status = TaskStatus::Completed {
                    summary: truncate(&output, 500),
                };
                Ok(TaskResult {
                    file_path: task.file_path.clone(),
                    status: task.status.clone(),
                    worktree_path,
                    agent_output: output,
                })
            }
            Err(e) => {
                let error_msg = e.to_string();
                task.status = TaskStatus::Failed {
                    error: error_msg.clone(),
                };
                // Clean up worktree on failure.
                if self.config.use_worktrees {
                    let _ = gf_git::remove_worktree(&self.config.repo_path, &session_id);
                }
                Ok(TaskResult {
                    file_path: task.file_path.clone(),
                    status: task.status.clone(),
                    worktree_path: None,
                    agent_output: error_msg,
                })
            }
        }
    }

    /// Execute all tasks sequentially.
    pub async fn execute_all(&self, tasks: &mut [MigrationTask]) -> Vec<TaskResult> {
        let total = tasks.len();
        let mut results = Vec::with_capacity(total);

        for (i, task) in tasks.iter_mut().enumerate() {
            info!(
                task = i + 1,
                total,
                file = %task.file_path,
                priority = %task.priority,
                findings = task.findings.len(),
                "executing migration task"
            );

            match self.execute_task(task).await {
                Ok(result) => {
                    info!(file = %result.file_path, status = %result.status, "task completed");
                    results.push(result);
                }
                Err(e) => {
                    error!(file = %task.file_path, error = %e, "task execution failed");
                    task.status = TaskStatus::Failed {
                        error: e.to_string(),
                    };
                    results.push(TaskResult {
                        file_path: task.file_path.clone(),
                        status: task.status.clone(),
                        worktree_path: None,
                        agent_output: e.to_string(),
                    });
                }
            }
        }

        results
    }

    /// Spawn a Claude Code agent and collect its stdout.
    async fn run_agent(
        &self,
        spawn_config: &SpawnConfig,
        prompt: &str,
        file_path: &str,
    ) -> Result<String, ForgeError> {
        let mut handle = spawn(spawn_config, prompt, None)
            .await
            .map_err(|e| ForgeError::Process(format!("failed to spawn agent for {}: {}", file_path, e)))?;

        let stdout = handle
            .take_stdout()
            .ok_or_else(|| ForgeError::Process("agent stdout not available".into()))?;

        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut output = String::new();
        let max_bytes = self.config.max_output_bytes;

        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| ForgeError::Process(format!("reading agent output: {}", e)))?
        {
            if output.len() + line.len() > max_bytes {
                warn!(file = file_path, "agent output exceeded max bytes, truncating");
                break;
            }
            output.push_str(&line);
            output.push('\n');
        }

        let exit_status = handle
            .wait()
            .await
            .map_err(|e| ForgeError::Process(format!("waiting for agent: {}", e)))?;

        if !exit_status.success() {
            let code = exit_status.code().unwrap_or(-1);
            return Err(ForgeError::Process(format!(
                "agent for {} exited with code {}",
                file_path, code
            )));
        }

        Ok(output)
    }
}

// ---------------------------------------------------------------------------
// Persona assignment
// ---------------------------------------------------------------------------

/// Assign persona IDs to tasks based on their dominant finding source.
pub fn assign_personas(tasks: &mut [MigrationTask], loader: &PersonaLoader) {
    for task in tasks.iter_mut() {
        if matches!(task.status, TaskStatus::Skipped { .. }) {
            continue;
        }

        // Determine dominant finding source for this task.
        let source_label = dominant_source(&task.findings);
        let persona_name = persona_for_finding_source(source_label);

        if let Some(persona) = loader.find_by_name(persona_name) {
            task.persona_id = Some(persona.name.clone());
        }
    }
}

/// Return the finding-source label for the dominant source in a set of findings.
fn dominant_source(findings: &[gf_migrate::task::Finding]) -> &'static str {
    let mut deprecated = 0usize;
    let mut liquid_glass = 0usize;
    for f in findings {
        match f.source {
            FindingSource::DeprecatedApi => deprecated += 1,
            FindingSource::LiquidGlass => liquid_glass += 1,
        }
    }
    if liquid_glass > deprecated {
        "LiquidGlass"
    } else {
        "DeprecatedApi"
    }
}

// ---------------------------------------------------------------------------
// Top-level command entry point
// ---------------------------------------------------------------------------

/// Run the full migration pipeline.
///
/// Returns `Ok(true)` on success, `Ok(false)` if no actionable tasks, or an error.
pub async fn run_migrate(
    artifact_path: &Path,
    repo_path: &Path,
    dry_run: bool,
    no_worktree: bool,
) -> Result<bool, anyhow::Error> {
    // 1. Validate repo is a git directory.
    if !gf_git::is_git_repo(repo_path) {
        anyhow::bail!(
            "repo path is not a git repository: {}",
            repo_path.display()
        );
    }

    // 2. Parse artifact.
    info!(artifact = %artifact_path.display(), "parsing GlassForge artifact");
    let artifact = parse_artifact(artifact_path)
        .map_err(|e| anyhow::anyhow!("failed to parse artifact: {}", e))?;

    // Print scan metadata if available.
    if let Some(ref meta) = artifact.meta {
        info!(
            repo = meta.repo_name.as_deref().unwrap_or("unknown"),
            branch = meta.branch.as_deref().unwrap_or("unknown"),
            commit = meta.commit.as_deref().unwrap_or("unknown"),
            target_sdk = meta.target_sdk.unwrap_or(0),
            "scan metadata"
        );
    }

    // 3. Plan migration.
    let mut plan = plan_migration(&artifact);
    info!(summary = %plan.summary(), "migration plan ready");

    print_plan(&plan);

    // 4. Dry-run stops here.
    if dry_run {
        println!("Dry run complete. No agents were executed.");
        return Ok(true);
    }

    // 5. Count actionable tasks.
    let actionable_count = plan
        .tasks
        .iter()
        .filter(|t| matches!(t.status, TaskStatus::Pending))
        .count();

    if actionable_count == 0 {
        println!("No actionable migration tasks. Nothing to do.");
        return Ok(false);
    }

    // 6. Load personas (best-effort; migration works without them).
    let personas_dir = find_personas_dir();
    if let Some(ref dir) = personas_dir {
        match PersonaLoader::load_from_dir(dir) {
            Ok(loader) => {
                info!(count = loader.all().len(), "loaded personas");
                assign_personas(&mut plan.tasks, &loader);
            }
            Err(e) => {
                warn!(error = %e, "failed to load personas, continuing without persona assignment");
            }
        }
    } else {
        info!("no personas directory found, continuing without persona assignment");
    }

    // 7. Execute.
    println!(
        "Executing {} actionable migration tasks...",
        actionable_count
    );
    println!();

    let config = RunnerConfig {
        repo_path: repo_path.to_path_buf(),
        use_worktrees: !no_worktree,
        spawn_config: None,
        max_output_bytes: 2 * 1024 * 1024,
    };
    let runner = MigrationRunner::new(config);
    let results = runner.execute_all(&mut plan.tasks).await;

    // 8. Print results.
    print_results(&results);

    let failed = results
        .iter()
        .filter(|r| matches!(r.status, TaskStatus::Failed { .. }))
        .count();

    if failed > 0 {
        error!(failed, "some migration tasks failed");
        anyhow::bail!("{} migration task(s) failed", failed);
    }

    Ok(true)
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

fn print_plan(plan: &MigrationPlan) {
    println!();
    println!("=== Migration Plan ===");
    println!("{}", plan.summary());
    println!();

    for (i, task) in plan.tasks.iter().enumerate() {
        let status_tag = match &task.status {
            TaskStatus::Skipped { reason } => format!("[SKIP: {}]", reason),
            _ => format!("[{}]", task.priority),
        };
        let persona_tag = task
            .persona_id
            .as_deref()
            .map(|p| format!(" ({})", p))
            .unwrap_or_default();
        println!(
            "  {:>3}. {} {}{} ({} findings)",
            i + 1,
            status_tag,
            task.file_path,
            persona_tag,
            task.findings.len(),
        );
    }
    println!();
}

fn print_results(results: &[TaskResult]) {
    println!();
    println!("=== Migration Results ===");
    println!();

    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    for result in results {
        match &result.status {
            TaskStatus::Completed { summary } => {
                succeeded += 1;
                println!("  [OK] {} - {}", result.file_path, summary);
                if let Some(ref wt) = result.worktree_path {
                    println!("       Changes in worktree: {}", wt.display());
                }
            }
            TaskStatus::Failed { error } => {
                failed += 1;
                println!("  [FAIL] {} - {}", result.file_path, error);
            }
            TaskStatus::Skipped { reason } => {
                skipped += 1;
                println!("  [SKIP] {} - {}", result.file_path, reason);
            }
            other => {
                println!("  [{}] {}", other, result.file_path);
            }
        }
    }

    println!();
    println!(
        "Summary: {} succeeded, {} failed, {} skipped (of {} total)",
        succeeded,
        failed,
        skipped,
        results.len()
    );
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Look for a personas directory in well-known locations.
fn find_personas_dir() -> Option<PathBuf> {
    // 1. Relative to CWD: personas/specialized/
    let cwd_relative = PathBuf::from("personas/specialized");
    if cwd_relative.is_dir() {
        return Some(cwd_relative);
    }

    // 2. Relative to the binary location.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let from_bin = parent.join("personas/specialized");
            if from_bin.is_dir() {
                return Some(from_bin);
            }
            // Also try one level up (e.g. target/release/../personas)
            if let Some(grandparent) = parent.parent() {
                let from_grandparent = grandparent.join("personas/specialized");
                if from_grandparent.is_dir() {
                    return Some(from_grandparent);
                }
            }
        }
    }

    // 3. GLASSFORGE_PERSONAS_DIR env var.
    if let Ok(dir) = std::env::var("GLASSFORGE_PERSONAS_DIR") {
        let p = PathBuf::from(&dir);
        if p.is_dir() {
            return Some(p);
        }
    }

    None
}

/// Truncate a string to `max_len` characters, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut t = s[..max_len].to_string();
        t.push_str("...");
        t
    }
}
