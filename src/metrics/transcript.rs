use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

/// Aggregated token metrics from transcript
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub assistant_turns: usize,
}

/// Parse transcript file and extract token metrics
pub fn parse_transcript_file<P: AsRef<Path>>(transcript_path: P) -> Result<TokenMetrics> {
    let content = fs::read_to_string(transcript_path.as_ref()).with_context(|| {
        format!(
            "Failed to read transcript file: {:?}",
            transcript_path.as_ref()
        )
    })?;

    let mut metrics = TokenMetrics::default();

    for (line_num, line) in content.lines().enumerate() {
        let event: TranscriptEvent = serde_json::from_str(line).with_context(|| {
            format!("Failed to parse transcript event at line {}", line_num + 1)
        })?;

        // Only assistant events have token usage
        if event.event_type == "assistant" {
            // Try both old format (.usage) and new format (.message.usage)
            // Claude Code changed schema between versions, handle both for resilience
            let usage = event
                .usage
                .or_else(|| event.message.as_ref().and_then(|m| m.usage.clone()));

            if let Some(usage) = usage {
                metrics.total_input_tokens += usage.input_tokens;
                metrics.total_output_tokens += usage.output_tokens;
                metrics.total_cache_creation_tokens +=
                    usage.cache_creation_input_tokens.unwrap_or(0);
                metrics.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
                metrics.assistant_turns += 1;
            }
        }
    }

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_parse_transcript_token_usage() {
        let events = vec![
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":200,"cache_read_input_tokens":300}}"#,
            r#"{"type":"user","content":"test message"}"#,
            r#"{"type":"assistant","usage":{"input_tokens":150,"output_tokens":75}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.total_input_tokens, 250);
        assert_eq!(metrics.total_output_tokens, 125);
        assert_eq!(metrics.total_cache_creation_tokens, 200);
        assert_eq!(metrics.total_cache_read_tokens, 300);
        assert_eq!(metrics.assistant_turns, 2);
    }

    #[test]
    fn test_parse_transcript_skip_non_assistant() {
        let events = vec![
            r#"{"type":"user","content":"hello"}"#,
            r#"{"type":"system","content":"system message"}"#,
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.assistant_turns, 1);
        assert_eq!(metrics.total_input_tokens, 100);
    }

    #[test]
    fn test_parse_transcript_new_format_message_usage() {
        // New Claude Code format: token usage nested in message.usage
        let events = vec![
            r#"{"type":"assistant","message":{"usage":{"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":200,"cache_read_input_tokens":300}}}"#,
            r#"{"type":"user","content":"test message"}"#,
            r#"{"type":"assistant","message":{"usage":{"input_tokens":150,"output_tokens":75}}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.total_input_tokens, 250);
        assert_eq!(metrics.total_output_tokens, 125);
        assert_eq!(metrics.total_cache_creation_tokens, 200);
        assert_eq!(metrics.total_cache_read_tokens, 300);
        assert_eq!(metrics.assistant_turns, 2);
    }

    #[test]
    fn test_parse_transcript_mixed_format() {
        // Mix of old and new formats in same file (resilience test)
        let events = vec![
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50}}"#,
            r#"{"type":"assistant","message":{"usage":{"input_tokens":150,"output_tokens":75,"cache_creation_input_tokens":100}}}"#,
            r#"{"type":"user","content":"test"}"#,
            r#"{"type":"assistant","usage":{"input_tokens":200,"output_tokens":100}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.total_input_tokens, 450); // 100 + 150 + 200
        assert_eq!(metrics.total_output_tokens, 225); // 50 + 75 + 100
        assert_eq!(metrics.total_cache_creation_tokens, 100);
        assert_eq!(metrics.assistant_turns, 3);
    }

    #[test]
    fn test_parse_transcript_empty_file() {
        let events: Vec<&str> = vec![];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.assistant_turns, 0);
        assert_eq!(metrics.total_input_tokens, 0);
        assert_eq!(metrics.total_output_tokens, 0);
    }

    #[test]
    fn test_parse_transcript_file_not_found() {
        use std::path::PathBuf;
        let nonexistent = PathBuf::from("/nonexistent/path/transcript.jsonl");
        let result = parse_transcript_file(&nonexistent);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read transcript file"));
    }

    #[test]
    fn test_parse_transcript_malformed_json() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let transcript_path = temp_dir.path().join("transcript.jsonl");

        std::fs::write(&transcript_path, "not valid json\n").unwrap();

        let result = parse_transcript_file(&transcript_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse transcript event"));
    }

    #[test]
    fn test_parse_transcript_assistant_without_usage() {
        // Assistant event without usage field - should be skipped
        let events = vec![
            r#"{"type":"assistant","content":"Hello"}"#,
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        // Only one assistant turn should be counted (the one with usage)
        assert_eq!(metrics.assistant_turns, 1);
        assert_eq!(metrics.total_input_tokens, 100);
        assert_eq!(metrics.total_output_tokens, 50);
    }

    #[test]
    fn test_parse_transcript_with_timestamps() {
        // Events with timestamps (new field for phase correlation)
        let events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:00:00Z","usage":{"input_tokens":100,"output_tokens":50}}"#,
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":150,"output_tokens":75}}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.assistant_turns, 2);
        assert_eq!(metrics.total_input_tokens, 250);
        assert_eq!(metrics.total_output_tokens, 125);
    }

    #[test]
    fn test_parse_transcript_other_event_types() {
        // Test various event types beyond assistant/user/system
        let events = vec![
            r#"{"type":"file-history-snapshot","data":"..."}"#,
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50}}"#,
        ];
        let (_temp_dir, transcript_path) = create_transcript_file(&events);
        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.assistant_turns, 1);
        assert_eq!(metrics.total_input_tokens, 100);
    }
}
