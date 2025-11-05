use anyhow::Result;

use crate::analyze::repair::repair_archives;
use crate::analyze::sections::*;
use crate::metrics::parse_unified_metrics;
use crate::storage::FileStorage;
use crate::theme::Theme;

pub fn analyze_metrics(
    storage: &FileStorage,
    export_dot: bool,
    fix_archives: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    // Handle archive repair if requested
    if fix_archives {
        return repair_archives(storage, dry_run || json, json);
    }

    // analyze command needs ALL metrics including archives
    let metrics = parse_unified_metrics(storage.state_dir(), true)?;

    // Export DOT format if requested
    if export_dot {
        render_workflow_graph_dot(&metrics)?;
        return Ok(());
    }

    // Otherwise, render full analysis
    println!("{}", Theme::header("=== Hegel Metrics Analysis ==="));
    println!();

    render_session(&metrics);
    render_tokens(&metrics);
    render_activity(&metrics);
    render_top_bash_commands(&metrics);
    render_command_output_summary(&metrics);
    render_top_file_modifications(&metrics);
    render_state_transitions(&metrics);
    render_phase_breakdown(&metrics.phase_metrics);
    render_workflow_graph(&metrics);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_analyze_empty_state() {
        // Empty state directory - should not error
        let (_temp_dir, storage) = test_storage_with_files(None, None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_session_data() {
        // State with session ID and token metrics
        let hooks = vec![
            r#"{"session_id":"test-session-123","hook_event_name":"SessionStart","timestamp":"2025-01-01T10:00:00Z"}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_tokens() {
        // State with token metrics
        let transcript_events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":200,"cache_read_input_tokens":300}}}"#,
        ];
        let (_transcript_temp, transcript_path) = create_transcript_file(&transcript_events);
        let hook = hook_with_transcript(&transcript_path, "test", "2025-01-01T10:00:00Z");
        let (_temp_dir, storage) = test_storage_with_files(Some(&[&hook]), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_bash_commands() {
        // State with bash commands
        let hooks = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:06:00Z","tool_input":{"command":"cargo test"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:07:00Z","tool_input":{"command":"cargo build"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_file_modifications() {
        // State with file modifications
        let hooks = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:05:00Z","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write","timestamp":"2025-01-01T10:06:00Z","tool_input":{"file_path":"README.md"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_state_transitions() {
        // State with workflow transitions
        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(None, Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_phase_metrics() {
        // Full workflow with phase metrics
        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let hooks = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:10:00Z","tool_input":{"file_path":"spec.md"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_active_phase() {
        // Active phase (no end_time) should display correctly
        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(None, Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_long_command() {
        // Very long bash command should be truncated
        let long_command = "a".repeat(100);
        let hook_str = format!(
            r#"{{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{{"command":"{}"}}}}"#,
            long_command
        );
        let (_temp_dir, storage) = test_storage_with_files(Some(&[&hook_str]), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_comprehensive() {
        // Test all sections together
        let transcript_events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":100,"output_tokens":50}}}"#,
            r#"{"type":"assistant","timestamp":"2025-01-01T10:20:00Z","message":{"usage":{"input_tokens":150,"output_tokens":75}}}"#,
        ];
        let (_transcript_temp, transcript_path) = create_transcript_file(&transcript_events);

        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];

        let hook = hook_with_transcript(
            &transcript_path,
            "test-comprehensive",
            "2025-01-01T10:00:00Z",
        );
        let hooks = vec![
            hook.as_str(),
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:10:00Z","tool_input":{"file_path":"spec.md"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:20:00Z","tool_input":{"command":"cargo test"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }
}
