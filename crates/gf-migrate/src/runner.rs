//! Migration runner: executes tasks by spawning Claude Code agents in git worktrees.
//!
//! Uses the `ProcessBackend` trait from `gf-process` to spawn agents, with optional
//! `EventBus` integration for lifecycle event emission and `PersonaLoader` for
//! persona-based system prompts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::Utc;
use gf_core::event_bus::EventBus;
use gf_core::events::{ForgeEvent, OutputKind};
use gf_core::ids::{AgentId, SessionId};
use gf_core::{ForgeError, ForgeResult};
use gf_persona::{PersonaLoader, persona_for_finding_source};
use gf_process::backend::{BackendSpawnConfig, ProcessBackend};
use tokio::io::AsyncBufReadExt;
use tracing::{error, info, warn};

use crate::task::{FindingSource, MigrationTask, TaskStatus};

// ---------------------------------------------------------------------------
// TaskResult
// ---------------------------------------------------------------------------

/// Result of a single migration task execution.
#[derive(Debug)]
pub struct TaskResult {
    /// The file that was migrated.
    pub file_path: String,
    /// Updated status after execution.
    pub status: TaskStatus,
    /// Worktree path where changes were made (if any).
    pub worktree_path: Option<PathBuf>,
    /// Raw stdout output from the Claude Code agent.
    pub agent_output: String,
}

// ---------------------------------------------------------------------------
// RunnerConfig
// ---------------------------------------------------------------------------

/// Configuration for the migration runner.
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Path to the iOS repo to modernize.
    pub repo_path: PathBuf,
    /// Whether to use git worktrees for isolation (recommended).
    pub use_worktrees: bool,
    /// Maximum output bytes to capture per agent. Default: 2 MB.
    pub max_output_bytes: usize,
}

impl RunnerConfig {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
            use_worktrees: true,
            max_output_bytes: 2 * 1024 * 1024,
        }
    }
}

// ---------------------------------------------------------------------------
// MigrationRunner
// ---------------------------------------------------------------------------

/// Orchestrates migration task execution: worktree creation, agent spawning,
/// event emission, and result collection.
pub struct MigrationRunner {
    config: RunnerConfig,
    backend: Arc<dyn ProcessBackend>,
    persona_loader: Arc<PersonaLoader>,
    event_bus: Option<Arc<EventBus>>,
}

impl MigrationRunner {
    pub fn new(
        config: RunnerConfig,
        backend: Arc<dyn ProcessBackend>,
        persona_loader: Arc<PersonaLoader>,
        event_bus: Option<Arc<EventBus>>,
    ) -> Self {
        Self {
            config,
            backend,
            persona_loader,
            event_bus,
        }
    }

    /// Assign persona IDs to tasks based on their finding sources.
    ///
    /// For each task, the dominant finding source is determined and the
    /// corresponding persona name is set as the `persona_id`.
    pub fn assign_personas(&self, tasks: &mut [MigrationTask]) {
        for task in tasks.iter_mut() {
            if task.persona_id.is_some() {
                continue;
            }
            let source = dominant_finding_source(task);
            let persona_name = persona_for_finding_source(source);
            task.persona_id = Some(persona_name.to_string());
        }
    }

