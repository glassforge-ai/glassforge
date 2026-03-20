//! Concurrent sub-agent runner with semaphore-limited parallel spawning.

use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::spawn::{spawn, ProcessHandle, SpawnConfig};
use gf_core::event_bus::EventBus;
use gf_core::events::ForgeEvent;
use gf_core::ids::{AgentId, SessionId};

/// A sub-task to be executed concurrently.
pub struct SubTask {
    pub agent_id: AgentId,
    pub prompt: String,
    pub working_dir: String,
}

/// Result from a completed sub-task.
#[derive(Debug)]
pub struct SubTaskResult {
    pub agent_id: AgentId,
    pub session_id: SessionId,
    pub output: String,
    pub exit_code: i32,
    pub success: bool,
}

pub struct ConcurrentRunner {
    event_bus: Arc<EventBus>,
    max_concurrent: usize,
    spawn_config: SpawnConfig,
}

impl ConcurrentRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config: SpawnConfig::from_env(),
        }
    }

    pub fn with_spawn_config(
        event_bus: Arc<EventBus>,
        max_concurrent: usize,
        config: SpawnConfig,
    ) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config: config,
        }
    }

    pub async fn run_all(
        &self,
        parent_session_id: &SessionId,
        tasks: Vec<SubTask>,
    ) -> Vec<SubTaskResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for task in tasks {
            let sem = semaphore.clone();
            let event_bus = Arc::clone(&self.event_bus);
            let parent_sid = *parent_session_id;
            let config = if task.working_dir.is_empty() {
                self.spawn_config.clone()
            } else {
                self.spawn_config.clone().with_working_dir(&task.working_dir)
            };

            let _ = event_bus.emit_sync(ForgeEvent::SubAgentRequested {
                parent_session_id: parent_sid,
                sub_agent_id: task.agent_id,
                prompt: task.prompt.clone(),
                timestamp: chrono::Utc::now(),
            });

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                let session_id = SessionId::new();

                let _ = event_bus.emit_sync(ForgeEvent::SubAgentStarted {
                    parent_session_id: parent_sid,
                    sub_agent_id: task.agent_id,
                    session_id,
                    timestamp: chrono::Utc::now(),
                });

                match spawn(&config, &task.prompt, None).await {
                    Ok(mut proc_handle) => {
                        let output = collect_output(&mut proc_handle).await;
                        let status = proc_handle.wait().await;
                        let exit_code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
                        let success = exit_code == 0;

                        if success {
                            let _ = event_bus.emit_sync(ForgeEvent::SubAgentCompleted {
                                parent_session_id: parent_sid,
                                sub_agent_id: task.agent_id,
                                session_id,
                                timestamp: chrono::Utc::now(),
                            });
                        } else {
                            let _ = event_bus.emit_sync(ForgeEvent::SubAgentFailed {
                                parent_session_id: parent_sid,
                                sub_agent_id: task.agent_id,
                                error: format!("exit code {}", exit_code),
                                timestamp: chrono::Utc::now(),
                            });
                        }

                        SubTaskResult {
                            agent_id: task.agent_id,
                            session_id,
                            output,
                            exit_code,
                            success,
                        }
                    }
                    Err(e) => {
                        let _ = event_bus.emit_sync(ForgeEvent::SubAgentFailed {
                            parent_session_id: parent_sid,
                            sub_agent_id: task.agent_id,
                            error: e.to_string(),
                            timestamp: chrono::Utc::now(),
                        });

                        SubTaskResult {
                            agent_id: task.agent_id,
                            session_id,
                            output: String::new(),
                            exit_code: -1,
                            success: false,
                        }
                    }
                }
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        results
    }
}

async fn collect_output(handle: &mut ProcessHandle) -> String {
    use tokio::io::AsyncReadExt;
    let mut output = String::new();
    if let Some(mut stdout) = handle.take_stdout() {
        let _ = stdout.read_to_string(&mut output).await;
    }
    output
}

/// Aggregate sub-task results into a summary.
pub fn aggregate_results(results: &[SubTaskResult]) -> String {
    let total = results.len();
    let succeeded = results.iter().filter(|r| r.success).count();

    let mut summary = format!("## Sub-agent Results: {}/{} succeeded\n\n", succeeded, total);

    for (i, result) in results.iter().enumerate() {
        let status = if result.success { "OK" } else { "FAILED" };
        summary.push_str(&format!(
            "### Sub-task {} [{}]\n{}\n\n",
            i + 1,
            status,
            if result.output.is_empty() {
                "(no output)".to_string()
            } else {
                result.output.chars().take(2000).collect::<String>()
            }
        ));
    }

    summary
}
