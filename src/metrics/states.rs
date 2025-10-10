use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_parse_states_file() {
        let events = vec![
            r#"{"timestamp":"2025-01-01T00:00:00Z","workflow_id":"wf-001","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T01:00:00Z","workflow_id":"wf-001","from_node":"plan","to_node":"code","phase":"code","mode":"discovery"}"#,
        ];
        let (_temp_dir, states_path) = create_states_file(&events);
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
        let events = vec![
            r#"{"timestamp":"2025-01-01T00:00:00Z","workflow_id":null,"from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let (_temp_dir, states_path) = create_states_file(&events);
        let transitions = parse_states_file(&states_path).unwrap();

        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].workflow_id, None);
    }

    #[test]
    fn test_parse_states_empty_file() {
        let events: Vec<&str> = vec![];
        let (_temp_dir, states_path) = create_states_file(&events);
        let transitions = parse_states_file(&states_path).unwrap();

        assert_eq!(transitions.len(), 0);
    }

    #[test]
    fn test_parse_states_file_not_found() {
        use std::path::PathBuf;
        let nonexistent = PathBuf::from("/nonexistent/path/states.jsonl");
        let result = parse_states_file(&nonexistent);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read states file"));
    }

    #[test]
    fn test_parse_states_malformed_json() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let states_path = temp_dir.path().join("states.jsonl");

        // Write invalid JSON
        std::fs::write(&states_path, "not valid json\n").unwrap();

        let result = parse_states_file(&states_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse state transition"));
    }

    #[test]
    fn test_parse_states_missing_field() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let states_path = temp_dir.path().join("states.jsonl");

        // Missing required field "mode"
        std::fs::write(&states_path, r#"{"timestamp":"2025-01-01T00:00:00Z","from_node":"spec","to_node":"plan","phase":"plan"}"#).unwrap();

        let result = parse_states_file(&states_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_states_all_fields() {
        // Verify all fields are parsed correctly
        let events = vec![
            r#"{"timestamp":"2025-01-01T10:30:00Z","workflow_id":"wf-test","from_node":"spec","to_node":"plan","phase":"plan","mode":"execution"}"#,
        ];
        let (_temp_dir, states_path) = create_states_file(&events);
        let transitions = parse_states_file(&states_path).unwrap();

        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].timestamp, "2025-01-01T10:30:00Z");
        assert_eq!(transitions[0].workflow_id, Some("wf-test".to_string()));
        assert_eq!(transitions[0].from_node, "spec");
        assert_eq!(transitions[0].to_node, "plan");
        assert_eq!(transitions[0].phase, "plan");
        assert_eq!(transitions[0].mode, "execution");
    }
}
