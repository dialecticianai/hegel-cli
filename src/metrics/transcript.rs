use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token usage from transcript events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
}

/// Message wrapper for new transcript format
/// Claude Code changed schema from .usage to .message.usage between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWrapper {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

/// Transcript event (from Claude Code session transcript)
///
/// SCHEMA RESILIENCE: Handles both old and new Claude Code transcript formats
/// - Old format: {"type":"assistant","usage":{...}}
/// - New format: {"type":"assistant","message":{"usage":{...}}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    #[serde(rename = "type")]
    pub event_type: String, // "assistant", "user", "system", "file-history-snapshot"

    // Timestamp for phase correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    // Old format: token usage directly on event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    // New format: token usage nested in message wrapper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<MessageWrapper>,

    // Catch-all for other fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl TranscriptEvent {
    /// Extract token usage from either old (`.usage`) or new (`.message.usage`)
    /// Claude Code transcript format. Old format takes precedence when both exist.
    pub fn usage(&self) -> Option<&TokenUsage> {
        self.usage
            .as_ref()
            .or_else(|| self.message.as_ref().and_then(|m| m.usage.as_ref()))
    }
}

/// Aggregated token metrics from transcript
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub assistant_turns: usize,
}

impl TokenMetrics {
    /// Accumulate one usage record, counting it as an assistant turn.
    pub fn accumulate(&mut self, usage: &TokenUsage) {
        self.total_input_tokens += usage.input_tokens;
        self.total_output_tokens += usage.output_tokens;
        self.total_cache_creation_tokens += usage.cache_creation_input_tokens.unwrap_or(0);
        self.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
        self.assistant_turns += 1;
    }
}

impl std::ops::AddAssign<&TokenMetrics> for TokenMetrics {
    fn add_assign(&mut self, other: &TokenMetrics) {
        self.total_input_tokens += other.total_input_tokens;
        self.total_output_tokens += other.total_output_tokens;
        self.total_cache_creation_tokens += other.total_cache_creation_tokens;
        self.total_cache_read_tokens += other.total_cache_read_tokens;
        self.assistant_turns += other.assistant_turns;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> TranscriptEvent {
        serde_json::from_str(line).unwrap()
    }

    #[test]
    fn test_usage_old_format() {
        let e = parse(
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":200,"cache_read_input_tokens":300}}"#,
        );
        let u = e.usage().expect("old-format usage extracted");
        assert_eq!(u.input_tokens, 100);
        assert_eq!(u.output_tokens, 50);
        assert_eq!(u.cache_creation_input_tokens, Some(200));
        assert_eq!(u.cache_read_input_tokens, Some(300));
    }

    #[test]
    fn test_usage_new_format_message_usage() {
        // New Claude Code format: token usage nested in message.usage
        let e = parse(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":150,"output_tokens":75}}}"#,
        );
        let u = e.usage().expect("new-format usage extracted");
        assert_eq!(u.input_tokens, 150);
        assert_eq!(u.output_tokens, 75);
    }

    #[test]
    fn test_usage_old_format_takes_precedence() {
        // Both present: old `.usage` wins over `.message.usage`
        let e = parse(
            r#"{"type":"assistant","usage":{"input_tokens":1,"output_tokens":2},"message":{"usage":{"input_tokens":999,"output_tokens":999}}}"#,
        );
        let u = e.usage().unwrap();
        assert_eq!(u.input_tokens, 1);
        assert_eq!(u.output_tokens, 2);
    }

    #[test]
    fn test_usage_absent() {
        assert!(parse(r#"{"type":"assistant","content":"Hello"}"#)
            .usage()
            .is_none());
        assert!(parse(r#"{"type":"user","content":"hi"}"#).usage().is_none());
    }

    #[test]
    fn test_accumulate_old_and_new_formats() {
        // Mix of old and new formats accumulate identically (resilience test)
        let events = [
            parse(r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50}}"#),
            parse(
                r#"{"type":"assistant","message":{"usage":{"input_tokens":150,"output_tokens":75,"cache_creation_input_tokens":100}}}"#,
            ),
            parse(r#"{"type":"assistant","usage":{"input_tokens":200,"output_tokens":100}}"#),
        ];
        let mut m = TokenMetrics::default();
        for e in &events {
            if let Some(u) = e.usage() {
                m.accumulate(u);
            }
        }
        assert_eq!(m.total_input_tokens, 450); // 100 + 150 + 200
        assert_eq!(m.total_output_tokens, 225); // 50 + 75 + 100
        assert_eq!(m.total_cache_creation_tokens, 100); // absent cache fields count as 0
        assert_eq!(m.assistant_turns, 3);
    }

    #[test]
    fn test_accumulate_optional_cache_fields_default_to_zero() {
        let u = TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        };
        let mut m = TokenMetrics::default();
        m.accumulate(&u);
        assert_eq!(m.total_cache_creation_tokens, 0);
        assert_eq!(m.total_cache_read_tokens, 0);
        assert_eq!(m.assistant_turns, 1);
    }

    #[test]
    fn test_add_assign_sums_all_fields() {
        let mut a = TokenMetrics {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_cache_creation_tokens: 10,
            total_cache_read_tokens: 5,
            assistant_turns: 2,
        };
        let b = TokenMetrics {
            total_input_tokens: 200,
            total_output_tokens: 100,
            total_cache_creation_tokens: 20,
            total_cache_read_tokens: 15,
            assistant_turns: 3,
        };
        a += &b;
        assert_eq!(a.total_input_tokens, 300);
        assert_eq!(a.total_output_tokens, 150);
        assert_eq!(a.total_cache_creation_tokens, 30);
        assert_eq!(a.total_cache_read_tokens, 20);
        assert_eq!(a.assistant_turns, 5);
    }
}
