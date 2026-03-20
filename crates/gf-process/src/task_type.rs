/// Task type classification for methodology routing.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    NewFeature,
    BugFix,
    CodeReview,
    Refactor,
    Research,
    General,
}

pub struct TaskTypeDetector {
    rules: Vec<(TaskType, Vec<&'static str>)>,
}

impl TaskTypeDetector {
    pub fn new() -> Self {
        Self {
            rules: vec![
                (
                    TaskType::NewFeature,
                    vec![
                        "add", "create", "implement", "build", "new feature",
                        "new endpoint", "introduce",
                    ],
                ),
                (
                    TaskType::BugFix,
                    vec![
                        "fix", "bug", "broken", "error", "crash", "failing",
                        "doesn't work", "not working", "500", "404", "regression",
                    ],
                ),
                (
                    TaskType::CodeReview,
                    vec![
                        "review", "check this", "look at this pr", "code review",
                        "pr review", "feedback on",
                    ],
                ),
                (
                    TaskType::Refactor,
                    vec![
                        "refactor", "clean up", "reorganize", "restructure",
                        "extract", "move to", "rename", "simplify",
                    ],
                ),
                (
                    TaskType::Research,
                    vec![
                        "how does", "explain", "what is", "understand", "explore",
                        "investigate", "why does", "documentation",
                    ],
                ),
            ],
        }
    }

    pub fn classify(&self, prompt: &str) -> TaskType {
        let lower = prompt.to_lowercase();
        let mut best_type = TaskType::General;
        let mut best_count = 0usize;
        let mut tied = false;

        for (task_type, keywords) in &self.rules {
            let count = keywords.iter().filter(|kw| lower.contains(*kw)).count();
            if count > best_count {
                best_count = count;
                best_type = *task_type;
                tied = false;
            } else if count == best_count && count > 0 {
                tied = true;
            }
        }

        if best_count == 0 || tied {
            TaskType::General
        } else {
            best_type
        }
    }
}

impl Default for TaskTypeDetector {
    fn default() -> Self {
        Self::new()
    }
}
