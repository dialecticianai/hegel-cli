use crate::storage::*;
use crate::test_helpers::*;
use tempfile::TempDir;

// ========== FileStorage::new Tests ==========

#[test]
fn test_file_storage_new_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path().join("test_state");

    assert!(!state_dir.exists());

    let storage = FileStorage::new(&state_dir).unwrap();
    assert!(state_dir.exists());
    assert_eq!(storage.state_dir, state_dir);
}

#[test]
fn test_file_storage_new_existing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path().join("existing");
    std::fs::create_dir_all(&state_dir).unwrap();

    assert!(state_dir.exists());

    let storage = FileStorage::new(&state_dir).unwrap();
    assert!(state_dir.exists());
    assert_eq!(storage.state_dir, state_dir);
}

// ========== load Tests ==========

#[test]
fn test_load_returns_empty_state_when_no_file_exists() {
    let (_temp_dir, storage) = test_storage();

    let state = storage.load().unwrap();
    assert!(state.workflow.is_none());
}

#[test]
fn test_load_returns_saved_state() {
    let (_temp_dir, storage) = test_storage();
    storage
        .save(&test_state("spec", "discovery", &["spec"]))
        .unwrap();
    let loaded = storage.load().unwrap();
    assert_state_eq(&loaded, "spec", "discovery", &["spec"]);
}

#[test]
fn test_load_invalid_json_returns_error() {
    let (temp_dir, storage) = test_storage();

    // Write invalid JSON
    let state_file = temp_dir.path().join("state.json");
    std::fs::write(&state_file, "invalid json content {{{").unwrap();

    let result = storage.load();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to parse state file"));
}

// ========== save Tests ==========

#[test]
fn test_save_creates_state_file() {
    let (temp_dir, storage) = test_storage();
    storage
        .save(&test_state("plan", "execution", &["spec", "plan"]))
        .unwrap();
    assert!(temp_dir.path().join("state.json").exists());
}

#[test]
fn test_save_overwrites_existing_state() {
    let (_temp_dir, storage) = test_storage();
    storage
        .save(&test_state("spec", "discovery", &["spec"]))
        .unwrap();
    storage
        .save(&test_state("plan", "execution", &["spec", "plan"]))
        .unwrap();
    let loaded = storage.load().unwrap();
    assert_state_eq(&loaded, "plan", "execution", &["spec", "plan"]);
}

#[test]
fn test_save_is_atomic() {
    let (temp_dir, storage) = test_storage();
    storage
        .save(&test_state("spec", "discovery", &["spec"]))
        .unwrap();
    assert!(!temp_dir.path().join("state.json.tmp").exists());
    assert!(temp_dir.path().join("state.json").exists());
}

// ========== clear Tests ==========

#[test]
fn test_clear_removes_state_file() {
    let (temp_dir, storage) = test_storage();
    storage
        .save(&test_state("spec", "discovery", &["spec"]))
        .unwrap();
    let state_file = temp_dir.path().join("state.json");
    assert!(state_file.exists());
    storage.clear().unwrap();
    assert!(!state_file.exists());
}

#[test]
fn test_clear_when_no_state_file_exists() {
    let (_temp_dir, storage) = test_storage();
    assert!(storage.clear().is_ok());
}

#[test]
fn test_clear_then_load_returns_empty_state() {
    let (_temp_dir, storage) = test_storage();
    storage
        .save(&test_state("spec", "discovery", &["spec"]))
        .unwrap();
    storage.clear().unwrap();
    let loaded = storage.load().unwrap();
    assert!(loaded.workflow.is_none() && loaded.workflow.is_none());
}

// ========== State Directory Resolution Tests ==========

#[test]
fn test_resolve_state_dir_default() {
    // When no CLI flag or env var, should find .hegel by walking up from cwd
    let resolved = FileStorage::resolve_state_dir(None).unwrap();
    // Should find the project's .hegel directory
    assert!(resolved.exists());
    assert!(resolved.is_dir());
    assert_eq!(resolved.file_name().unwrap(), ".hegel");
}

#[test]
#[serial_test::serial]
fn test_resolve_state_dir_with_env_var() {
    // When HEGEL_STATE_DIR is set, should use it
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("HEGEL_STATE_DIR", temp_dir.path());

    let resolved = FileStorage::resolve_state_dir(None).unwrap();
    assert_eq!(resolved, temp_dir.path());

    std::env::remove_var("HEGEL_STATE_DIR");
}

