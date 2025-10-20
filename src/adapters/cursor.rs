use super::{AgentAdapter, CanonicalHookEvent, EventType};
use anyhow::Result;
use std::collections::HashMap;

/// Cursor adapter - normalizes Cursor hook events to canonical format
///
/// Cursor emits hooks configured in `~/.cursor/hooks.json` with events:
/// - beforeShellExecution / beforeMCPExecution - Pre-execution hooks
/// - afterFileEdit - Post-edit hooks
/// - beforeReadFile - Pre-read hooks (with redaction)
/// - beforeSubmitPrompt - Pre-submit hooks
/// - stop - Agent loop end
///
/// All events share common schema:
/// - conversation_id, generation_id, hook_event_name, workspace_roots
pub struct CursorAdapter;

impl CursorAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Map Cursor hook_event_name to EventType
    fn map_event_type(hook_event_name: &str) -> EventType {
        match hook_event_name {
            "beforeShellExecution" => EventType::PreToolUse,
            "beforeMCPExecution" => EventType::PreToolUse,
            "afterFileEdit" => EventType::PostToolUse,
            "beforeReadFile" => EventType::PreToolUse,
            "beforeSubmitPrompt" => EventType::PreToolUse,
            "stop" => EventType::Stop,
            other => EventType::Other(other.to_string()),
        }
    }

    /// Extract tool name from Cursor event
    fn extract_tool_name(hook_event_name: &str, input: &serde_json::Value) -> Option<String> {
        match hook_event_name {
            "beforeShellExecution" => Some("Bash".to_string()),
            "beforeMCPExecution" => input
                .get("tool_name")
                .and_then(|t| t.as_str())
                .map(String::from),
            "afterFileEdit" => Some("Edit".to_string()),
            "beforeReadFile" => Some("Read".to_string()),
            "beforeSubmitPrompt" => Some("SubmitPrompt".to_string()),
            _ => None,
        }
    }
}

impl AgentAdapter for CursorAdapter {
    fn name(&self) -> &str {
        "cursor"
    }

    fn detect(&self) -> bool {
        // Check for Cursor-specific env vars or config
        if std::env::var("CURSOR_SESSION_ID").is_ok() {
            return true;
        }

        // Check for ~/.cursor/hooks.json
        if let Ok(home) = std::env::var("HOME") {
            let cursor_config = std::path::PathBuf::from(home).join(".cursor/hooks.json");
            if cursor_config.exists() {
                return true;
            }
        }

        false
    }

