use super::super::*;
use crate::test_helpers::*;
use tempfile::TempDir;

#[test]
fn test_phase_metrics_empty_workflow() {
    // No states.jsonl = no phases
    let temp_dir = TempDir::new().unwrap();
    let metrics = parse_unified_metrics(temp_dir.path(), false, None).unwrap();

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

    let metrics = parse_unified_metrics(temp_dir.path(), false, None).unwrap();

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

    let metrics = parse_unified_metrics(temp_dir.path(), false, None).unwrap();

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

    let metrics = parse_unified_metrics(temp_dir.path(), false, None).unwrap();

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

    // Create state.json with session_metadata
    use crate::storage::{FileStorage, SessionMetadata, State};
    let storage = FileStorage::new(temp_dir.path()).unwrap();
    let session = SessionMetadata {
        session_id: "test".to_string(),
        transcript_path: transcript_path.display().to_string(),
        started_at: "2025-01-01T10:00:00Z".to_string(),
    };
    let state = State {
        workflow: None,
        session_metadata: Some(session),
        cumulative_totals: None,
        git_info: None,
    };
    storage.save(&state).unwrap();

    let metrics = parse_unified_metrics(temp_dir.path(), false, None).unwrap();

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
fn test_fallback_to_hooks_jsonl_when_no_state_session_metadata() {
    // Test backward compatibility: if state.json has no session_metadata,
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
    let metrics = parse_unified_metrics(temp_dir.path(), false, None).unwrap();

    // Verify session metadata was loaded from hooks.jsonl
    assert_eq!(metrics.session_id, Some("fallback-test".to_string()));

    // Verify token metrics were loaded from transcript
    assert_eq!(metrics.token_metrics.total_input_tokens, 500);
    assert_eq!(metrics.token_metrics.total_output_tokens, 250);
    assert_eq!(metrics.token_metrics.assistant_turns, 1);
}

#[test]
fn test_parse_metrics_with_archives() {
    use crate::storage::archive::{
        write_archive, PhaseArchive, TokenTotals, TransitionArchive, WorkflowArchive,
        WorkflowTotals,
    };

    let temp_dir = TempDir::new().unwrap();

    // Create an archived workflow
    let archive = WorkflowArchive {
        workflow_id: "2025-10-24T10:00:00Z".to_string(),
        mode: "discovery".to_string(),
        completed_at: "2025-10-24T12:00:00Z".to_string(),
        session_id: Some("archived-session".to_string()),
        is_synthetic: false,
        phases: vec![PhaseArchive {
            phase_name: "spec".to_string(),
            start_time: "2025-10-24T10:00:00Z".to_string(),
            end_time: Some("2025-10-24T10:15:00Z".to_string()),
            duration_seconds: 900,
            tokens: TokenTotals {
                input: 1000,
                output: 500,
                cache_creation: 200,
                cache_read: 300,
                assistant_turns: 5,
            },
            bash_commands: vec![],
            file_modifications: vec![],
            git_commits: vec![],
        }],
        transitions: vec![TransitionArchive {
            from_node: "START".to_string(),
            to_node: "spec".to_string(),
            timestamp: "2025-10-24T10:00:00Z".to_string(),
        }],
        totals: WorkflowTotals {
            tokens: TokenTotals {
                input: 1000,
                output: 500,
                cache_creation: 200,
                cache_read: 300,
                assistant_turns: 5,
            },
            bash_commands: 0,
            file_modifications: 0,
            unique_files: 0,
            unique_commands: 0,
            git_commits: 0,
        },
    };

    write_archive(&archive, temp_dir.path()).unwrap();

    // Parse metrics - should include archived workflow
    let metrics = parse_unified_metrics(temp_dir.path(), true, None).unwrap();

    // Verify archived phase included
    assert_eq!(metrics.phase_metrics.len(), 1);
    assert_eq!(metrics.phase_metrics[0].phase_name, "spec");
    assert_eq!(
        metrics.phase_metrics[0].token_metrics.total_input_tokens,
        1000
    );

    // Verify archived tokens aggregated
    assert_eq!(metrics.token_metrics.total_input_tokens, 1000);
    assert_eq!(metrics.token_metrics.total_output_tokens, 500);

    // Verify archived transitions included
    assert_eq!(metrics.state_transitions.len(), 1);
    assert_eq!(metrics.state_transitions[0].from_node, "START");
}

#[test]
fn test_parse_metrics_with_multiple_archives() {
    use crate::storage::archive::{
        write_archive, PhaseArchive, TokenTotals, TransitionArchive, WorkflowArchive,
        WorkflowTotals,
    };

    let temp_dir = TempDir::new().unwrap();

    // Create 2 archived workflows
    for (i, workflow_id) in ["2025-10-24T10:00:00Z", "2025-10-24T14:00:00Z"]
        .iter()
        .enumerate()
    {
        let archive = WorkflowArchive {
            workflow_id: workflow_id.to_string(),
            mode: "discovery".to_string(),
            completed_at: format!("2025-10-24T{}:00:00Z", 12 + i * 4),
            session_id: None,
            is_synthetic: false,
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: workflow_id.to_string(),
                end_time: Some(format!("2025-10-24T{}:15:00Z", 10 + i * 4)),
                duration_seconds: 900,
                tokens: TokenTotals {
                    input: 1000,
                    output: 500,
                    cache_creation: 0,
                    cache_read: 0,
                    assistant_turns: 5,
                },
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: workflow_id.to_string(),
            }],
            totals: WorkflowTotals {
                tokens: TokenTotals {
                    input: 1000,
                    output: 500,
                    cache_creation: 0,
                    cache_read: 0,
                    assistant_turns: 5,
                },
                bash_commands: 0,
                file_modifications: 0,
                unique_files: 0,
                unique_commands: 0,
                git_commits: 0,
            },
        };

        write_archive(&archive, temp_dir.path()).unwrap();
    }

    // Parse metrics - should aggregate both archives
    let metrics = parse_unified_metrics(temp_dir.path(), true, None).unwrap();

    // Verify both phases included
    assert_eq!(metrics.phase_metrics.len(), 2);

    // Verify tokens aggregated across both workflows
    assert_eq!(metrics.token_metrics.total_input_tokens, 2000); // 1000 * 2
    assert_eq!(metrics.token_metrics.total_output_tokens, 1000); // 500 * 2
    assert_eq!(metrics.token_metrics.assistant_turns, 10); // 5 * 2

    // Verify all transitions included
    assert_eq!(metrics.state_transitions.len(), 2);
}

