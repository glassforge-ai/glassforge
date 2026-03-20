//! Spawn the CLI with working directory and env isolation.

use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::process::{Child, Command};

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("failed to spawn process: {0}")]
    Io(#[from] std::io::Error),

    #[error("command missing or empty")]
    CommandMissing,
}

/// Configuration for spawning the agent process.
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub command: String,
    pub args_before_prompt: Vec<String>,
    pub working_dir: Option<std::path::PathBuf>,
    pub env_remove: Vec<String>,
    pub env_set: Vec<(String, String)>,
    pub timeout: Option<Duration>,
    pub max_concurrent: usize,
    pub max_output_bytes: usize,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            command: "claude".to_string(),
            args_before_prompt: vec![
                "--output-format".into(),
                "stream-json".into(),
                "--verbose".into(),
            ],
            working_dir: None,
            env_remove: vec![
                "CLAUDECODE".to_string(),
                "ANTHROPIC_API_KEY".to_string(),
            ],
            env_set: vec![],
            timeout: Some(Duration::from_secs(300)),
            max_concurrent: 4,
            max_output_bytes: 10 * 1024 * 1024,
        }
    }
}

pub const ENV_CLI_COMMAND: &str = "FORGE_CLI_COMMAND";
pub const ENV_CLI_ARGS: &str = "FORGE_CLI_ARGS";

impl SpawnConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(cmd) = std::env::var(ENV_CLI_COMMAND) {
            if !cmd.is_empty() {
                config.command = cmd;
            }
        }
        if let Ok(args) = std::env::var(ENV_CLI_ARGS) {
            let parsed: Vec<String> = args
                .split_whitespace()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !parsed.is_empty() {
                config.args_before_prompt = parsed;
            }
        }
        if let Ok(max) = std::env::var("FORGE_MAX_CONCURRENT") {
            if let Ok(n) = max.parse::<usize>() {
                config.max_concurrent = n;
            }
        }
        config
    }

    pub fn with_working_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.working_dir = Some(dir.as_ref().to_path_buf());
        self
    }
}

/// Handle to a spawned process.
pub struct ProcessHandle {
    pub(crate) child: Child,
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if let Err(e) = self.child.start_kill() {
            tracing::warn!("failed to kill child process on drop: {}", e);
        }
    }
}

impl ProcessHandle {
    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.child.stdout.take()
    }

    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.start_kill()
    }

    pub async fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait().await
    }

    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }
}

/// Spawn the configured command with prompt and optional session ID.
pub async fn spawn(
    config: &SpawnConfig,
    prompt: &str,
    session_id: Option<&str>,
) -> Result<ProcessHandle, SpawnError> {
    if config.command.is_empty() {
        return Err(SpawnError::CommandMissing);
    }

    let mut cmd = Command::new(&config.command);
    cmd.args(&config.args_before_prompt)
        .arg("-p")
        .arg(prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if let Some(ref dir) = config.working_dir {
        cmd.current_dir(Path::new(dir));
    }
    for key in &config.env_remove {
        cmd.env_remove(key);
    }
    for (k, v) in &config.env_set {
        cmd.env(k, v);
    }
    if let Some(sid) = session_id {
        cmd.arg("--resume").arg(sid);
    }

    let child = cmd.spawn()?;
    Ok(ProcessHandle { child })
}

/// Enforces max concurrent process spawns.
pub struct SpawnLimiter {
    semaphore: Arc<tokio::sync::Semaphore>,
    pub config: SpawnConfig,
}

impl SpawnLimiter {
    pub fn new(config: SpawnConfig, max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            config,
        }
    }

    pub async fn spawn_limited(
        &self,
        prompt: &str,
        session_id: Option<&str>,
    ) -> Result<(ProcessHandle, tokio::sync::OwnedSemaphorePermit), SpawnError> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| SpawnError::CommandMissing)?;
        let handle = spawn(&self.config, prompt, session_id).await?;
        Ok((handle, permit))
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}