    fn normalize(&self, input: serde_json::Value) -> Result<Option<CanonicalHookEvent>> {
        // Extract common fields
        let hook_event_name = input
            .get("hook_event_name")
            .and_then(|h| h.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'hook_event_name' field"))?;

        let conversation_id = input
            .get("conversation_id")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");

        let generation_id = input
            .get("generation_id")
            .and_then(|g| g.as_str())
            .unwrap_or("unknown");

        // Use generation_id as session_id (more granular than conversation_id)
        let session_id = generation_id.to_string();

        // Cursor doesn't provide timestamps in hook input, generate one
        let timestamp = chrono::Utc::now().to_rfc3339();

        let event_type = Self::map_event_type(hook_event_name);
        let tool_name = Self::extract_tool_name(hook_event_name, &input);

        // Extract tool input based on event type
        let tool_input = match hook_event_name {
            "beforeShellExecution" => Some(serde_json::json!({
                "command": input.get("command"),
                "cwd": input.get("cwd"),
            })),
            "beforeMCPExecution" => Some(serde_json::json!({
                "tool_name": input.get("tool_name"),
                "tool_input": input.get("tool_input"),
                "url": input.get("url"),
                "command": input.get("command"),
            })),
            "afterFileEdit" => Some(serde_json::json!({
                "file_path": input.get("file_path"),
                "edits": input.get("edits"),
            })),
            "beforeReadFile" => Some(serde_json::json!({
                "file_path": input.get("file_path"),
                "content": input.get("content"),
                "attachments": input.get("attachments"),
            })),
            "beforeSubmitPrompt" => Some(serde_json::json!({
                "prompt": input.get("prompt"),
                "attachments": input.get("attachments"),
            })),
            _ => None,
        };

        // Extract workspace roots and cwd
        let workspace_roots = input.get("workspace_roots");
        let cwd = input.get("cwd").and_then(|c| c.as_str()).map(String::from);

        // Build extra metadata
        let mut extra = HashMap::new();
        extra.insert(
            "conversation_id".to_string(),
            serde_json::json!(conversation_id),
        );
        if let Some(roots) = workspace_roots {
            extra.insert("workspace_roots".to_string(), roots.clone());
        }

        Ok(Some(CanonicalHookEvent {
            timestamp,
            session_id,
            event_type,
            tool_name,
            tool_input,
            tool_response: None, // Cursor hooks are pre/post without response data
            cwd,
            transcript_path: None,
            adapter: Some("cursor".to_string()),
            fallback_used: None,
            extra,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_with_env_var() {
        std::env::set_var("CURSOR_SESSION_ID", "test");
        let adapter = CursorAdapter::new();
        assert!(adapter.detect());
        std::env::remove_var("CURSOR_SESSION_ID");
    }

    #[test]
    fn test_map_event_type() {
        assert_eq!(
            CursorAdapter::map_event_type("beforeShellExecution"),
            EventType::PreToolUse
        );
        assert_eq!(
            CursorAdapter::map_event_type("afterFileEdit"),
            EventType::PostToolUse
        );
        assert_eq!(CursorAdapter::map_event_type("stop"), EventType::Stop);
    }

    #[test]
    fn test_normalize_before_shell_execution() {
        let adapter = CursorAdapter::new();
        let input = serde_json::json!({
            "conversation_id": "conv-123",
            "generation_id": "gen-456",
            "hook_event_name": "beforeShellExecution",
            "workspace_roots": ["/home/user/project"],
            "command": "cargo test",
            "cwd": "/home/user/project"
        });

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.session_id, "gen-456");
        assert_eq!(event.event_type, EventType::PreToolUse);
        assert_eq!(event.tool_name, Some("Bash".to_string()));
        assert_eq!(event.adapter, Some("cursor".to_string()));
        assert_eq!(event.cwd, Some("/home/user/project".to_string()));

        let tool_input = event.tool_input.unwrap();
        assert_eq!(
            tool_input.get("command").unwrap().as_str(),
            Some("cargo test")
        );
    }

    #[test]
    fn test_normalize_after_file_edit() {
        let adapter = CursorAdapter::new();
        let input = serde_json::json!({
            "conversation_id": "conv-123",
            "generation_id": "gen-456",
            "hook_event_name": "afterFileEdit",
            "workspace_roots": ["/home/user/project"],
            "file_path": "/home/user/project/src/main.rs",
            "edits": [
                {
                    "old_string": "fn main() {}",
                    "new_string": "fn main() { println!(\"Hello\"); }"
                }
            ]
        });

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.event_type, EventType::PostToolUse);
        assert_eq!(event.tool_name, Some("Edit".to_string()));

        let tool_input = event.tool_input.unwrap();
        assert_eq!(
            tool_input.get("file_path").unwrap().as_str(),
            Some("/home/user/project/src/main.rs")
        );
        assert!(tool_input.get("edits").unwrap().is_array());
    }

    #[test]
    fn test_normalize_before_mcp_execution() {
        let adapter = CursorAdapter::new();
        let input = serde_json::json!({
            "conversation_id": "conv-123",
            "generation_id": "gen-456",
            "hook_event_name": "beforeMCPExecution",
            "workspace_roots": ["/home/user/project"],
            "tool_name": "git_status",
            "tool_input": "{}",
            "url": "http://localhost:3000"
        });

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.event_type, EventType::PreToolUse);
        assert_eq!(event.tool_name, Some("git_status".to_string()));

        let tool_input = event.tool_input.unwrap();
        assert_eq!(
            tool_input.get("tool_name").unwrap().as_str(),
            Some("git_status")
        );
    }

    #[test]
    fn test_normalize_stop() {
        let adapter = CursorAdapter::new();
        let input = serde_json::json!({
            "conversation_id": "conv-123",
            "generation_id": "gen-456",
            "hook_event_name": "stop",
            "workspace_roots": ["/home/user/project"],
            "status": "completed"
        });

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.event_type, EventType::Stop);
        assert_eq!(event.tool_name, None);
    }

    #[test]
    fn test_normalize_before_read_file() {
        let adapter = CursorAdapter::new();
        let input = serde_json::json!({
            "conversation_id": "conv-123",
            "generation_id": "gen-456",
            "hook_event_name": "beforeReadFile",
            "workspace_roots": ["/home/user/project"],
            "file_path": "/home/user/project/.env",
            "content": "SECRET_KEY=xyz",
            "attachments": [
                {
                    "type": "rule",
                    "file_path": "/home/user/project/.cursorrules"
                }
            ]
        });

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.event_type, EventType::PreToolUse);
        assert_eq!(event.tool_name, Some("Read".to_string()));

        let tool_input = event.tool_input.unwrap();
        assert_eq!(
            tool_input.get("file_path").unwrap().as_str(),
            Some("/home/user/project/.env")
        );
        assert!(tool_input.get("attachments").unwrap().is_array());
    }
}
