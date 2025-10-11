pub mod graph;
mod hooks;
mod states;
mod transcript;

// Re-export public types from submodules
pub use graph::WorkflowDAG;
pub use hooks::{parse_hooks_file, BashCommand, FileModification, HookEvent, HookMetrics};
pub use states::{parse_states_file, StateTransitionEvent};
pub use transcript::{parse_transcript_file, TokenMetrics, TranscriptEvent};

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::storage::FileStorage;

/// Metrics for a single workflow phase
#[derive(Debug, Default, Clone)]
pub struct PhaseMetrics {
    pub phase_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: u64,
    pub token_metrics: TokenMetrics,
    pub bash_commands: Vec<BashCommand>,
    pub file_modifications: Vec<FileModification>,
}

/// Unified metrics combining all data sources
#[derive(Debug, Default)]
pub struct UnifiedMetrics {
    pub hook_metrics: HookMetrics,
    pub token_metrics: TokenMetrics,
    pub state_transitions: Vec<StateTransitionEvent>,
    pub session_id: Option<String>,
    pub phase_metrics: Vec<PhaseMetrics>,
}

/// Parse all available metrics from .hegel directory
pub fn parse_unified_metrics<P: AsRef<Path>>(state_dir: P) -> Result<UnifiedMetrics> {
    let state_dir = state_dir.as_ref();
    let hooks_path = state_dir.join("hooks.jsonl");
    let states_path = state_dir.join("states.jsonl");

    let mut unified = UnifiedMetrics::default();

    // Parse hooks if available
    let mut transcript_path_opt: Option<String> = None;
    if hooks_path.exists() {
        unified.hook_metrics = parse_hooks_file(&hooks_path)?;
    }

    // Load current session metadata - try fast path first, fall back to scanning hooks.jsonl
    let storage = FileStorage::new(state_dir)?;
    if let Some(session) = storage.load_current_session()? {
        // Fast path: O(1) lookup from current_session.json
        unified.session_id = Some(session.session_id);

        // Parse transcript if the file exists
        if Path::new(&session.transcript_path).exists() {
            unified.token_metrics = parse_transcript_file(&session.transcript_path)?;
            transcript_path_opt = Some(session.transcript_path);
        }
    } else if hooks_path.exists() {
        // Fallback: O(n) scan of hooks.jsonl for backward compatibility
        // (handles sessions that started before current_session.json feature was deployed)
        let content = fs::read_to_string(&hooks_path)?;
        let mut last_session_start: Option<HookEvent> = None;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(event) = serde_json::from_str::<HookEvent>(line) {
                if event.hook_event_name == "SessionStart" {
                    last_session_start = Some(event);
                }
            }
        }

        if let Some(event) = last_session_start {
            unified.session_id = Some(event.session_id.clone());

            // Parse transcript if we have a path
            if let Some(transcript_path) = event.transcript_path {
                if Path::new(&transcript_path).exists() {
                    unified.token_metrics = parse_transcript_file(&transcript_path)?;
                    transcript_path_opt = Some(transcript_path);
                }
            }
        }
    }

    // Parse states if available
    if states_path.exists() {
        unified.state_transitions = parse_states_file(&states_path)?;
    }

    // Build phase metrics from state transitions
    unified.phase_metrics = build_phase_metrics(
        &unified.state_transitions,
        &unified.hook_metrics,
        transcript_path_opt.as_deref(),
    )?;

    Ok(unified)
}

