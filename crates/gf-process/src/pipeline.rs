//! Pipeline engine: sequential and fanout step execution.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::concurrent::{ConcurrentRunner, SubTask, SubTaskResult};
use crate::spawn::SpawnConfig;
use gf_core::event_bus::EventBus;
use gf_core::ids::{AgentId, SessionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PipelineStep {
    Sequential {
        agent_id: String,
        prompt_template: String,
    },
    Fanout {
        agent_ids: Vec<String>,
        prompt_template: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
}

pub struct StepResult {
    pub step_index: usize,
    pub outputs: Vec<SubTaskResult>,
    pub success: bool,
}

impl std::fmt::Debug for StepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StepResult")
            .field("step_index", &self.step_index)
            .field("outputs_count", &self.outputs.len())
            .field("success", &self.success)
            .finish()
    }
}

pub struct PipelineRunner {
    event_bus: Arc<EventBus>,
    max_concurrent: usize,
    spawn_config: SpawnConfig,
}

impl PipelineRunner {
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
        spawn_config: SpawnConfig,
    ) -> Self {
        Self {
            event_bus,
            max_concurrent,
            spawn_config,
        }
    }

    pub async fn run(
        &self,
        parent_session_id: &SessionId,
        pipeline: &Pipeline,
        initial_input: &str,
        working_dir: &str,
    ) -> Vec<StepResult> {
        let mut results = Vec::new();
        let mut current_input = initial_input.to_string();

        for (step_index, step) in pipeline.steps.iter().enumerate() {
            let tasks = self.build_tasks(step, &current_input, working_dir);

            let concurrency = match step {
                PipelineStep::Sequential { .. } => 1,
                PipelineStep::Fanout { .. } => self.max_concurrent,
            };

            let runner = ConcurrentRunner::with_spawn_config(
                Arc::clone(&self.event_bus),
                concurrency,
                self.spawn_config.clone(),
            );

            let sub_results = runner.run_all(parent_session_id, tasks).await;

            let success = sub_results.iter().all(|r| r.success);

            current_input = sub_results
                .iter()
                .map(|r| r.output.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            results.push(StepResult {
                step_index,
                outputs: sub_results,
                success,
            });

            if !success {
                break;
            }
        }

        results
    }

    fn build_tasks(
        &self,
        step: &PipelineStep,
        input: &str,
        working_dir: &str,
    ) -> Vec<SubTask> {
        match step {
            PipelineStep::Sequential {
                agent_id,
                prompt_template,
            } => {
                let prompt = prompt_template.replace("{input}", input);
                vec![SubTask {
                    agent_id: AgentId(uuid::Uuid::parse_str(agent_id).unwrap_or_else(|_| uuid::Uuid::new_v4())),
                    prompt,
                    working_dir: working_dir.to_string(),
                }]
            }
            PipelineStep::Fanout {
                agent_ids,
                prompt_template,
            } => {
                let prompt = prompt_template.replace("{input}", input);
                agent_ids
                    .iter()
                    .map(|aid| SubTask {
                        agent_id: AgentId(uuid::Uuid::parse_str(aid).unwrap_or_else(|_| uuid::Uuid::new_v4())),
                        prompt: prompt.clone(),
                        working_dir: working_dir.to_string(),
                    })
                    .collect()
            }
        }
    }
}
