use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendHealth {
    Healthy,
    Degraded(String),
    Unavailable(String),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackendCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supported_models: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BackendSpawnConfig {
    pub prompt: String,
    pub working_dir: String,
    pub model: Option<String>,
    pub max_turns: Option<u32>,
    pub allowed_tools: Option<Vec<String>>,
    pub system_prompt: Option<String>,
    pub resume_session_id: Option<String>,
    pub env: HashMap<String, String>,
}

pub use crate::spawn::ProcessHandle;

/// Trait that all execution backends must implement.
pub trait ProcessBackend: Send + Sync {
    fn name(&self) -> &str;
    fn health_check(&self) -> Pin<Box<dyn Future<Output = BackendHealth> + Send + '_>>;
    fn capabilities(&self) -> BackendCapabilities;
    fn spawn(
        &self,
        config: &BackendSpawnConfig,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessHandle, crate::spawn::SpawnError>> + Send + '_>>;
}

/// Registry of available backends.
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn ProcessBackend>>,
    default_backend: String,
}

impl BackendRegistry {
    pub fn new(default_backend: &str) -> Self {
        Self {
            backends: HashMap::new(),
            default_backend: default_backend.to_string(),
        }
    }

    pub fn register(&mut self, backend: Box<dyn ProcessBackend>) {
        let name = backend.name().to_string();
        self.backends.insert(name, backend);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ProcessBackend> {
        self.backends.get(name).map(|b| b.as_ref())
    }

    pub fn default(&self) -> Option<&dyn ProcessBackend> {
        self.get(&self.default_backend)
    }

    pub fn available_backends(&self) -> Vec<&str> {
        self.backends.keys().map(|s| s.as_str()).collect()
    }

    pub fn list_backends(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    pub async fn health_check_all(&self) -> Vec<(String, BackendHealth)> {
        let mut results = Vec::new();
        for (name, backend) in &self.backends {
            let health = backend.health_check().await;
            results.push((name.clone(), health));
        }
        results
    }
}
