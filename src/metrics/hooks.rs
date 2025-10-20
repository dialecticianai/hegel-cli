use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Re-export canonical event from adapters
use crate::adapters::{CanonicalHookEvent, EventType};

/// Raw hook event from hooks.jsonl (LEGACY - for backward compatibility)
///
/// New code should use CanonicalHookEvent from adapters module.
/// This is kept only for reading old hooks.jsonl files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub session_id: String,
    pub hook_event_name: String,

    // Injected by our hook command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    // Tool-specific fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    // CRITICAL: transcript_path is needed to parse token usage from transcript files
    // Token data lives in message.usage (input_tokens, output_tokens, cache_*_input_tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,

    // TODO: Potentially unneeded field (captured for schema exploration)
    // - permission_mode: Probably not relevant for cycle detection or metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,

    // Catch-all for any other fields we haven't modeled
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Parsed bash command from tool_input
#[derive(Debug, Clone)]
pub struct BashCommand {
    pub command: String,
    pub timestamp: Option<String>,
    /// TODO: Use for detailed command output analysis
    #[allow(dead_code)]
    pub stdout: Option<String>,
    /// TODO: Use for detailed command output analysis
    #[allow(dead_code)]
    pub stderr: Option<String>,
}

/// File modification from Edit/Write tools
#[derive(Debug, Clone)]
pub struct FileModification {
    pub file_path: String,
    pub tool: String, // "Edit" or "Write"
    pub timestamp: Option<String>,
}

/// Aggregated metrics from hook events
#[derive(Debug, Default)]
pub struct HookMetrics {
    pub total_events: usize,
    pub bash_commands: Vec<BashCommand>,
    pub file_modifications: Vec<FileModification>,
    pub session_start_time: Option<String>,
    pub session_end_time: Option<String>,
}

impl HookMetrics {
    /// Get bash command frequency (command → count)
    pub fn bash_command_frequency(&self) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for cmd in &self.bash_commands {
            *freq.entry(cmd.command.clone()).or_insert(0) += 1;
        }
        freq
    }

    /// Get file modification frequency (file_path → count)
    pub fn file_modification_frequency(&self) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for file_mod in &self.file_modifications {
            *freq.entry(file_mod.file_path.clone()).or_insert(0) += 1;
        }
        freq
    }
}

