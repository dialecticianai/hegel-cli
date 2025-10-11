use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};

use crate::storage::{FileStorage, SessionMetadata};

/// Process a hook event JSON string and write to hooks.jsonl with timestamp
fn process_hook_event(hook_json: &str, storage: &FileStorage) -> Result<()> {
    use chrono::Utc;

    // Parse and validate JSON
    let mut hook_value: serde_json::Value =
        serde_json::from_str(hook_json).context("Invalid JSON received")?;

    // Inject timestamp if not present
    if let serde_json::Value::Object(ref mut map) = hook_value {
        if !map.contains_key("timestamp") {
            map.insert(
                "timestamp".to_string(),
                serde_json::Value::String(Utc::now().to_rfc3339()),
            );
        }
    }

    // Serialize back to JSON
    let enriched_json =
        serde_json::to_string(&hook_value).context("Failed to serialize enriched hook event")?;

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

    // If this is a SessionStart event, update current_session.json
    if let serde_json::Value::Object(ref map) = hook_value {
        if let Some(event_name) = map.get("hook_event_name").and_then(|v| v.as_str()) {
            if event_name == "SessionStart" {
                // Extract session metadata
                let session_id = map
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .context("SessionStart event missing session_id")?;

                let transcript_path = map
                    .get("transcript_path")
                    .and_then(|v| v.as_str())
                    .context("SessionStart event missing transcript_path")?;

                let started_at = map
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .context("SessionStart event missing timestamp")?;

                // Save session metadata
                let session = SessionMetadata {
                    session_id: session_id.to_string(),
                    transcript_path: transcript_path.to_string(),
                    started_at: started_at.to_string(),
                };

                storage
                    .save_current_session(&session)
                    .context("Failed to save current session metadata to current_session.json")?;
            }
        }
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
        assert_eq!(parsed["hook_event_name"], "PostToolUse");
        assert_eq!(parsed["tool_name"], "Read");
    }

    #[test]
    fn test_hook_preserves_existing_timestamp() {
        let (temp_dir, storage) = test_storage();

        let hook_json =
            r#"{"session_id":"test","timestamp":"2025-01-01T00:00:00Z","tool_name":"Edit"}"#;

        process_hook_event(hook_json, &storage).unwrap();

        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let parsed = read_jsonl_line(&hooks_file, 0);
        assert_eq!(parsed["timestamp"], "2025-01-01T00:00:00Z"); // Original timestamp preserved
    }

    #[test]
    fn test_hook_appends_multiple_events() {
        let (temp_dir, storage) = test_storage();

        process_hook_event(r#"{"event":"first"}"#, &storage).unwrap();
        process_hook_event(r#"{"event":"second"}"#, &storage).unwrap();

        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let events = read_jsonl_all(&hooks_file);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event"], "first");
        assert_eq!(events[1]["event"], "second");
        assert!(events[0].get("timestamp").is_some());
        assert!(events[1].get("timestamp").is_some());
    }

    #[test]
    fn test_session_start_creates_current_session_file() {
        let (temp_dir, storage) = test_storage();

        let hook_json = r#"{
            "session_id": "test-session-abc",
            "hook_event_name": "SessionStart",
            "transcript_path": "/tmp/test-transcript.jsonl"
        }"#;

        process_hook_event(hook_json, &storage).unwrap();

        // Verify current_session.json was created
        let session_file = temp_dir.path().join("current_session.json");
        assert!(session_file.exists());

        // Load and verify content
        let loaded_session = storage.load_current_session().unwrap();
        assert!(loaded_session.is_some());

        let session = loaded_session.unwrap();
        assert_eq!(session.session_id, "test-session-abc");
        assert_eq!(session.transcript_path, "/tmp/test-transcript.jsonl");
        assert!(session.started_at.starts_with("20")); // ISO 8601 timestamp starts with year
    }

    #[test]
    fn test_non_session_start_does_not_create_current_session_file() {
        let (temp_dir, storage) = test_storage();

        let hook_json = r#"{
            "session_id": "test-session-xyz",
            "hook_event_name": "PostToolUse",
            "tool_name": "Read"
        }"#;

        process_hook_event(hook_json, &storage).unwrap();

        // current_session.json should not exist
        let session_file = temp_dir.path().join("current_session.json");
        assert!(!session_file.exists());
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

        // Most recent session should be loaded
        let loaded = storage.load_current_session().unwrap().unwrap();
        assert_eq!(loaded.session_id, "session-2");
        assert_eq!(loaded.transcript_path, "/tmp/transcript2.jsonl");
    }
}