/// Build per-phase metrics from state transitions, hooks, and transcript
fn build_phase_metrics(
    transitions: &[StateTransitionEvent],
    hook_metrics: &HookMetrics,
    transcript_path: Option<&str>,
) -> Result<Vec<PhaseMetrics>> {
    use chrono::DateTime;

    if transitions.is_empty() {
        return Ok(Vec::new());
    }

    let mut phases = Vec::new();

    // Build phase boundaries: each transition marks a phase
    for (i, transition) in transitions.iter().enumerate() {
        let phase_name = transition.phase.clone();
        let start_time = transition.timestamp.clone();
        let end_time = transitions.get(i + 1).map(|t| t.timestamp.clone());

        // Calculate duration
        let duration_seconds = if let Some(ref end) = end_time {
            let start = DateTime::parse_from_rfc3339(&start_time)?;
            let end = DateTime::parse_from_rfc3339(end)?;
            (end - start).num_seconds() as u64
        } else {
            0 // Active phase has no duration yet
        };

        // Bucket hooks by timestamp
        let bash_commands: Vec<BashCommand> = hook_metrics
            .bash_commands
            .iter()
            .filter(|cmd| {
                cmd.timestamp.as_ref().map_or(false, |ts| {
                    is_in_range(ts, &start_time, end_time.as_deref())
                })
            })
            .cloned()
            .collect();

        let file_modifications: Vec<FileModification> = hook_metrics
            .file_modifications
            .iter()
            .filter(|file_mod| {
                file_mod.timestamp.as_ref().map_or(false, |ts| {
                    is_in_range(ts, &start_time, end_time.as_deref())
                })
            })
            .cloned()
            .collect();

        // Aggregate tokens from transcript for this phase
        let token_metrics = if let Some(transcript_path) = transcript_path {
            aggregate_tokens_for_phase(transcript_path, &start_time, end_time.as_deref())?
        } else {
            TokenMetrics::default()
        };

        phases.push(PhaseMetrics {
            phase_name,
            start_time,
            end_time,
            duration_seconds,
            token_metrics,
            bash_commands,
            file_modifications,
        });
    }

    Ok(phases)
}

/// Check if timestamp is in range [start, end)
fn is_in_range(timestamp: &str, start: &str, end: Option<&str>) -> bool {
    if let (Ok(ts), Ok(start_ts)) = (
        chrono::DateTime::parse_from_rfc3339(timestamp),
        chrono::DateTime::parse_from_rfc3339(start),
    ) {
        let after_start = ts >= start_ts;
        let before_end = end.map_or(true, |e| {
            chrono::DateTime::parse_from_rfc3339(e).map_or(false, |end_ts| ts < end_ts)
        });
        after_start && before_end
    } else {
        false
    }
}

