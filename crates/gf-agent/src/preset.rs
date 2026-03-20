//! Agent presets with default system prompts and tool allowlists.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPreset {
    CodeWriter,
    Reviewer,
    Tester,
    Debugger,
    Architect,
    Documenter,
    SecurityAuditor,
    Refactorer,
    Explorer,
    Coordinator,
}

pub struct PresetDefaults {
    pub system_prompt: String,
    pub model: String,
    pub allowed_tools: Option<Vec<String>>,
}

impl AgentPreset {
    pub fn defaults(&self) -> PresetDefaults {
        match self {
            Self::CodeWriter => PresetDefaults {
                system_prompt: "You are a senior software engineer. Write clean, well-tested, production-ready code. Follow existing patterns in the codebase. Include error handling. Write tests for new functionality.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Reviewer => PresetDefaults {
                system_prompt: "You are a code reviewer. Analyze code for bugs, security issues, performance problems, and style violations. Be specific about line numbers and suggest fixes. Check for edge cases and error handling gaps.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into()]),
            },
            Self::Tester => PresetDefaults {
                system_prompt: "You are a test engineer. Write comprehensive tests: unit tests, integration tests, and edge case tests. Use the project's existing test framework. Aim for meaningful coverage of business logic, not line count.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Debugger => PresetDefaults {
                system_prompt: "You are a debugging specialist. Systematically identify root causes. Read error messages carefully. Add targeted logging. Form hypotheses, test them, and narrow down. Fix the root cause, not symptoms.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Architect => PresetDefaults {
                system_prompt: "You are a software architect. Design systems for simplicity, maintainability, and correctness. Identify abstractions, define boundaries, and document trade-offs. Prefer boring technology over clever solutions.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into(), "WebSearch".into()]),
            },
            Self::Documenter => PresetDefaults {
                system_prompt: "You are a technical writer. Write clear, concise documentation. Include examples. Document the why, not just the what. Keep docs close to the code they describe. Use the project's documentation conventions.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::SecurityAuditor => PresetDefaults {
                system_prompt: "You are a security auditor. Check for OWASP Top 10 vulnerabilities: injection, broken auth, sensitive data exposure, XXE, broken access control, misconfig, XSS, insecure deserialization, known vulns, insufficient logging. Report severity and remediation.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into()]),
            },
            Self::Refactorer => PresetDefaults {
                system_prompt: "You are a refactoring specialist. Improve code structure without changing behavior. Apply SOLID principles where they reduce complexity. Extract when duplication is proven, not speculative. Ensure tests pass before and after.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
            Self::Explorer => PresetDefaults {
                system_prompt: "You are a codebase explorer. Navigate unfamiliar code quickly. Map dependencies, find entry points, trace execution flows. Summarize architecture and key patterns. Identify tech debt and improvement opportunities.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: Some(vec!["Read".into(), "Grep".into(), "Glob".into()]),
            },
            Self::Coordinator => PresetDefaults {
                system_prompt: "You are a task coordinator. Break down complex tasks into independent sub-tasks that can run in parallel. For each sub-task, specify: the agent type (CodeWriter, Tester, Reviewer, etc.), a clear prompt, and the working directory. After all sub-tasks complete, synthesize their outputs into a coherent final response.".into(),
                model: "claude-sonnet-4-20250514".into(),
                allowed_tools: None,
            },
        }
    }

    pub fn all() -> &'static [AgentPreset] {
        &[
            Self::CodeWriter,
            Self::Reviewer,
            Self::Tester,
            Self::Debugger,
            Self::Architect,
            Self::Documenter,
            Self::SecurityAuditor,
            Self::Refactorer,
            Self::Explorer,
            Self::Coordinator,
        ]
    }
}
