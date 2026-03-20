use crate::backend::*;
use crate::spawn::{ProcessHandle, SpawnConfig, SpawnError, spawn};
use std::future::Future;
use std::pin::Pin;

/// Claude CLI backend adapter.
pub struct ClaudeBackend {
    base_config: SpawnConfig,
}

impl ClaudeBackend {
    pub fn new() -> Self {
        Self {
            base_config: SpawnConfig::from_env(),
        }
    }

    pub fn with_config(config: SpawnConfig) -> Self {
        Self { base_config: config }
    }
}

impl Default for ClaudeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessBackend for ClaudeBackend {
    fn name(&self) -> &str {
        "claude"
    }

    fn health_check(&self) -> Pin<Box<dyn Future<Output = BackendHealth> + Send + '_>> {
        Box::pin(async {
            match tokio::process::Command::new(&self.base_config.command)
                .arg("--version")
                .output()
                .await
            {
                Ok(output) if output.status.success() => BackendHealth::Healthy,
                Ok(_) => BackendHealth::Degraded("claude --version returned non-zero".into()),
                Err(e) => BackendHealth::Unavailable(format!("claude not found: {}", e)),
            }
        })
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_streaming: true,
            supports_tools: true,
            supported_models: vec![
                "claude-opus-4-6".into(),
                "claude-sonnet-4-6".into(),
                "claude-haiku-4-5".into(),
            ],
        }
    }

    fn spawn(
        &self,
        config: &BackendSpawnConfig,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessHandle, SpawnError>> + Send + '_>> {
        let spawn_config = self.base_config.clone().with_working_dir(&config.working_dir);
        let prompt = config.prompt.clone();
        let resume = config.resume_session_id.clone();

        Box::pin(async move {
            spawn(&spawn_config, &prompt, resume.as_deref()).await
        })
    }
}