/// Parse hooks.jsonl and extract metrics
///
/// Supports both new (CanonicalHookEvent) and legacy (HookEvent) formats
pub fn parse_hooks_file<P: AsRef<Path>>(hooks_path: P) -> Result<HookMetrics> {
    let content = fs::read_to_string(hooks_path.as_ref())
        .with_context(|| format!("Failed to read hooks file: {:?}", hooks_path.as_ref()))?;

    let mut metrics = HookMetrics::default();

    for (_line_num, line) in content.lines().enumerate() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Try parsing as canonical event first, fall back to legacy format
        let event: CanonicalHookEvent = match serde_json::from_str(line) {
            Ok(event) => event,
            Err(_) => {
                // Try legacy format
                if let Ok(legacy_event) = serde_json::from_str::<HookEvent>(line) {
                    // Convert to canonical format
                    convert_legacy_event(legacy_event)
                } else {
                    // Skip malformed lines silently
                    continue;
                }
            }
        };

        metrics.total_events += 1;

        // Track session boundaries
        if matches!(event.event_type, EventType::SessionStart) {
            metrics.session_start_time = Some(event.timestamp.clone());
        }
        if matches!(event.event_type, EventType::Stop) {
            metrics.session_end_time = Some(event.timestamp.clone());
        }

        // Extract bash commands (PostToolUse only)
        if matches!(event.event_type, EventType::PostToolUse)
            && event.tool_name.as_deref() == Some("Bash")
        {
            if let Some(tool_input) = &event.tool_input {
                if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
                    let stdout = event
                        .tool_response
                        .as_ref()
                        .and_then(|r| r.get("stdout"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let stderr = event
                        .tool_response
                        .as_ref()
                        .and_then(|r| r.get("stderr"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    metrics.bash_commands.push(BashCommand {
                        command: command.to_string(),
                        timestamp: Some(event.timestamp.clone()),
                        stdout,
                        stderr,
                    });
                }
            }
        }

        // Extract file modifications (Edit/Write tools)
        if matches!(event.event_type, EventType::PostToolUse) {
            if let Some(tool_name) = &event.tool_name {
                if tool_name == "Edit" || tool_name == "Write" {
                    if let Some(tool_input) = &event.tool_input {
                        if let Some(file_path) =
                            tool_input.get("file_path").and_then(|v| v.as_str())
                        {
                            metrics.file_modifications.push(FileModification {
                                file_path: file_path.to_string(),
                                tool: tool_name.clone(),
                                timestamp: Some(event.timestamp.clone()),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(metrics)
}

/// Convert legacy HookEvent to CanonicalHookEvent
fn convert_legacy_event(legacy: HookEvent) -> CanonicalHookEvent {
    let event_type = match legacy.hook_event_name.as_str() {
        "SessionStart" => EventType::SessionStart,
        "SessionEnd" => EventType::SessionEnd,
        "PreToolUse" => EventType::PreToolUse,
        "PostToolUse" => EventType::PostToolUse,
        "Stop" => EventType::Stop,
        other => EventType::Other(other.to_string()),
    };

    CanonicalHookEvent {
        timestamp: legacy.timestamp.unwrap_or_else(|| "unknown".to_string()),
        session_id: legacy.session_id,
        event_type,
        tool_name: legacy.tool_name,
        tool_input: legacy.tool_input,
        tool_response: legacy.tool_response,
        cwd: legacy.cwd,
        transcript_path: legacy.transcript_path,
        adapter: Some("legacy".to_string()),
        fallback_used: Some(true),
        extra: legacy.extra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_parse_empty_file() {
        let (_temp_dir, hooks_path) = create_hooks_file(&[]);
        let metrics = parse_hooks_file(&hooks_path).unwrap();
        assert_eq!(metrics.total_events, 0);
        assert!(metrics.bash_commands.is_empty());
        assert!(metrics.file_modifications.is_empty());
    }

    #[test]
    fn test_parse_bash_command() {
        let events = vec![
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo build"},"tool_response":{"stdout":"Compiling...","stderr":""}}"#,
        ];
        let (_temp_dir, hooks_path) = create_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        assert_eq!(metrics.bash_commands.len(), 1);
        assert_eq!(metrics.bash_commands[0].command, "cargo build");
        assert_eq!(
            metrics.bash_commands[0].stdout,
            Some("Compiling...".to_string())
        );
    }

    #[test]
    fn test_parse_file_modifications() {
        let events = vec![
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Edit","timestamp":"2025-01-01T00:00:00Z","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Write","timestamp":"2025-01-01T00:00:01Z","tool_input":{"file_path":"README.md"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        assert_eq!(metrics.file_modifications.len(), 2);
        assert_eq!(metrics.file_modifications[0].file_path, "src/main.rs");
        assert_eq!(metrics.file_modifications[0].tool, "Edit");
        assert_eq!(metrics.file_modifications[1].file_path, "README.md");
        assert_eq!(metrics.file_modifications[1].tool, "Write");
    }

    #[test]
    fn test_bash_command_frequency() {
        let events = vec![
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo test"}}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo build"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        let freq = metrics.bash_command_frequency();
        assert_eq!(freq.get("cargo build"), Some(&2));
        assert_eq!(freq.get("cargo test"), Some(&1));
    }

    #[test]
    fn test_file_modification_frequency() {
        let events = vec![
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Edit","timestamp":"2025-01-01T00:00:00Z","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Write","timestamp":"2025-01-01T00:00:00Z","tool_input":{"file_path":"src/lib.rs"}}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Edit","timestamp":"2025-01-01T00:00:00Z","tool_input":{"file_path":"src/main.rs"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        let freq = metrics.file_modification_frequency();
        assert_eq!(freq.get("src/main.rs"), Some(&2));
        assert_eq!(freq.get("src/lib.rs"), Some(&1));
    }

    #[test]
    fn test_session_boundaries() {
        let events = vec![
            r#"{"session_id":"test","event_type":"session_start","timestamp":"2025-01-01T00:00:00Z"}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","event_type":"stop","timestamp":"2025-01-01T01:00:00Z"}"#,
        ];
        let (_temp_dir, hooks_path) = create_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        assert_eq!(
            metrics.session_start_time,
            Some("2025-01-01T00:00:00Z".to_string())
        );
        assert_eq!(
            metrics.session_end_time,
            Some("2025-01-01T01:00:00Z".to_string())
        );
    }

    #[test]
    fn test_skip_pre_tool_use_for_commands() {
        let events = vec![
            r#"{"session_id":"test","event_type":"pre_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","event_type":"post_tool_use","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo test"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        // Only PostToolUse should be counted
        assert_eq!(metrics.bash_commands.len(), 1);
        assert_eq!(metrics.bash_commands[0].command, "cargo test");
    }
}
