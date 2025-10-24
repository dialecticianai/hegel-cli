use super::{AgentAdapter, CanonicalHookEvent, EventType};
use anyhow::{Context, Result};

/// Claude Code adapter - normalizes Claude Code hook events
pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse event_type from Claude Code hook_event_name
    fn parse_event_type(hook_event_name: &str) -> EventType {
        match hook_event_name {
            "SessionStart" => EventType::SessionStart,
            "SessionEnd" => EventType::SessionEnd,
            "PreToolUse" => EventType::PreToolUse,
            "PostToolUse" => EventType::PostToolUse,
            "Stop" => EventType::Stop,
            other => EventType::Other(other.to_string()),
        }
    }
}

impl AgentAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        "claude_code"
    }

    fn detect(&self) -> bool {
        // Claude Code sets various environment variables:
        // - CLAUDE_CODE_SESSION_ID (session-specific)
        // - CLAUDE_CODE_TRANSCRIPT_PATH (session-specific)
        // - CLAUDECODE=1 (always set)
        std::env::var("CLAUDE_CODE_SESSION_ID").is_ok()
            || std::env::var("CLAUDE_CODE_TRANSCRIPT_PATH").is_ok()
            || std::env::var("CLAUDECODE").is_ok()
            || {
                // Fallback: check if we're in a Claude Code environment by looking for typical paths
                let home = std::env::var("HOME").unwrap_or_default();
                let claude_paths = [
                    format!("{}/.config/claude", home),
                    format!("{}/.claude", home),
                ];
                claude_paths
                    .iter()
                    .any(|p| std::path::Path::new(p).exists())
            }
    }

    fn normalize(&self, input: serde_json::Value) -> Result<Option<CanonicalHookEvent>> {
        // Parse as Claude Code hook event
        let obj = input
            .as_object()
            .context("Expected JSON object for Claude Code event")?;

        // Extract required fields
        let session_id = obj
            .get("session_id")
            .and_then(|v| v.as_str())
            .context("Missing session_id field")?
            .to_string();

        let hook_event_name = obj
            .get("hook_event_name")
            .and_then(|v| v.as_str())
            .context("Missing hook_event_name field")?;

        let event_type = Self::parse_event_type(hook_event_name);

        // Extract optional timestamp (will be injected later if missing)
        let timestamp = obj
            .get("timestamp")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Extract tool-specific fields
        let tool_name = obj
            .get("tool_name")
            .and_then(|v| v.as_str())
            .map(String::from);

        let tool_input = obj.get("tool_input").cloned();
        let tool_response = obj.get("tool_response").cloned();

        // Extract context fields
        let cwd = obj.get("cwd").and_then(|v| v.as_str()).map(String::from);

        let transcript_path = obj
            .get("transcript_path")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Collect any extra fields
        let mut extra = std::collections::HashMap::new();
        for (key, value) in obj {
            match key.as_str() {
                "session_id" | "hook_event_name" | "timestamp" | "tool_name" | "tool_input"
                | "tool_response" | "cwd" | "transcript_path" => {
                    // Skip known fields
                }
                _ => {
                    extra.insert(key.clone(), value.clone());
                }
            }
        }

        // Build canonical event
        let canonical = CanonicalHookEvent {
            timestamp: timestamp.unwrap_or_else(|| {
                // Will be injected by hook command if missing
                String::new()
            }),
            session_id,
            event_type,
            tool_name,
            tool_input,
            tool_response,
            cwd,
            transcript_path,
            adapter: Some("claude_code".to_string()),
            fallback_used: None, // Claude Code events are well-structured
            extra,
        };

        Ok(Some(canonical))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_session_start() {
        let adapter = ClaudeCodeAdapter::new();
        let input = serde_json::json!({
            "session_id": "test-session-123",
            "hook_event_name": "SessionStart",
            "timestamp": "2025-01-01T00:00:00Z",
            "transcript_path": "/tmp/transcript.jsonl"
        });

        let result = adapter.normalize(input).unwrap().unwrap();

        assert_eq!(result.session_id, "test-session-123");
        assert!(matches!(result.event_type, EventType::SessionStart));
        assert_eq!(result.timestamp, "2025-01-01T00:00:00Z");
        assert_eq!(
            result.transcript_path,
            Some("/tmp/transcript.jsonl".to_string())
        );
        assert_eq!(result.adapter, Some("claude_code".to_string()));
    }

    #[test]
    fn test_normalize_post_tool_use_bash() {
        let adapter = ClaudeCodeAdapter::new();
        let input = serde_json::json!({
            "session_id": "test",
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "cargo build"},
            "tool_response": {"stdout": "Compiling...", "stderr": ""}
        });

        let result = adapter.normalize(input).unwrap().unwrap();

        assert_eq!(result.session_id, "test");
        assert!(matches!(result.event_type, EventType::PostToolUse));
        assert_eq!(result.tool_name, Some("Bash".to_string()));
        assert_eq!(
            result.tool_input.unwrap()["command"],
            serde_json::Value::String("cargo build".to_string())
        );
    }

    #[test]
    fn test_normalize_file_edit() {
        let adapter = ClaudeCodeAdapter::new();
        let input = serde_json::json!({
            "session_id": "test",
            "hook_event_name": "PostToolUse",
            "tool_name": "Edit",
            "tool_input": {"file_path": "src/main.rs", "old_string": "foo", "new_string": "bar"}
        });

        let result = adapter.normalize(input).unwrap().unwrap();

        assert_eq!(result.tool_name, Some("Edit".to_string()));
        let input = result.tool_input.unwrap();
        assert_eq!(input["file_path"], "src/main.rs");
        assert_eq!(input["old_string"], "foo");
        assert_eq!(input["new_string"], "bar");
    }

    #[test]
    fn test_parse_event_types() {
        let adapter = ClaudeCodeAdapter::new();

        let test_cases = vec![
            ("SessionStart", EventType::SessionStart),
            ("SessionEnd", EventType::SessionEnd),
            ("PreToolUse", EventType::PreToolUse),
            ("PostToolUse", EventType::PostToolUse),
            ("Stop", EventType::Stop),
        ];

        for (hook_name, expected) in test_cases {
            let input = serde_json::json!({
                "session_id": "test",
                "hook_event_name": hook_name
            });

            let result = adapter.normalize(input).unwrap().unwrap();
            assert_eq!(result.event_type, expected);
        }
    }

    #[test]
    fn test_normalize_missing_session_id_fails() {
        let adapter = ClaudeCodeAdapter::new();
        let input = serde_json::json!({
            "hook_event_name": "PostToolUse"
        });

        let result = adapter.normalize(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("session_id"));
    }

    #[test]
    fn test_normalize_missing_event_name_fails() {
        let adapter = ClaudeCodeAdapter::new();
        let input = serde_json::json!({
            "session_id": "test"
        });

        let result = adapter.normalize(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("hook_event_name"));
    }

    #[test]
    fn test_normalize_preserves_extra_fields() {
        let adapter = ClaudeCodeAdapter::new();
        let input = serde_json::json!({
            "session_id": "test",
            "hook_event_name": "PostToolUse",
            "custom_field": "custom_value",
            "another_field": 42
        });

        let result = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(result.extra.get("custom_field").unwrap(), "custom_value");
        assert_eq!(result.extra.get("another_field").unwrap(), 42);
    }
}
