//! Strategy types for best-of-N agent execution.

/// A single strategy that biases an agent's approach.
#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub system_prompt_suffix: String,
}

/// A set of strategies to run in parallel for best-of-N selection.
#[derive(Debug, Clone)]
pub struct StrategySet {
    pub strategies: Vec<Strategy>,
}

impl StrategySet {
    /// Returns the default set of three strategies.
    pub fn default_three() -> Self {
        Self {
            strategies: vec![
                Strategy {
                    name: "minimal_changes".to_string(),
                    system_prompt_suffix: concat!(
                        "Make the smallest possible change to solve the problem. ",
                        "Touch as few files as possible and prefer the simplest diff ",
                        "that correctly addresses the requirement."
                    )
                    .to_string(),
                },
                Strategy {
                    name: "modular_refactor".to_string(),
                    system_prompt_suffix: concat!(
                        "Favor clean abstractions and modular design. ",
                        "Extract reusable helpers where appropriate and ensure ",
                        "each function has a single clear responsibility."
                    )
                    .to_string(),
                },
                Strategy {
                    name: "thorough_with_tests".to_string(),
                    system_prompt_suffix: concat!(
                        "Provide a comprehensive solution with thorough test coverage. ",
                        "Add unit tests for new logic, handle edge cases explicitly, ",
                        "and document any non-obvious decisions."
                    )
                    .to_string(),
                },
            ],
        }
    }
}
