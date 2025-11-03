//! JSONL file creation and reading helpers

use std::path::PathBuf;
use tempfile::TempDir;

/// Read a specific line from a JSONL file and parse as JSON
///
/// # Arguments
/// * `path` - Path to the JSONL file
/// * `line_num` - Zero-indexed line number to read
///
/// # Returns
/// Parsed JSON value from the specified line
///
/// # Panics
/// Panics if file doesn't exist, line doesn't exist, or JSON is invalid
///
/// # Example
/// ```ignore
/// let event = read_jsonl_line(&hooks_file, 0);
/// assert_eq!(event["session_id"], "test");
/// ```
pub fn read_jsonl_line(path: &PathBuf, line_num: usize) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    serde_json::from_str(lines[line_num]).unwrap()
}

/// Read all lines from a JSONL file and parse as JSON array
///
/// # Arguments
/// * `path` - Path to the JSONL file
///
/// # Returns
/// Vector of parsed JSON values, one per line
///
/// # Example
/// ```ignore
/// let events = read_jsonl_all(&hooks_file);
/// assert_eq!(events.len(), 3);
/// ```
pub fn read_jsonl_all(path: &PathBuf) -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string(path).unwrap();
    content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// Count the number of lines in a JSONL file
///
/// # Arguments
/// * `path` - Path to the JSONL file
///
/// # Returns
/// Number of lines in the file, or 0 if file doesn't exist
///
/// # Example
/// ```ignore
/// let count = count_jsonl_lines(&states_file);
/// assert_eq!(count, 2);
/// ```
#[allow(dead_code)] // Reserved for future metrics validation tests (see test_helpers/README.md)
pub fn count_jsonl_lines(path: &PathBuf) -> usize {
    if !path.exists() {
        return 0;
    }
    let content = std::fs::read_to_string(path).unwrap();
    content.lines().count()
}

/// Create a JSONL file for testing with given events
///
/// # Arguments
/// * `events` - Array of JSON strings (one per line)
/// * `filename` - Name of the JSONL file to create
///
/// # Returns
/// A tuple of (TempDir, PathBuf) where PathBuf points to the created file
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"type":"test","value":1}"#];
/// let (_temp_dir, path) = create_jsonl_file(&events, "data.jsonl");
/// ```
pub fn create_jsonl_file(events: &[&str], filename: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    let content = events.join("\n");
    std::fs::write(&file_path, content).unwrap();
    (temp_dir, file_path)
}

/// Create a transcript.jsonl file for testing
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"type":"assistant","usage":{"input_tokens":100}}"#];
/// let (_temp_dir, path) = create_transcript_file(&events);
/// ```
pub fn create_transcript_file(events: &[&str]) -> (TempDir, PathBuf) {
    create_jsonl_file(events, "transcript.jsonl")
}

/// Create a states.jsonl file for testing
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"timestamp":"2025-01-01T00:00:00Z","from_node":"spec","to_node":"plan"}"#];
/// let (_temp_dir, path) = create_states_file(&events);
/// ```
pub fn create_states_file(events: &[&str]) -> (TempDir, PathBuf) {
    create_jsonl_file(events, "states.jsonl")
}

/// Create a hooks.jsonl file for testing
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"session_id":"test","hook_event_name":"SessionStart"}"#];
/// let (_temp_dir, path) = create_hooks_file(&events);
/// ```
pub fn create_hooks_file(events: &[&str]) -> (TempDir, PathBuf) {
    create_jsonl_file(events, "hooks.jsonl")
}