/// Aggregate token usage from transcript for a specific phase
fn aggregate_tokens_for_phase(
    transcript_path: &str,
    start_time: &str,
    end_time: Option<&str>,
) -> Result<TokenMetrics> {
    let content = fs::read_to_string(transcript_path)?;
    let mut metrics = TokenMetrics::default();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let event: TranscriptEvent = serde_json::from_str(line)?;

        // Only process assistant events
        if event.event_type != "assistant" {
            continue;
        }

        // Check if timestamp is in phase range
        if let Some(ref timestamp) = event.timestamp {
            if !is_in_range(timestamp, start_time, end_time) {
                continue;
            }
        } else {
            continue; // Skip events without timestamps
        }

        // Extract token usage
        let usage = event
            .usage
            .or_else(|| event.message.as_ref().and_then(|m| m.usage.clone()));

        if let Some(usage) = usage {
            metrics.total_input_tokens += usage.input_tokens;
            metrics.total_output_tokens += usage.output_tokens;
            metrics.total_cache_creation_tokens += usage.cache_creation_input_tokens.unwrap_or(0);
            metrics.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
            metrics.assistant_turns += 1;
        }
    }

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use tempfile::TempDir;

    #[test]
    fn test_phase_metrics_empty_workflow() {
        // No states.jsonl = no phases
        let temp_dir = TempDir::new().unwrap();
        let metrics = parse_unified_metrics(temp_dir.path()).unwrap();

        assert!(metrics.phase_metrics.is_empty());
    }

    #[test]
    fn test_phase_metrics_single_active_phase() {
        // Workflow started but no transitions yet = one active phase
        let temp_dir = TempDir::new().unwrap();

        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
        ];
        let (_states_temp, states_path) = create_states_file(&states);
        std::fs::copy(&states_path, temp_dir.path().join("states.jsonl")).unwrap();

        let metrics = parse_unified_metrics(temp_dir.path()).unwrap();

        assert_eq!(metrics.phase_metrics.len(), 1);
        assert_eq!(metrics.phase_metrics[0].phase_name, "spec");
        assert_eq!(metrics.phase_metrics[0].start_time, "2025-01-01T10:00:00Z");
        assert_eq!(metrics.phase_metrics[0].end_time, None); // Still active
    }

    #[test]
    fn test_phase_metrics_multiple_completed_phases() {
        // Multiple transitions = multiple completed phases
        let temp_dir = TempDir::new().unwrap();

        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:30:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"plan","to_node":"code","phase":"code","mode":"discovery"}"#,
        ];
        let (_states_temp, states_path) = create_states_file(&states);
        std::fs::copy(&states_path, temp_dir.path().join("states.jsonl")).unwrap();

        let metrics = parse_unified_metrics(temp_dir.path()).unwrap();

        assert_eq!(metrics.phase_metrics.len(), 3);

        // spec phase (completed)
        assert_eq!(metrics.phase_metrics[0].phase_name, "spec");
        assert_eq!(metrics.phase_metrics[0].start_time, "2025-01-01T10:00:00Z");
        assert_eq!(
            metrics.phase_metrics[0].end_time,
            Some("2025-01-01T10:15:00Z".to_string())
        );
        assert_eq!(metrics.phase_metrics[0].duration_seconds, 900); // 15 minutes

        // plan phase (completed)
        assert_eq!(metrics.phase_metrics[1].phase_name, "plan");
        assert_eq!(metrics.phase_metrics[1].start_time, "2025-01-01T10:15:00Z");
        assert_eq!(
            metrics.phase_metrics[1].end_time,
            Some("2025-01-01T10:30:00Z".to_string())
        );
        assert_eq!(metrics.phase_metrics[1].duration_seconds, 900); // 15 minutes

        // code phase (active)
        assert_eq!(metrics.phase_metrics[2].phase_name, "code");
        assert_eq!(metrics.phase_metrics[2].start_time, "2025-01-01T10:30:00Z");
        assert_eq!(metrics.phase_metrics[2].end_time, None);
    }

    #[test]
    fn test_phase_metrics_buckets_hooks_by_timestamp() {
        // Hooks should be bucketed into correct phases based on timestamps
        let temp_dir = TempDir::new().unwrap();

        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let (_states_temp, states_path) = create_states_file(&states);
        std::fs::copy(&states_path, temp_dir.path().join("states.jsonl")).unwrap();

        let hooks = vec![
            // spec phase hooks
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:10:00Z","tool_input":{"file_path":"spec.md"}}"#,
            // plan phase hooks
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:20:00Z","tool_input":{"command":"cargo test"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write","timestamp":"2025-01-01T10:25:00Z","tool_input":{"file_path":"plan.md"}}"#,
        ];
        let (_hooks_temp, hooks_path) = create_hooks_file(&hooks);
        std::fs::copy(&hooks_path, temp_dir.path().join("hooks.jsonl")).unwrap();

        let metrics = parse_unified_metrics(temp_dir.path()).unwrap();

        // spec phase should have 1 bash command, 1 file edit
        assert_eq!(metrics.phase_metrics[0].bash_commands.len(), 1);
        assert_eq!(
            metrics.phase_metrics[0].bash_commands[0].command,
            "cargo build"
        );
        assert_eq!(metrics.phase_metrics[0].file_modifications.len(), 1);
        assert_eq!(
            metrics.phase_metrics[0].file_modifications[0].file_path,
            "spec.md"
        );

        // plan phase should have 1 bash command, 1 file write
        assert_eq!(metrics.phase_metrics[1].bash_commands.len(), 1);
        assert_eq!(
            metrics.phase_metrics[1].bash_commands[0].command,
            "cargo test"
        );
        assert_eq!(metrics.phase_metrics[1].file_modifications.len(), 1);
        assert_eq!(
            metrics.phase_metrics[1].file_modifications[0].file_path,
            "plan.md"
        );
    }

    #[test]
    fn test_phase_metrics_aggregates_tokens_per_phase() {
        // Transcript events should be aggregated per phase by timestamp
        let temp_dir = TempDir::new().unwrap();

        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let (_states_temp, states_path) = create_states_file(&states);
        std::fs::copy(&states_path, temp_dir.path().join("states.jsonl")).unwrap();

        // Create transcript file
        let transcript_events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":100,"output_tokens":50}}}"#,
            r#"{"type":"assistant","timestamp":"2025-01-01T10:10:00Z","message":{"usage":{"input_tokens":200,"output_tokens":75}}}"#,
            r#"{"type":"assistant","timestamp":"2025-01-01T10:20:00Z","message":{"usage":{"input_tokens":150,"output_tokens":100}}}"#,
        ];
        let (_transcript_temp, transcript_path) = create_transcript_file(&transcript_events);

        // Create current_session.json with the transcript path
        use crate::storage::{FileStorage, SessionMetadata};
        let storage = FileStorage::new(temp_dir.path()).unwrap();
        let session = SessionMetadata {
            session_id: "test".to_string(),
            transcript_path: transcript_path.display().to_string(),
            started_at: "2025-01-01T10:00:00Z".to_string(),
        };
        storage.save_current_session(&session).unwrap();

        let metrics = parse_unified_metrics(temp_dir.path()).unwrap();

        // spec phase: 2 assistant turns (10:05, 10:10)
        assert_eq!(metrics.phase_metrics[0].token_metrics.assistant_turns, 2);
        assert_eq!(
            metrics.phase_metrics[0].token_metrics.total_input_tokens,
            300
        ); // 100 + 200
        assert_eq!(
            metrics.phase_metrics[0].token_metrics.total_output_tokens,
            125
        ); // 50 + 75

        // plan phase: 1 assistant turn (10:20)
        assert_eq!(metrics.phase_metrics[1].token_metrics.assistant_turns, 1);
        assert_eq!(
            metrics.phase_metrics[1].token_metrics.total_input_tokens,
            150
        );
        assert_eq!(
            metrics.phase_metrics[1].token_metrics.total_output_tokens,
            100
        );
    }

    #[test]
    fn test_fallback_to_hooks_jsonl_when_no_current_session() {
        // Test backward compatibility: if current_session.json doesn't exist,
        // should fall back to scanning hooks.jsonl
        let temp_dir = TempDir::new().unwrap();

        // Create transcript file
        let transcript_events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":500,"output_tokens":250}}}"#,
        ];
        let (_transcript_temp, transcript_path) = create_transcript_file(&transcript_events);

        // Create hooks.jsonl with SessionStart (but NO current_session.json)
        let hook_str = format!(
            r#"{{"session_id":"fallback-test","hook_event_name":"SessionStart","timestamp":"2025-01-01T10:00:00Z","transcript_path":"{}"}}"#,
            transcript_path.display()
        );
        let hooks = vec![hook_str.as_str()];
        let (_hooks_temp, hooks_path) = create_hooks_file(&hooks);
        std::fs::copy(&hooks_path, temp_dir.path().join("hooks.jsonl")).unwrap();

        // Parse metrics - should use fallback path
        let metrics = parse_unified_metrics(temp_dir.path()).unwrap();

        // Verify session metadata was loaded from hooks.jsonl
        assert_eq!(metrics.session_id, Some("fallback-test".to_string()));

        // Verify token metrics were loaded from transcript
        assert_eq!(metrics.token_metrics.total_input_tokens, 500);
        assert_eq!(metrics.token_metrics.total_output_tokens, 250);
        assert_eq!(metrics.token_metrics.assistant_turns, 1);
    }
}
