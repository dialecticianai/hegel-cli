use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Raw hook event from hooks.jsonl
///
/// NOTE: We're capturing aggressively to understand full schema.
/// Some fields may not be needed long-term (see TODO comments).
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
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// File modification from Edit/Write tools
#[derive(Debug, Clone)]
pub struct FileModification {
    pub file_path: String,
    pub tool: String, // "Edit" or "Write"
    pub timestamp: Option<String>,
}

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

/// Transcript event (from Claude Code session transcript)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEvent {
    #[serde(rename = "type")]
    pub event_type: String, // "assistant", "user", "system", "file-history-snapshot"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    // Catch-all for other fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// State transition event from states.jsonl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionEvent {
    pub timestamp: String,
    pub workflow_id: Option<String>,
    pub from_node: String,
    pub to_node: String,
    pub phase: String,
    pub mode: String,
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

/// Aggregated token metrics from transcript
#[derive(Debug, Default)]
pub struct TokenMetrics {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub assistant_turns: usize,
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
pub fn parse_hooks_file<P: AsRef<Path>>(hooks_path: P) -> Result<HookMetrics> {
    let content = fs::read_to_string(hooks_path.as_ref())
        .with_context(|| format!("Failed to read hooks file: {:?}", hooks_path.as_ref()))?;

    let mut metrics = HookMetrics::default();

    for (line_num, line) in content.lines().enumerate() {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        let event: HookEvent = match serde_json::from_str(line) {
            Ok(event) => event,
            Err(e) => {
                // Skip malformed lines with warning (e.g., concatenated JSON)
                eprintln!(
                    "Warning: Skipping malformed hook event at line {}: {}",
                    line_num + 1,
                    e
                );
                continue;
            }
        };

        metrics.total_events += 1;

        // Track session boundaries
        if event.hook_event_name == "SessionStart" {
            metrics.session_start_time = event.timestamp.clone();
        }
        if event.hook_event_name == "Stop" {
            metrics.session_end_time = event.timestamp.clone();
        }

        // Extract bash commands (PostToolUse only)
        if event.hook_event_name == "PostToolUse" && event.tool_name.as_deref() == Some("Bash") {
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
                        timestamp: event.timestamp.clone(),
                        stdout,
                        stderr,
                    });
                }
            }
        }

        // Extract file modifications (Edit/Write tools)
        if event.hook_event_name == "PostToolUse" {
            if let Some(tool_name) = &event.tool_name {
                if tool_name == "Edit" || tool_name == "Write" {
                    if let Some(tool_input) = &event.tool_input {
                        if let Some(file_path) =
                            tool_input.get("file_path").and_then(|v| v.as_str())
                        {
                            metrics.file_modifications.push(FileModification {
                                file_path: file_path.to_string(),
                                tool: tool_name.clone(),
                                timestamp: event.timestamp.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(metrics)
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
            if let Some(usage) = event.usage {
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

/// Parse states.jsonl and extract state transitions
pub fn parse_states_file<P: AsRef<Path>>(states_path: P) -> Result<Vec<StateTransitionEvent>> {
    let content = fs::read_to_string(states_path.as_ref())
        .with_context(|| format!("Failed to read states file: {:?}", states_path.as_ref()))?;

    let mut transitions = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let event: StateTransitionEvent = serde_json::from_str(line).with_context(|| {
            format!("Failed to parse state transition at line {}", line_num + 1)
        })?;
        transitions.push(event);
    }

    Ok(transitions)
}

/// Unified metrics combining all data sources
#[derive(Debug, Default)]
pub struct UnifiedMetrics {
    pub hook_metrics: HookMetrics,
    pub token_metrics: TokenMetrics,
    pub state_transitions: Vec<StateTransitionEvent>,
    pub session_id: Option<String>,
}

/// Parse all available metrics from .hegel directory
pub fn parse_unified_metrics<P: AsRef<Path>>(state_dir: P) -> Result<UnifiedMetrics> {
    let state_dir = state_dir.as_ref();
    let hooks_path = state_dir.join("hooks.jsonl");
    let states_path = state_dir.join("states.jsonl");

    let mut unified = UnifiedMetrics::default();

    // Parse hooks if available
    if hooks_path.exists() {
        unified.hook_metrics = parse_hooks_file(&hooks_path)?;

        // Extract session_id and transcript_path from first hook event
        let content = fs::read_to_string(&hooks_path)?;
        if let Some(first_line) = content.lines().next() {
            let event: HookEvent = serde_json::from_str(first_line)?;
            unified.session_id = Some(event.session_id.clone());

            // Parse transcript if we have a path
            if let Some(transcript_path) = event.transcript_path {
                if Path::new(&transcript_path).exists() {
                    unified.token_metrics = parse_transcript_file(&transcript_path)?;
                }
            }
        }
    }

    // Parse states if available
    if states_path.exists() {
        unified.state_transitions = parse_states_file(&states_path)?;
    }

    Ok(unified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_hooks_file(events: &[&str]) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let hooks_path = temp_dir.path().join("hooks.jsonl");
        let content = events.join("\n");
        fs::write(&hooks_path, content).unwrap();
        (temp_dir, hooks_path)
    }

    #[test]
    fn test_parse_empty_file() {
        let (_temp_dir, hooks_path) = create_test_hooks_file(&[]);
        let metrics = parse_hooks_file(&hooks_path).unwrap();
        assert_eq!(metrics.total_events, 0);
        assert!(metrics.bash_commands.is_empty());
        assert!(metrics.file_modifications.is_empty());
    }

    #[test]
    fn test_parse_bash_command() {
        let events = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T00:00:00Z","tool_input":{"command":"cargo build"},"tool_response":{"stdout":"Compiling...","stderr":""}}"#,
        ];
        let (_temp_dir, hooks_path) = create_test_hooks_file(&events);
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
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T00:00:00Z","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write","timestamp":"2025-01-01T00:00:01Z","tool_input":{"file_path":"README.md"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_test_hooks_file(&events);
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
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo build"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_test_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        let freq = metrics.bash_command_frequency();
        assert_eq!(freq.get("cargo build"), Some(&2));
        assert_eq!(freq.get("cargo test"), Some(&1));
    }

    #[test]
    fn test_file_modification_frequency() {
        let events = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write","tool_input":{"file_path":"src/lib.rs"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","tool_input":{"file_path":"src/main.rs"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_test_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        let freq = metrics.file_modification_frequency();
        assert_eq!(freq.get("src/main.rs"), Some(&2));
        assert_eq!(freq.get("src/lib.rs"), Some(&1));
    }

    #[test]
    fn test_session_boundaries() {
        let events = vec![
            r#"{"session_id":"test","hook_event_name":"SessionStart","timestamp":"2025-01-01T00:00:00Z"}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"Stop","timestamp":"2025-01-01T01:00:00Z"}"#,
        ];
        let (_temp_dir, hooks_path) = create_test_hooks_file(&events);
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
            r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}"#,
        ];
        let (_temp_dir, hooks_path) = create_test_hooks_file(&events);
        let metrics = parse_hooks_file(&hooks_path).unwrap();

        // Only PostToolUse should be counted
        assert_eq!(metrics.bash_commands.len(), 1);
        assert_eq!(metrics.bash_commands[0].command, "cargo test");
    }

    // ========== Transcript Parser Tests ==========

    #[test]
    fn test_parse_transcript_token_usage() {
        let temp_dir = TempDir::new().unwrap();
        let transcript_path = temp_dir.path().join("transcript.jsonl");

        let events = vec![
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":200,"cache_read_input_tokens":300}}"#,
            r#"{"type":"user","content":"test message"}"#,
            r#"{"type":"assistant","usage":{"input_tokens":150,"output_tokens":75}}"#,
        ];

        fs::write(&transcript_path, events.join("\n")).unwrap();

        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.total_input_tokens, 250);
        assert_eq!(metrics.total_output_tokens, 125);
        assert_eq!(metrics.total_cache_creation_tokens, 200);
        assert_eq!(metrics.total_cache_read_tokens, 300);
        assert_eq!(metrics.assistant_turns, 2);
    }

    #[test]
    fn test_parse_transcript_skip_non_assistant() {
        let temp_dir = TempDir::new().unwrap();
        let transcript_path = temp_dir.path().join("transcript.jsonl");

        let events = vec![
            r#"{"type":"user","content":"hello"}"#,
            r#"{"type":"system","content":"system message"}"#,
            r#"{"type":"assistant","usage":{"input_tokens":100,"output_tokens":50}}"#,
        ];

        fs::write(&transcript_path, events.join("\n")).unwrap();

        let metrics = parse_transcript_file(&transcript_path).unwrap();

        assert_eq!(metrics.assistant_turns, 1);
        assert_eq!(metrics.total_input_tokens, 100);
    }

    // ========== States Parser Tests ==========

    #[test]
    fn test_parse_states_file() {
        let temp_dir = TempDir::new().unwrap();
        let states_path = temp_dir.path().join("states.jsonl");

        let events = vec![
            r#"{"timestamp":"2025-01-01T00:00:00Z","workflow_id":"wf-001","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T01:00:00Z","workflow_id":"wf-001","from_node":"plan","to_node":"code","phase":"code","mode":"discovery"}"#,
        ];

        fs::write(&states_path, events.join("\n")).unwrap();

        let transitions = parse_states_file(&states_path).unwrap();

        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].from_node, "spec");
        assert_eq!(transitions[0].to_node, "plan");
        assert_eq!(transitions[0].phase, "plan");
        assert_eq!(transitions[1].from_node, "plan");
        assert_eq!(transitions[1].to_node, "code");
    }

    #[test]
    fn test_parse_states_with_none_workflow_id() {
        let temp_dir = TempDir::new().unwrap();
        let states_path = temp_dir.path().join("states.jsonl");

        let event = r#"{"timestamp":"2025-01-01T00:00:00Z","workflow_id":null,"from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#;

        fs::write(&states_path, event).unwrap();

        let transitions = parse_states_file(&states_path).unwrap();

        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].workflow_id, None);
    }
}
