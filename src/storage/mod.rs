use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Workflow state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub current_node: String,
    pub mode: String,
    pub history: Vec<String>,
}

/// Complete state including workflow definition and current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_state: Option<WorkflowState>,
}

/// File-based state storage
pub struct FileStorage {
    state_dir: PathBuf,
}

impl FileStorage {
    /// Create a new FileStorage instance
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create state directory: {:?}", state_dir))?;
        Ok(Self { state_dir })
    }

    /// Get the default state directory (.hegel in current working directory)
    pub fn default_state_dir() -> Result<PathBuf> {
        let cwd =
            std::env::current_dir().context("Could not determine current working directory")?;
        Ok(cwd.join(".hegel"))
    }

    /// Load state from file
    pub fn load(&self) -> Result<State> {
        let state_file = self.state_dir.join("state.json");

        if !state_file.exists() {
            return Ok(State {
                workflow: None,
                workflow_state: None,
            });
        }

        let content = fs::read_to_string(&state_file)
            .with_context(|| format!("Failed to read state file: {:?}", state_file))?;

        let state: State =
            serde_json::from_str(&content).with_context(|| "Failed to parse state file")?;

        Ok(state)
    }

    /// Save state to file (atomic write)
    pub fn save(&self, state: &State) -> Result<()> {
        let state_file = self.state_dir.join("state.json");
        let temp_file = self.state_dir.join("state.json.tmp");

        let content =
            serde_json::to_string_pretty(state).with_context(|| "Failed to serialize state")?;

        fs::write(&temp_file, content)
            .with_context(|| format!("Failed to write temp state file: {:?}", temp_file))?;

        fs::rename(&temp_file, &state_file)
            .with_context(|| format!("Failed to rename state file: {:?}", state_file))?;

        Ok(())
    }

    /// Clear all state
    pub fn clear(&self) -> Result<()> {
        let state_file = self.state_dir.join("state.json");
        if state_file.exists() {
            fs::remove_file(&state_file)
                .with_context(|| format!("Failed to remove state file: {:?}", state_file))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        fs::create_dir_all(&state_dir).unwrap();

        assert!(state_dir.exists());

        let storage = FileStorage::new(&state_dir).unwrap();
        assert!(state_dir.exists());
        assert_eq!(storage.state_dir, state_dir);
    }

    // ========== load Tests ==========

    #[test]
    fn test_load_returns_empty_state_when_no_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        let state = storage.load().unwrap();
        assert!(state.workflow.is_none());
        assert!(state.workflow_state.is_none());
    }

    #[test]
    fn test_load_returns_saved_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Save a state
        let workflow_state = WorkflowState {
            current_node: "spec".to_string(),
            mode: "discovery".to_string(),
            history: vec!["spec".to_string()],
        };

        let state = State {
            workflow: None,
            workflow_state: Some(workflow_state.clone()),
        };

        storage.save(&state).unwrap();

        // Load it back
        let loaded_state = storage.load().unwrap();
        assert!(loaded_state.workflow_state.is_some());
        let loaded_workflow_state = loaded_state.workflow_state.unwrap();
        assert_eq!(loaded_workflow_state.current_node, "spec");
        assert_eq!(loaded_workflow_state.mode, "discovery");
        assert_eq!(loaded_workflow_state.history, vec!["spec"]);
    }

    #[test]
    fn test_load_invalid_json_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Write invalid JSON
        let state_file = temp_dir.path().join("state.json");
        fs::write(&state_file, "invalid json content {{{").unwrap();

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
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        let state = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "plan".to_string(),
                mode: "execution".to_string(),
                history: vec!["spec".to_string(), "plan".to_string()],
            }),
        };

        storage.save(&state).unwrap();

        let state_file = temp_dir.path().join("state.json");
        assert!(state_file.exists());
    }

    #[test]
    fn test_save_overwrites_existing_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Save first state
        let state1 = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "spec".to_string(),
                mode: "discovery".to_string(),
                history: vec!["spec".to_string()],
            }),
        };
        storage.save(&state1).unwrap();

        // Save second state (should overwrite)
        let state2 = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "plan".to_string(),
                mode: "execution".to_string(),
                history: vec!["spec".to_string(), "plan".to_string()],
            }),
        };
        storage.save(&state2).unwrap();

        // Load and verify second state
        let loaded = storage.load().unwrap();
        let loaded_state = loaded.workflow_state.unwrap();
        assert_eq!(loaded_state.current_node, "plan");
        assert_eq!(loaded_state.mode, "execution");
    }

    #[test]
    fn test_save_is_atomic() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        let state = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "spec".to_string(),
                mode: "discovery".to_string(),
                history: vec!["spec".to_string()],
            }),
        };

        storage.save(&state).unwrap();

        // Verify temp file was cleaned up
        let temp_file = temp_dir.path().join("state.json.tmp");
        assert!(!temp_file.exists());

        // Verify final file exists
        let state_file = temp_dir.path().join("state.json");
        assert!(state_file.exists());
    }

    // ========== clear Tests ==========

    #[test]
    fn test_clear_removes_state_file() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Save a state
        let state = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "spec".to_string(),
                mode: "discovery".to_string(),
                history: vec!["spec".to_string()],
            }),
        };
        storage.save(&state).unwrap();

        let state_file = temp_dir.path().join("state.json");
        assert!(state_file.exists());

        // Clear state
        storage.clear().unwrap();
        assert!(!state_file.exists());
    }

    #[test]
    fn test_clear_when_no_state_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Should not error when clearing non-existent state
        let result = storage.clear();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_then_load_returns_empty_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Save, clear, then load
        let state = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "spec".to_string(),
                mode: "discovery".to_string(),
                history: vec!["spec".to_string()],
            }),
        };
        storage.save(&state).unwrap();
        storage.clear().unwrap();

        let loaded = storage.load().unwrap();
        assert!(loaded.workflow.is_none());
        assert!(loaded.workflow_state.is_none());
    }

    // ========== Round-trip Tests ==========

    #[test]
    fn test_save_load_roundtrip_preserves_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        let original_state = State {
            workflow: None,
            workflow_state: Some(WorkflowState {
                current_node: "code".to_string(),
                mode: "execution".to_string(),
                history: vec!["spec".to_string(), "plan".to_string(), "code".to_string()],
            }),
        };

        storage.save(&original_state).unwrap();
        let loaded_state = storage.load().unwrap();

        // Verify all fields match
        assert_eq!(
            original_state.workflow_state.as_ref().unwrap().current_node,
            loaded_state.workflow_state.as_ref().unwrap().current_node
        );
        assert_eq!(
            original_state.workflow_state.as_ref().unwrap().mode,
            loaded_state.workflow_state.as_ref().unwrap().mode
        );
        assert_eq!(
            original_state.workflow_state.as_ref().unwrap().history,
            loaded_state.workflow_state.as_ref().unwrap().history
        );
    }

    #[test]
    fn test_multiple_save_load_cycles() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Simulate workflow progression
        let states = vec![
            ("spec", vec!["spec"]),
            ("plan", vec!["spec", "plan"]),
            ("code", vec!["spec", "plan", "code"]),
            ("done", vec!["spec", "plan", "code", "done"]),
        ];

        for (node, history) in states {
            let state = State {
                workflow: None,
                workflow_state: Some(WorkflowState {
                    current_node: node.to_string(),
                    mode: "discovery".to_string(),
                    history: history.iter().map(|s| s.to_string()).collect(),
                }),
            };

            storage.save(&state).unwrap();
            let loaded = storage.load().unwrap();
            assert_eq!(loaded.workflow_state.as_ref().unwrap().current_node, node);
            assert_eq!(
                loaded.workflow_state.as_ref().unwrap().history.len(),
                history.len()
            );
        }
    }
}
