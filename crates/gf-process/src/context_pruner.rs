//! Context pruner -- summarizes and truncates messages to fit within a token budget.

pub struct ContextPruner;

impl ContextPruner {
    pub fn summarize_tool_call(tool_name: &str, args: &str, result: &str) -> String {
        let args_preview = if args.len() > 50 {
            format!("{}...", &args[..50])
        } else {
            args.to_string()
        };

        let result_preview = if result.len() > 100 {
            format!("{}...", &result[..100])
        } else {
            result.to_string()
        };

        format!("[{}] {} -> {}", tool_name, args_preview, result_preview)
    }

    pub fn truncate_text(text: &str, max_chars: usize) -> String {
        if text.len() <= max_chars {
            return text.to_string();
        }

        let marker = "\n[...truncated...]\n";
        let available = max_chars.saturating_sub(marker.len());
        let head_len = (available * 80) / 100;
        let tail_len = available - head_len;

        let head = &text[..head_len];
        let tail = &text[text.len() - tail_len..];

        format!("{}{}{}", head, marker, tail)
    }

    pub fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(3)
    }

    pub fn prune_messages(messages: &[String], max_tokens: usize) -> Vec<String> {
        let mut result: Vec<String> = messages.to_vec();

        let total: usize = result.iter().map(|m| Self::estimate_tokens(m)).sum();
        if total <= max_tokens {
            return result;
        }

        let keep_count = result.len() / 2;
        let summarize_count = result.len() - keep_count;

        if summarize_count == 0 {
            return result;
        }

        let mut target_len = 80usize;
        loop {
            let suffix = "...";
            for msg in result.iter_mut().take(summarize_count) {
                if msg.len() > target_len + suffix.len() {
                    *msg = format!("{}{}", &msg[..target_len], suffix);
                }
            }

            let current: usize = result.iter().map(|m| Self::estimate_tokens(m)).sum();
            if current <= max_tokens {
                return result;
            }

            if target_len <= 3 {
                break;
            }
            target_len /= 2;
        }

        while result.len() > 1 {
            result.remove(0);
            let current: usize = result.iter().map(|m| Self::estimate_tokens(m)).sum();
            if current <= max_tokens {
                return result;
            }
        }

        result
    }
}