    /// Execute a single migration task. Creates a worktree, spawns a Claude
    /// Code agent via the backend, collects output, and returns the result.
    pub async fn execute_task(&self, task: &mut MigrationTask) -> ForgeResult<TaskResult> {
        // Skip tasks that are already completed or skipped.
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
        let session_id_str = uuid::Uuid::new_v4().to_string();
        let session_id = SessionId::new();
        let agent_id = AgentId::new();

        // Emit ProcessStarted event.
        self.emit_event(ForgeEvent::ProcessStarted {
            session_id,
            agent_id,
            timestamp: Utc::now(),
        });

        // Determine working directory: worktree or repo root.
        let working_dir = if self.config.use_worktrees {
            match gf_git::create_worktree(&self.config.repo_path, &session_id_str) {
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

        // Look up persona for this task.
        let system_prompt = self.resolve_system_prompt(task);

        // Build BackendSpawnConfig.
        let spawn_config = BackendSpawnConfig {
            prompt: task.prompt.clone(),
            working_dir: working_dir.to_string_lossy().to_string(),
            model: None,
            max_turns: None,
            allowed_tools: None,
            system_prompt: system_prompt,
            resume_session_id: None,
            env: HashMap::new(),
        };

        // Spawn the agent via the backend.
        let result = self
            .run_agent(&spawn_config, &task.file_path, session_id, agent_id)
            .await;

        // Determine worktree path for the result.
        let worktree_path = if self.config.use_worktrees && working_dir != self.config.repo_path {
            Some(working_dir.clone())
        } else {
            None
        };

        match result {
            Ok(output) => {
                task.status = TaskStatus::Completed {
                    summary: truncate_summary(&output, 500),
                };
                self.emit_event(ForgeEvent::ProcessCompleted {
                    session_id,
                    exit_code: 0,
                    timestamp: Utc::now(),
                });
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
                self.emit_event(ForgeEvent::ProcessFailed {
                    session_id,
                    error: error_msg.clone(),
                    timestamp: Utc::now(),
                });
                // Clean up worktree on failure.
                if self.config.use_worktrees {
                    let _ = gf_git::remove_worktree(&self.config.repo_path, &session_id_str);
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

    /// Execute all pending tasks sequentially.
    pub async fn execute_all(&self, tasks: &mut [MigrationTask]) -> Vec<TaskResult> {
        let mut results = Vec::with_capacity(tasks.len());

        for (i, task) in tasks.iter_mut().enumerate() {
            info!(
                task = i + 1,
                total = results.capacity(),
                file = %task.file_path,
                priority = %task.priority,
                findings = task.findings.len(),
                "executing migration task"
            );

            match self.execute_task(task).await {
                Ok(result) => {
                    info!(
                        file = %result.file_path,
                        status = %result.status,
                        "task completed"
                    );
                    results.push(result);
                }
                Err(e) => {
                    error!(
                        file = %task.file_path,
                        error = %e,
                        "task execution failed"
                    );
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

    /// Clean up all worktrees created during migration.
    pub fn cleanup_worktrees(&self, results: &[TaskResult]) {
        for result in results {
            if let Some(ref wt_path) = result.worktree_path {
                if let Some(session_id) = wt_path.file_name().and_then(|n| n.to_str()) {
                    info!(session_id, "removing migration worktree");
                    let _ = gf_git::remove_worktree(&self.config.repo_path, session_id);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Spawn a Claude Code agent via the backend and collect its stdout.
    async fn run_agent(
        &self,
        config: &BackendSpawnConfig,
        file_path: &str,
        session_id: SessionId,
        _agent_id: AgentId,
    ) -> Result<String, ForgeError> {
        let mut handle = self
            .backend
            .spawn(config)
            .await
            .map_err(|e| {
                ForgeError::Process(format!("failed to spawn agent for {}: {}", file_path, e))
            })?;

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
                warn!(
                    file = file_path,
                    "agent output exceeded max bytes, truncating"
                );
                break;
            }
            // Emit each line as ProcessOutput.
            self.emit_event(ForgeEvent::ProcessOutput {
                session_id,
                kind: OutputKind::Assistant,
                content: line.clone(),
                timestamp: Utc::now(),
            });
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

    /// Resolve the system prompt for a task by looking up its persona.
    fn resolve_system_prompt(&self, task: &MigrationTask) -> Option<String> {
        let persona_name = task.persona_id.as_deref().unwrap_or_else(|| {
            let source = dominant_finding_source(task);
            persona_for_finding_source(source)
        });
        self.persona_loader
            .find_by_name(persona_name)
            .map(|p| p.system_prompt.clone())
    }

    /// Emit a ForgeEvent if the event bus is configured.
    fn emit_event(&self, event: ForgeEvent) {
        if let Some(ref bus) = self.event_bus {
            if let Err(e) = bus.emit_sync(event) {
                warn!(error = %e, "failed to emit event");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Determine the dominant finding source for a task (most frequent source).
fn dominant_finding_source(task: &MigrationTask) -> &'static str {
    let mut deprecated = 0usize;
    let mut liquid = 0usize;
    for f in &task.findings {
        match f.source {
            FindingSource::DeprecatedApi => deprecated += 1,
            FindingSource::LiquidGlass => liquid += 1,
        }
    }
    if liquid > deprecated {
        "LiquidGlass"
    } else {
        "DeprecatedApi"
    }
}

/// Truncate a string to at most `max_len` characters, appending "..." if truncated.
fn truncate_summary(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated = s[..max_len].to_string();
        truncated.push_str("...");
        truncated
    }
}

/// Validate that a repo path is a git repository before starting migration.
pub fn validate_repo(repo_path: &Path) -> Result<(), ForgeError> {
    if !repo_path.is_dir() {
        return Err(ForgeError::Validation(format!(
            "repo path does not exist or is not a directory: {}",
            repo_path.display()
        )));
    }
    if !gf_git::is_git_repo(repo_path) {
        return Err(ForgeError::Validation(format!(
            "repo path is not a git repository: {}",
            repo_path.display()
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Finding, FindingSource, Priority};

    #[test]
    fn validate_repo_rejects_missing_dir() {
        let result = validate_repo(Path::new("/nonexistent/path/to/repo"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn validate_repo_rejects_non_git_dir() {
        let dir = std::env::temp_dir().join("gf_runner_test_non_git");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let result = validate_repo(&dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a git repository"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn runner_config_defaults() {
        let config = RunnerConfig::new("/tmp/test-repo");
        assert!(config.use_worktrees);
        assert_eq!(config.max_output_bytes, 2 * 1024 * 1024);
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_summary("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(100);
        let result = truncate_summary(&long, 20);
        assert_eq!(result.len(), 23); // 20 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn dominant_source_deprecated_api() {
        let task = MigrationTask {
            file_path: "test.swift".into(),
            findings: vec![
                Finding {
                    symbol: "keyWindow".into(),
                    line: 10,
                    severity: "high".into(),
                    replacement: None,
                    notes: None,
                    source: FindingSource::DeprecatedApi,
                },
                Finding {
                    symbol: "statusBarFrame".into(),
                    line: 20,
                    severity: "medium".into(),
                    replacement: None,
                    notes: None,
                    source: FindingSource::DeprecatedApi,
                },
            ],
            prompt: String::new(),
            priority: Priority::High,
            status: TaskStatus::Pending,
            persona_id: None,
        };
        assert_eq!(dominant_finding_source(&task), "DeprecatedApi");
    }

    #[test]
    fn dominant_source_liquid_glass() {
        let task = MigrationTask {
            file_path: "test.swift".into(),
            findings: vec![
                Finding {
                    symbol: ".blur".into(),
                    line: 10,
                    severity: "high".into(),
                    replacement: None,
                    notes: None,
                    source: FindingSource::LiquidGlass,
                },
                Finding {
                    symbol: ".visualEffect".into(),
                    line: 20,
                    severity: "high".into(),
                    replacement: None,
                    notes: None,
                    source: FindingSource::LiquidGlass,
                },
                Finding {
                    symbol: "keyWindow".into(),
                    line: 30,
                    severity: "low".into(),
                    replacement: None,
                    notes: None,
                    source: FindingSource::DeprecatedApi,
                },
            ],
            prompt: String::new(),
            priority: Priority::High,
            status: TaskStatus::Pending,
            persona_id: None,
        };
        assert_eq!(dominant_finding_source(&task), "LiquidGlass");
    }

    #[test]
    fn assign_personas_sets_deprecated_api() {
        let loader = empty_persona_loader();
        let backend = Arc::new(StubBackend);
        let runner = MigrationRunner::new(
            RunnerConfig::new("/tmp/test"),
            backend,
            Arc::new(loader),
            None,
        );

        let mut tasks = vec![MigrationTask {
            file_path: "AppDelegate.swift".into(),
            findings: vec![Finding {
                symbol: "keyWindow".into(),
                line: 10,
                severity: "high".into(),
                replacement: None,
                notes: None,
                source: FindingSource::DeprecatedApi,
            }],
            prompt: String::new(),
            priority: Priority::High,
            status: TaskStatus::Pending,
            persona_id: None,
        }];

        runner.assign_personas(&mut tasks);
        assert_eq!(
            tasks[0].persona_id.as_deref(),
            Some("iOS Deprecated API Fixer")
        );
    }

    #[test]
    fn assign_personas_sets_liquid_glass() {
        let loader = empty_persona_loader();
        let backend = Arc::new(StubBackend);
        let runner = MigrationRunner::new(
            RunnerConfig::new("/tmp/test"),
            backend,
            Arc::new(loader),
            None,
        );

        let mut tasks = vec![MigrationTask {
            file_path: "BlurView.swift".into(),
            findings: vec![Finding {
                symbol: ".blur".into(),
                line: 5,
                severity: "high".into(),
                replacement: None,
                notes: None,
                source: FindingSource::LiquidGlass,
            }],
            prompt: String::new(),
            priority: Priority::High,
            status: TaskStatus::Pending,
            persona_id: None,
        }];

        runner.assign_personas(&mut tasks);
        assert_eq!(
            tasks[0].persona_id.as_deref(),
            Some("iOS Liquid Glass Migrator")
        );
    }

    #[test]
    fn assign_personas_skips_already_assigned() {
        let loader = empty_persona_loader();
        let backend = Arc::new(StubBackend);
        let runner = MigrationRunner::new(
            RunnerConfig::new("/tmp/test"),
            backend,
            Arc::new(loader),
            None,
        );

        let mut tasks = vec![MigrationTask {
            file_path: "test.swift".into(),
            findings: vec![],
            prompt: String::new(),
            priority: Priority::Info,
            status: TaskStatus::Pending,
            persona_id: Some("custom-persona".into()),
        }];

        runner.assign_personas(&mut tasks);
        assert_eq!(tasks[0].persona_id.as_deref(), Some("custom-persona"));
    }

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Create a PersonaLoader with no personas (for unit tests that don't need real files).
    fn empty_persona_loader() -> PersonaLoader {
        let dir = std::env::temp_dir().join("gf_runner_empty_personas");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let loader = PersonaLoader::load_from_dir(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        loader
    }

    /// Stub ProcessBackend for tests — never actually spawns anything.
    struct StubBackend;

    impl ProcessBackend for StubBackend {
        fn name(&self) -> &str {
            "stub"
        }

        fn health_check(
            &self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = gf_process::BackendHealth> + Send + '_>,
        > {
            Box::pin(async { gf_process::BackendHealth::Healthy })
        }

        fn capabilities(&self) -> gf_process::BackendCapabilities {
            gf_process::BackendCapabilities {
                supports_streaming: false,
                supports_tools: false,
                supported_models: vec![],
            }
        }

        fn spawn(
            &self,
            _config: &BackendSpawnConfig,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<gf_process::ProcessHandle, gf_process::SpawnError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async {
                Err(gf_process::SpawnError::CommandMissing)
            })
        }
    }
}