#[test]
fn test_resolve_state_dir_with_cli_flag() {
    // When CLI flag is provided, should use it
    let temp_dir = TempDir::new().unwrap();
    let cli_path = temp_dir.path().to_path_buf();

    let resolved = FileStorage::resolve_state_dir(Some(cli_path.clone())).unwrap();
    assert_eq!(resolved, cli_path);
}

#[test]
#[serial_test::serial]
fn test_resolve_state_dir_precedence() {
    // CLI flag should override env var
    let env_dir = TempDir::new().unwrap();
    let cli_dir = TempDir::new().unwrap();

    std::env::set_var("HEGEL_STATE_DIR", env_dir.path());
    let cli_path = cli_dir.path().to_path_buf();

    let resolved = FileStorage::resolve_state_dir(Some(cli_path.clone())).unwrap();
    assert_eq!(resolved, cli_path);

    std::env::remove_var("HEGEL_STATE_DIR");
}

// ========== Parent Directory Discovery Tests ==========

#[test]
fn test_find_project_root_in_current_dir() {
    // Create temp dir with .hegel subdirectory
    let temp_dir = TempDir::new().unwrap();
    let hegel_dir = temp_dir.path().join(".hegel");
    std::fs::create_dir(&hegel_dir).unwrap();

    // Find project root starting from temp_dir (no cwd mutation!)
    let found = FileStorage::find_project_root_from(Some(temp_dir.path().to_path_buf())).unwrap();

    // Canonicalize both paths to handle symlinks (e.g., /var -> /private/var on macOS)
    assert_eq!(
        found.canonicalize().unwrap(),
        hegel_dir.canonicalize().unwrap()
    );
}

#[test]
fn test_find_project_root_in_parent_dir() {
    // Create nested structure: temp/.hegel and temp/subdir/subdir2
    let temp_dir = TempDir::new().unwrap();
    let hegel_dir = temp_dir.path().join(".hegel");
    std::fs::create_dir(&hegel_dir).unwrap();

    let subdir1 = temp_dir.path().join("subdir");
    let subdir2 = subdir1.join("subdir2");
    std::fs::create_dir_all(&subdir2).unwrap();

    // Should find .hegel in ancestor directory starting from subdir2 (no cwd mutation!)
    let found = FileStorage::find_project_root_from(Some(subdir2)).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        hegel_dir.canonicalize().unwrap()
    );
}

#[test]
#[serial_test::serial]
fn test_find_project_root_not_found() {
    // Create temp dir WITHOUT .hegel
    let temp_dir = TempDir::new().unwrap();
    let _guard = DirGuard::new(temp_dir.path()).unwrap();

    // Should error with helpful message
    let result = FileStorage::find_project_root();
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No .hegel directory found"));
    assert!(err_msg.contains("current or parent directories"));
}

#[test]
#[serial_test::serial]
fn test_find_project_root_stops_at_first_hegel() {
    // Create nested structure with multiple .hegel dirs
    // temp/.hegel and temp/project/.hegel
    let temp_dir = TempDir::new().unwrap();
    let outer_hegel = temp_dir.path().join(".hegel");
    std::fs::create_dir(&outer_hegel).unwrap();

    let project_dir = temp_dir.path().join("project");
    std::fs::create_dir(&project_dir).unwrap();
    let inner_hegel = project_dir.join(".hegel");
    std::fs::create_dir(&inner_hegel).unwrap();

    let subdir = project_dir.join("src");
    std::fs::create_dir(&subdir).unwrap();

    // From project/src, should find project/.hegel (closest one)
    let _guard = DirGuard::new(&subdir).unwrap();

    let found = FileStorage::find_project_root().unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        inner_hegel.canonicalize().unwrap()
    );
}

// ========== Round-trip Tests ==========

#[test]
fn test_save_load_roundtrip_preserves_state() {
    let (_temp_dir, storage) = test_storage();
    storage
        .save(&test_state("code", "execution", &["spec", "plan", "code"]))
        .unwrap();
    let loaded = storage.load().unwrap();
    assert_state_eq(&loaded, "code", "execution", &["spec", "plan", "code"]);
}

#[test]
fn test_multiple_save_load_cycles() {
    let (_temp_dir, storage) = test_storage();
    for (node, history) in [
        ("spec", &["spec"][..]),
        ("plan", &["spec", "plan"][..]),
        ("code", &["spec", "plan", "code"][..]),
        ("done", &["spec", "plan", "code", "done"][..]),
    ] {
        storage
            .save(&test_state(node, "discovery", history))
            .unwrap();
        let loaded = storage.load().unwrap();
        assert_state_eq(&loaded, node, "discovery", history);
    }
}

