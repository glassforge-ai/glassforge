use crate::task_type::TaskType;

pub struct SkillRouter {
    routes: Vec<(TaskType, Vec<String>)>,
}

impl SkillRouter {
    pub fn new() -> Self {
        Self {
            routes: vec![
                (TaskType::NewFeature, vec![
                    "brainstorming".into(),
                    "writing-plans".into(),
                    "test-driven-development".into(),
                    "subagent-driven-development".into(),
                ]),
                (TaskType::BugFix, vec![
                    "systematic-debugging".into(),
                    "test-driven-development".into(),
                ]),
                (TaskType::CodeReview, vec![
                    "code-review".into(),
                    "security-guidance".into(),
                ]),
                (TaskType::Refactor, vec![
                    "refactor".into(),
                    "verification-before-completion".into(),
                ]),
                (TaskType::Research, vec![
                    "explore".into(),
                    "deep-research".into(),
                ]),
            ],
        }
    }

    pub fn skills_for(&self, task_type: TaskType) -> Vec<String> {
        for (tt, skills) in &self.routes {
            if *tt == task_type {
                return skills.clone();
            }
        }
        Vec::new()
    }
}

impl Default for SkillRouter {
    fn default() -> Self {
        Self::new()
    }
}
