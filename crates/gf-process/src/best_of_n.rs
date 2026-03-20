//! Best-of-N runner: executes the same prompt with multiple strategies
//! and selects the best result.

use std::sync::Arc;

use gf_core::event_bus::EventBus;
use gf_core::ids::{AgentId, SessionId};

use crate::concurrent::{ConcurrentRunner, SubTask, SubTaskResult};

#[derive(Debug, Clone)]
pub struct SelectionResult {
    pub chosen_index: usize,
    pub reason: String,
    pub improvements: Vec<String>,
}

pub struct BestOfNRunner {
    runner: ConcurrentRunner,
}

impl BestOfNRunner {
    pub fn new(event_bus: Arc<EventBus>, max_concurrent: usize) -> Self {
        Self {
            runner: ConcurrentRunner::new(event_bus, max_concurrent),
        }
    }

    pub async fn run_best_of_n(
        &self,
        parent_session_id: &SessionId,
        base_task: SubTask,
        strategies: &gf_agent::strategy::StrategySet,
    ) -> (Vec<SubTaskResult>, SelectionResult) {
        let tasks: Vec<SubTask> = strategies
            .strategies
            .iter()
            .map(|strategy| SubTask {
                agent_id: AgentId::new(),
                prompt: format!(
                    "{}\n\n[Strategy: {}] {}",
                    base_task.prompt, strategy.name, strategy.system_prompt_suffix
                ),
                working_dir: base_task.working_dir.clone(),
            })
            .collect();

        let results = self.runner.run_all(parent_session_id, tasks).await;
        let selection = select_best(&results);

        (results, selection)
    }
}

pub fn select_best(results: &[SubTaskResult]) -> SelectionResult {
    if results.is_empty() {
        return SelectionResult {
            chosen_index: 0,
            reason: "No results to compare".to_string(),
            improvements: vec!["Provide at least one strategy to evaluate".to_string()],
        };
    }

    let scores: Vec<i64> = results.iter().map(score_result).collect();

    let (chosen_index, &best_score) = scores
        .iter()
        .enumerate()
        .max_by_key(|(_, score)| *score)
        .unwrap();

    let mut improvements = Vec::new();

    if best_score <= 0 {
        improvements.push("All candidates scored poorly; consider revising the prompt".to_string());
    }

    if !results[chosen_index].success {
        improvements.push("The chosen result did not exit successfully".to_string());
    }

    let reason = format!(
        "Candidate {} selected with score {} (success={}, output_len={})",
        chosen_index,
        best_score,
        results[chosen_index].success,
        results[chosen_index].output.len(),
    );

    SelectionResult {
        chosen_index,
        reason,
        improvements,
    }
}

fn score_result(result: &SubTaskResult) -> i64 {
    let mut score: i64 = 0;

    if result.success {
        score += 10;
    }

    score += (result.output.len() / 100) as i64;

    let error_count = result.output.matches("error").count()
        + result.output.matches("Error").count();
    score -= (error_count as i64) * 5;

    let fail_count = result.output.matches("fail").count()
        + result.output.matches("Fail").count();
    score -= (fail_count as i64) * 3;

    score
}