// ========== State Transition Logging Tests ==========

#[test]
fn test_log_state_transition_creates_file() {
    let (temp_dir, storage) = test_storage();

    storage
        .log_state_transition("spec", "plan", "discovery", Some("2025-10-09T04:15:23Z"))
        .unwrap();

    let states_file = temp_dir.path().join("states.jsonl");
    assert!(states_file.exists());
}

#[test]
fn test_log_state_transition_event_schema() {
    let (temp_dir, storage) = test_storage();

    storage
        .log_state_transition("spec", "plan", "discovery", Some("2025-10-09T04:15:23Z"))
        .unwrap();

    let states_file = temp_dir.path().join("states.jsonl");

    // Parse and verify schema
    let parsed = read_jsonl_line(&states_file, 0);
    assert!(parsed.get("timestamp").is_some());
    assert!(parsed.get("workflow_id").is_some());
    assert!(parsed.get("from_node").is_some());
    assert!(parsed.get("to_node").is_some());
    assert!(parsed.get("phase").is_some());
    assert!(parsed.get("mode").is_some());

    // Verify values
    assert_eq!(parsed["from_node"], "spec");
    assert_eq!(parsed["to_node"], "plan");
    assert_eq!(parsed["phase"], "plan"); // phase is the destination node
    assert_eq!(parsed["mode"], "discovery");
    assert_eq!(parsed["workflow_id"], "2025-10-09T04:15:23Z");

    // Verify timestamp is ISO 8601
    use chrono::DateTime;
    let timestamp_str = parsed["timestamp"].as_str().unwrap();
    assert!(DateTime::parse_from_rfc3339(timestamp_str).is_ok());
}

#[test]
fn test_log_state_transition_appends_multiple() {
    let (temp_dir, storage) = test_storage();
    let transitions = [("spec", "plan"), ("plan", "code"), ("code", "learnings")];
    for (from, to) in transitions {
        storage
            .log_state_transition(from, to, "discovery", Some("wf-001"))
            .unwrap();
    }
    let events = read_jsonl_all(&temp_dir.path().join("states.jsonl"));
    assert_eq!(events.len(), 3);
    for (i, (from, to)) in transitions.iter().enumerate() {
        assert_eq!(events[i]["from_node"], *from);
        assert_eq!(events[i]["to_node"], *to);
    }
}

#[test]
fn test_log_state_transition_with_none_workflow_id() {
    let (temp_dir, storage) = test_storage();

    storage
        .log_state_transition("spec", "plan", "discovery", None)
        .unwrap();

    let states_file = temp_dir.path().join("states.jsonl");
    let parsed = read_jsonl_line(&states_file, 0);
    assert!(parsed["workflow_id"].is_null());
}

// ========== Stash Helper Functions ==========

fn minimal_workflow() -> serde_yaml::Value {
    serde_yaml::to_value(serde_yaml::Mapping::new()).unwrap()
}

fn stash_count(storage: &FileStorage) -> usize {
    let stash_dir = storage.state_dir().join("stashes");
    if !stash_dir.exists() {
        return 0;
    }
    std::fs::read_dir(&stash_dir)
        .unwrap()
        .filter(|e| e.is_ok())
        .count()
}

fn stash_with_message(storage: &FileStorage, message: &str) {
    let state = test_state("plan", "discovery", &["spec", "plan"]);
    storage.save(&state).unwrap();
    storage.save_stash(Some(message.to_string())).unwrap();
}

fn stash_without_message(storage: &FileStorage) {
    let state = test_state("code", "execution", &["spec", "plan", "code"]);
    storage.save(&state).unwrap();
    storage.save_stash(None).unwrap();
}

fn assert_stash_has_message(entry: &StashEntry, expected: &str) {
    assert_eq!(entry.message.as_deref(), Some(expected));
}

fn assert_stash_no_message(entry: &StashEntry) {
    assert!(entry.message.is_none());
}

// ========== Save Stash Tests ==========

#[test]
fn test_save_stash_creates_file_and_clears_state() {
    let (_tmp, storage) = test_storage();
    let state = test_state("spec", "discovery", &["spec"]);
    storage.save(&state).unwrap();

    storage.save_stash(Some("test stash".to_string())).unwrap();

    assert_eq!(stash_count(&storage), 1);

    let loaded = storage.load().unwrap();
    assert!(loaded.workflow.is_none());
    assert!(loaded.workflow.is_none());
}

#[test]
fn test_save_stash_with_message() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "fixing bug");

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes.len(), 1);
    assert_stash_has_message(&stashes[0], "fixing bug");
}

