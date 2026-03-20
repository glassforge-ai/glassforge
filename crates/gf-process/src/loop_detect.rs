//! Loop detection for agent output streams.

use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ExitGateConfig {
    pub completion_patterns: Vec<String>,
    pub max_retries: u32,
    pub loop_detection_window: usize,
}

impl Default for ExitGateConfig {
    fn default() -> Self {
        Self {
            completion_patterns: vec![],
            max_retries: 2,
            loop_detection_window: 5,
        }
    }
}

pub struct LoopDetector {
    window: VecDeque<u64>,
    max_window: usize,
}

impl LoopDetector {
    pub fn new(max_window: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(max_window),
            max_window,
        }
    }

    pub fn push(&mut self, output: &str) -> bool {
        let hash = Self::hash_output(output);
        let is_repeat = self.window.contains(&hash);

        if self.window.len() >= self.max_window {
            self.window.pop_front();
        }
        self.window.push_back(hash);

        is_repeat
    }

    pub fn reset(&mut self) {
        self.window.clear();
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    fn hash_output(output: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        output.hash(&mut hasher);
        hasher.finish()
    }
}

pub fn check_completion_patterns(output: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|pattern| output.contains(pattern))
}

pub fn validate_exit(
    exit_code: i32,
    output: &str,
    config: &ExitGateConfig,
    detector: &mut LoopDetector,
) -> Result<(), String> {
    if detector.push(output) {
        return Err("loop detected: output matches previous output in window".to_string());
    }
    if exit_code != 0 {
        return Err(format!("non-zero exit code: {}", exit_code));
    }
    if !check_completion_patterns(output, &config.completion_patterns) {
        return Err("output missing required completion pattern".to_string());
    }
    Ok(())
}