#[test]
fn test_parse_metrics_with_archive_and_live() {
    use crate::storage::archive::{
        write_archive, PhaseArchive, TokenTotals, TransitionArchive, WorkflowArchive,
        WorkflowTotals,
    };

    let temp_dir = TempDir::new().unwrap();

    // Create archived workflow
    let archive = WorkflowArchive {
        workflow_id: "2025-10-24T10:00:00Z".to_string(),
        mode: "discovery".to_string(),
        completed_at: "2025-10-24T12:00:00Z".to_string(),
        session_id: None,
        is_synthetic: false,
        phases: vec![PhaseArchive {
            phase_name: "spec".to_string(),
            start_time: "2025-10-24T10:00:00Z".to_string(),
            end_time: Some("2025-10-24T10:15:00Z".to_string()),
            duration_seconds: 900,
            tokens: TokenTotals {
                input: 1000,
                output: 500,
                cache_creation: 0,
                cache_read: 0,
                assistant_turns: 5,
            },
            bash_commands: vec![],
            file_modifications: vec![],
            git_commits: vec![],
        }],
        transitions: vec![TransitionArchive {
            from_node: "START".to_string(),
            to_node: "spec".to_string(),
            timestamp: "2025-10-24T10:00:00Z".to_string(),
        }],
        totals: WorkflowTotals {
            tokens: TokenTotals {
                input: 1000,
                output: 500,
                cache_creation: 0,
                cache_read: 0,
                assistant_turns: 5,
            },
            bash_commands: 0,
            file_modifications: 0,
            unique_files: 0,
            unique_commands: 0,
            git_commits: 0,
        },
    };

    write_archive(&archive, temp_dir.path()).unwrap();

    // Create live workflow state
    let states = vec![
        r#"{"timestamp":"2025-10-24T14:00:00Z","workflow_id":"2025-10-24T14:00:00Z","from_node":"START","to_node":"plan","phase":"plan","mode":"execution"}"#,
    ];
    let (_states_temp, states_path) = create_states_file(&states);
    std::fs::copy(&states_path, temp_dir.path().join("states.jsonl")).unwrap();

    // Parse metrics - should include archived + live
    let metrics = parse_unified_metrics(temp_dir.path(), true, None).unwrap();

    // Verify both phases included (1 archived + 1 live)
    assert_eq!(metrics.phase_metrics.len(), 2);
    assert_eq!(metrics.phase_metrics[0].phase_name, "spec"); // Archived
    assert_eq!(metrics.phase_metrics[1].phase_name, "plan"); // Live

    // Verify archived tokens included in total
    assert_eq!(metrics.token_metrics.total_input_tokens, 1000);
    assert_eq!(metrics.token_metrics.assistant_turns, 5);

    // Verify both transitions included
    assert_eq!(metrics.state_transitions.len(), 2);
}