#[test]
fn test_save_stash_without_message() {
    let (_tmp, storage) = test_storage();
    stash_without_message(&storage);

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes.len(), 1);
    assert_stash_no_message(&stashes[0]);
}

#[test]
fn test_save_multiple_stashes_assigns_sequential_indices() {
    let (_tmp, storage) = test_storage();

    stash_with_message(&storage, "first");
    stash_with_message(&storage, "second");
    stash_with_message(&storage, "third");

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes.len(), 3);
    assert_eq!(stashes[0].index, 0);
    assert_eq!(stashes[1].index, 1);
    assert_eq!(stashes[2].index, 2);
}

#[test]
fn test_save_stash_preserves_workflow_state() {
    let (_tmp, storage) = test_storage();
    let state = test_state("plan", "discovery", &["spec", "plan"]);
    storage.save(&state).unwrap();

    storage.save_stash(None).unwrap();

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes[0].workflow.current_node, "plan");
    assert_eq!(stashes[0].workflow.mode, "discovery");
    assert_eq!(stashes[0].workflow.history, vec!["spec", "plan"]);
}

#[test]
fn test_save_stash_no_workflow_fails() {
    let (_tmp, storage) = test_storage();

    let result = storage.save_stash(None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No active workflow"));
}

// ========== List Stashes Tests ==========

#[test]
fn test_list_stashes_empty_returns_empty_vec() {
    let (_tmp, storage) = test_storage();
    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes.len(), 0);
}

#[test]
fn test_list_stashes_returns_newest_first() {
    let (_tmp, storage) = test_storage();

    stash_with_message(&storage, "first");
    std::thread::sleep(std::time::Duration::from_millis(10));
    stash_with_message(&storage, "second");
    std::thread::sleep(std::time::Duration::from_millis(10));
    stash_with_message(&storage, "third");

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes.len(), 3);
    assert_stash_has_message(&stashes[0], "third");
    assert_stash_has_message(&stashes[1], "second");
    assert_stash_has_message(&stashes[2], "first");
}

#[test]
fn test_list_stashes_includes_timestamp() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "test");

    let stashes = storage.list_stashes().unwrap();
    use chrono::DateTime;
    assert!(DateTime::parse_from_rfc3339(&stashes[0].timestamp).is_ok());
}

// ========== Load Stash Tests ==========

#[test]
fn test_load_stash_by_index() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "first");
    stash_with_message(&storage, "second");

    let stash = storage.load_stash(1).unwrap();
    assert_stash_has_message(&stash, "first");
}

#[test]
fn test_load_stash_invalid_index_fails() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "only one");

    let result = storage.load_stash(5);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Stash index 5 not found"));
    assert!(err_msg.contains("0-0"));
}

#[test]
fn test_load_stash_empty_fails() {
    let (_tmp, storage) = test_storage();

    let result = storage.load_stash(0);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No stashes"));
}

// ========== Delete Stash Tests ==========

#[test]
fn test_delete_stash_removes_file() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "to delete");

    assert_eq!(stash_count(&storage), 1);
    storage.delete_stash(0).unwrap();
    assert_eq!(stash_count(&storage), 0);
}

#[test]
fn test_delete_stash_reindexes_remaining() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "first");
    stash_with_message(&storage, "second");
    stash_with_message(&storage, "third");

    storage.delete_stash(1).unwrap();

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes.len(), 2);
    assert_eq!(stashes[0].index, 0);
    assert_eq!(stashes[1].index, 1);
    assert_stash_has_message(&stashes[0], "third");
    assert_stash_has_message(&stashes[1], "first");
}

#[test]
fn test_delete_stash_invalid_index_fails() {
    let (_tmp, storage) = test_storage();
    stash_with_message(&storage, "only one");

    let result = storage.delete_stash(5);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Stash index 5 not found"));
}

// ========== Round-trip Tests ==========

#[test]
fn test_stash_and_restore_preserves_state_exactly() {
    let (_tmp, storage) = test_storage();

    let original = test_state("code", "execution", &["spec", "plan", "code"]);
    storage.save(&original).unwrap();
    storage
        .save_stash(Some("round trip test".to_string()))
        .unwrap();

    let stash = storage.load_stash(0).unwrap();

    let restored = State {
        workflow: Some(stash.workflow),
        session_metadata: None,
        cumulative_totals: None,
        git_info: None,
    };

    assert_state_eq(&restored, "code", "execution", &["spec", "plan", "code"]);
}
