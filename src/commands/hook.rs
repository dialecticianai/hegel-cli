use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};

use crate::adapters::{AdapterRegistry, EventType};
use crate::storage::{FileStorage, SessionMetadata};

/// Process a hook event JSON string using adapter normalization
fn process_hook_event(hook_json: &str, storage: &FileStorage) -> Result<()> {
    use chrono::Utc;

    // Parse raw JSON
    let raw_value: serde_json::Value =
        serde_json::from_str(hook_json).context("Invalid JSON received")?;

    // Detect and use appropriate adapter
    let registry = AdapterRegistry::new();
    let adapter = registry
        .detect()
        .context("No agent adapter detected. Is this running in an AI coding environment?")?;

    // Normalize to canonical event
    let canonical = adapter
        .normalize(raw_value)?
        .context("Adapter returned None - event should be skipped")?;

    // Inject timestamp if not present
    let mut enriched = canonical;
    if enriched.timestamp.is_empty() {
        enriched.timestamp = Utc::now().to_rfc3339();
    }

    // Serialize enriched canonical event
    let enriched_json =
        serde_json::to_string(&enriched).context("Failed to serialize enriched hook event")?;

    // Get hooks.jsonl path using the storage's state dir
    let state_dir = storage.state_dir();
    let hooks_file = state_dir.join("hooks.jsonl");

    // Ensure directory exists (should already exist from storage init, but be safe)
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("Failed to create state directory: {:?}", state_dir))?;

    // Append hook JSON to hooks.jsonl with exclusive file lock
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&hooks_file)
        .with_context(|| format!("Failed to open hooks file: {:?}", hooks_file))?;

    // Acquire exclusive lock to prevent race conditions from concurrent hook processes
    file.lock_exclusive()
        .with_context(|| format!("Failed to lock hooks file: {:?}", hooks_file))?;

    writeln!(file, "{}", enriched_json)
        .with_context(|| format!("Failed to write to hooks file: {:?}", hooks_file))?;

    // Flush before unlocking to ensure data hits disk
    file.flush()
        .with_context(|| format!("Failed to flush hooks file: {:?}", hooks_file))?;

    // Lock is automatically released when file goes out of scope

    // If this is a SessionStart event, update state.json with session metadata
    if matches!(enriched.event_type, EventType::SessionStart) {
        // Extract session metadata
        let session_id = &enriched.session_id;

        let transcript_path = enriched
            .transcript_path
            .as_ref()
            .context("SessionStart event missing transcript_path")?;

        let started_at = &enriched.timestamp;

        // Create session metadata
        let session = SessionMetadata {
            session_id: session_id.to_string(),
            transcript_path: transcript_path.to_string(),
            started_at: started_at.to_string(),
        };

        // Load current state, update session metadata, save back
        let mut state = storage.load()?;
        state.session_metadata = Some(session);
        storage
            .save(&state)
            .context("Failed to save session metadata to state.json")?;
    }

    Ok(())
}

pub fn handle_hook(_event_name: &str, storage: &FileStorage) -> Result<()> {
    // Read JSON from stdin
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut hook_json = String::new();
    stdin_lock
        .read_line(&mut hook_json)
        .context("Failed to read hook JSON from stdin")?;

    // Trim whitespace and process
    process_hook_event(hook_json.trim(), storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_hook_injects_timestamp() {
        let (temp_dir, storage) = test_storage();

        let hook_json =
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Read"}"#;

        // Process the hook event
        process_hook_event(hook_json, &storage).unwrap();

        // Read and parse the hooks.jsonl file
        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let parsed = read_jsonl_line(&hooks_file, 0);
        assert!(parsed.get("timestamp").is_some());
        assert_eq!(parsed["session_id"], "test");
        assert_eq!(parsed["event_type"], "post_tool_use");
        assert_eq!(parsed["tool_name"], "Read");
        assert_eq!(parsed["adapter"], "claude_code");
    }

    #[test]
    fn test_hook_preserves_existing_timestamp() {
        let (temp_dir, storage) = test_storage();

        let hook_json = r#"{"session_id":"test","timestamp":"2025-01-01T00:00:00Z","hook_event_name":"PostToolUse","tool_name":"Edit"}"#;

        process_hook_event(hook_json, &storage).unwrap();

        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let parsed = read_jsonl_line(&hooks_file, 0);
        assert_eq!(parsed["timestamp"], "2025-01-01T00:00:00Z"); // Original timestamp preserved
    }

    #[test]
    fn test_hook_appends_multiple_events() {
        let (temp_dir, storage) = test_storage();

        process_hook_event(
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Read"}"#,
            &storage,
        )
        .unwrap();
        process_hook_event(
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write"}"#,
            &storage,
        )
        .unwrap();

        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let events = read_jsonl_all(&hooks_file);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["tool_name"], "Read");
        assert_eq!(events[1]["tool_name"], "Write");
        assert!(events[0].get("timestamp").is_some());
        assert!(events[1].get("timestamp").is_some());
    }

    #[test]
    fn test_session_start_updates_state_json() {
        let (_temp_dir, storage) = test_storage();

        let hook_json = r#"{
            "session_id": "test-session-abc",
            "hook_event_name": "SessionStart",
            "transcript_path": "/tmp/test-transcript.jsonl"
        }"#;

        process_hook_event(hook_json, &storage).unwrap();

        // Load state and verify session_metadata was updated
        let state = storage.load().unwrap();
        assert!(state.session_metadata.is_some());

        let session = state.session_metadata.unwrap();
        assert_eq!(session.session_id, "test-session-abc");
        assert_eq!(session.transcript_path, "/tmp/test-transcript.jsonl");
        assert!(session.started_at.starts_with("20")); // ISO 8601 timestamp starts with year
    }

    #[test]
    fn test_non_session_start_does_not_update_session_metadata() {
        let (_temp_dir, storage) = test_storage();

        let hook_json = r#"{
            "session_id": "test-session-xyz",
            "hook_event_name": "PostToolUse",
            "tool_name": "Read"
        }"#;

        process_hook_event(hook_json, &storage).unwrap();

        // session_metadata should remain None
        let state = storage.load().unwrap();
        assert!(state.session_metadata.is_none());
    }

    #[test]
    fn test_session_start_overwrites_previous_session() {
        let (_temp_dir, storage) = test_storage();

        let session1_json = r#"{
            "session_id": "session-1",
            "hook_event_name": "SessionStart",
            "transcript_path": "/tmp/transcript1.jsonl"
        }"#;

        let session2_json = r#"{
            "session_id": "session-2",
            "hook_event_name": "SessionStart",
            "transcript_path": "/tmp/transcript2.jsonl"
        }"#;

        process_hook_event(session1_json, &storage).unwrap();
        process_hook_event(session2_json, &storage).unwrap();

        // Most recent session should be in state.json
        let state = storage.load().unwrap();
        let session = state.session_metadata.unwrap();
        assert_eq!(session.session_id, "session-2");
        assert_eq!(session.transcript_path, "/tmp/transcript2.jsonl");
    }
}