#[test]
fn test_phase_metrics_default_git_commits() {
    // PhaseMetrics should have empty git_commits by default
    let phase = PhaseMetrics::default();
    assert!(phase.git_commits.is_empty());
}

#[test]
fn test_phase_metrics_with_git_commits() {
    // PhaseMetrics can hold git commits
    let mut phase = PhaseMetrics::default();
    phase.phase_name = "spec".to_string();

    let commit = GitCommit {
        hash: "abc1234".to_string(),
        timestamp: "2025-01-01T10:05:00Z".to_string(),
        message: "test commit".to_string(),
        author: "Test Author".to_string(),
        files_changed: 2,
        insertions: 10,
        deletions: 5,
    };

    phase.git_commits.push(commit.clone());

    assert_eq!(phase.git_commits.len(), 1);
    assert_eq!(phase.git_commits[0].hash, "abc1234");
}

#[test]
fn test_unified_metrics_default_git_commits() {
    // UnifiedMetrics should have empty git_commits by default
    let metrics = UnifiedMetrics::default();
    assert!(metrics.git_commits.is_empty());
}

#[test]
fn test_unified_metrics_with_git_commits() {
    // UnifiedMetrics can hold git commits
    let mut metrics = UnifiedMetrics::default();

    let commit = GitCommit {
        hash: "def5678".to_string(),
        timestamp: "2025-01-01T10:10:00Z".to_string(),
        message: "another commit".to_string(),
        author: "Another Author".to_string(),
        files_changed: 3,
        insertions: 15,
        deletions: 8,
    };

    metrics.git_commits.push(commit.clone());

    assert_eq!(metrics.git_commits.len(), 1);
    assert_eq!(metrics.git_commits[0].hash, "def5678");
}

#[test]
fn test_unified_metrics_serialization() {
    // Test that UnifiedMetrics can serialize to JSON
    let mut metrics = UnifiedMetrics::default();
    metrics.session_id = Some("test-session".to_string());
    metrics.token_metrics.total_input_tokens = 1000;
    metrics.token_metrics.total_output_tokens = 500;

    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("test-session"));
    assert!(json.contains("1000"));

    // Test deserialization
    let deserialized: UnifiedMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.session_id, Some("test-session".to_string()));
    assert_eq!(deserialized.token_metrics.total_input_tokens, 1000);
}

#[test]
fn test_phase_metrics_serialization() {
    let phase = PhaseMetrics {
        phase_name: "spec".to_string(),
        start_time: "2025-01-01T10:00:00Z".to_string(),
        end_time: Some("2025-01-01T10:15:00Z".to_string()),
        duration_seconds: 900,
        ..Default::default()
    };

    let json = serde_json::to_string(&phase).unwrap();
    assert!(json.contains("spec"));
    assert!(json.contains("900"));

    let deserialized: PhaseMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.phase_name, "spec");
    assert_eq!(deserialized.duration_seconds, 900);
}
