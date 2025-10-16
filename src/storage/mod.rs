use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Session metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub transcript_path: String,
    pub started_at: String,
}

/// Meta-mode structure - defines workflow progression pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaMode {
    pub name: String, // "learning" or "standard"
}

/// Workflow state structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub current_node: String,
    pub mode: String,
    pub history: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_mode: Option<MetaMode>,
}

/// Complete state including workflow definition and current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<serde_yaml::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_state: Option<WorkflowState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_metadata: Option<SessionMetadata>,
}

/// File-based state storage
pub struct FileStorage {
    state_dir: PathBuf,
    workflows_dir: Option<PathBuf>,
    guides_dir: Option<PathBuf>,
}

impl FileStorage {
    /// Create a new FileStorage instance
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create state directory: {:?}", state_dir))?;
        Ok(Self {
            state_dir,
            workflows_dir: None,
            guides_dir: None,
        })
    }

    /// Create a FileStorage with custom workflows and guides directories (for testing)
    pub fn with_dirs<P: AsRef<Path>>(
        state_dir: P,
        workflows_dir: Option<P>,
        guides_dir: Option<P>,
    ) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create state directory: {:?}", state_dir))?;
        Ok(Self {
            state_dir,
            workflows_dir: workflows_dir.map(|p| p.as_ref().to_path_buf()),
            guides_dir: guides_dir.map(|p| p.as_ref().to_path_buf()),
        })
    }

    /// Get the state directory path
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Get the workflows directory (if set), otherwise use env var or default
    pub fn workflows_dir(&self) -> String {
        if let Some(ref dir) = self.workflows_dir {
            dir.to_str().unwrap().to_string()
        } else {
            std::env::var("HEGEL_WORKFLOWS_DIR").unwrap_or_else(|_| "workflows".to_string())
        }
    }

    /// Get the guides directory (if set), otherwise use env var or default
    pub fn guides_dir(&self) -> String {
        if let Some(ref dir) = self.guides_dir {
            dir.to_str().unwrap().to_string()
        } else {
            std::env::var("HEGEL_GUIDES_DIR").unwrap_or_else(|_| "guides".to_string())
        }
    }

    /// Get the default state directory (.hegel in current working directory)
    pub fn default_state_dir() -> Result<PathBuf> {
        let cwd =
            std::env::current_dir().context("Could not determine current working directory")?;
        Ok(cwd.join(".hegel"))
    }

    /// Resolve state directory with precedence: CLI flag > env var > default
    pub fn resolve_state_dir(cli_flag: Option<PathBuf>) -> Result<PathBuf> {
        // Check CLI flag first (highest precedence)
        if let Some(path) = cli_flag {
            return Ok(path);
        }

        // Check HEGEL_STATE_DIR env var
        if let Ok(env_path) = std::env::var("HEGEL_STATE_DIR") {
            return Ok(PathBuf::from(env_path));
        }

        // Fall back to default
        Self::default_state_dir()
    }

    /// Load state from file
    pub fn load(&self) -> Result<State> {
        let state_file = self.state_dir.join("state.json");

        if !state_file.exists() {
            return Ok(State {
                workflow: None,
                workflow_state: None,
                session_metadata: None,
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

    /// Append a JSON value to a JSONL file with exclusive locking
    ///
    /// Helper method for appending JSON objects to JSONL files with proper error handling
    /// and file locking to prevent race conditions
    fn append_jsonl(&self, filename: &str, json_value: &serde_json::Value) -> Result<()> {
        let file_path = self.state_dir.join(filename);

        // Serialize to JSON line
        let json_line = serde_json::to_string(json_value)
            .with_context(|| format!("Failed to serialize JSON for {}", filename))?;

        // Append to file with exclusive lock
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;

        // Acquire exclusive lock to prevent race conditions
        file.lock_exclusive()
            .with_context(|| format!("Failed to lock file: {:?}", file_path))?;

        writeln!(file, "{}", json_line)
            .with_context(|| format!("Failed to write to file: {:?}", file_path))?;

        // Flush before unlocking to ensure data hits disk
        file.flush()
            .with_context(|| format!("Failed to flush file: {:?}", file_path))?;

        // Lock is automatically released when file goes out of scope
        Ok(())
    }

    /// Log a state transition to states.jsonl
    pub fn log_state_transition(
        &self,
        from_node: &str,
        to_node: &str,
        mode: &str,
        workflow_id: Option<&str>,
    ) -> Result<()> {
        use chrono::Utc;

        // Create state transition event
        let event = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "workflow_id": workflow_id,
            "from_node": from_node,
            "to_node": to_node,
            "phase": to_node,  // phase is the destination node
            "mode": mode,
        });

        self.append_jsonl("states.jsonl", &event)
    }

    /// Log a command invocation to command_log.jsonl
    pub fn log_command(
        &self,
        command: &str,
        args: &[String],
        success: bool,
        blocked_reason: Option<&str>,
    ) -> Result<()> {
        use chrono::Utc;

        let event = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "command": command,
            "args": args,
            "success": success,
            "blocked": blocked_reason.is_some(),
            "blocked_reason": blocked_reason,
        });

        self.append_jsonl("command_log.jsonl", &event)
    }

    /// Read command log for testing/analysis
    #[cfg(test)]
    pub fn read_command_log(&self) -> Result<Vec<CommandLogEntry>> {
        let log_path = self.state_dir.join("command_log.jsonl");
        if !log_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&log_path)?;
        let entries: Vec<CommandLogEntry> = content
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_str(line))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
pub struct CommandLogEntry {
    pub command: String,
    pub args: Vec<String>,
    pub success: bool,
    pub blocked: bool,
    pub blocked_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
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
        fs::create_dir_all(&state_dir).unwrap();

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
        assert!(state.workflow_state.is_none());
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
        assert!(loaded.workflow.is_none() && loaded.workflow_state.is_none());
    }

    // ========== State Directory Resolution Tests ==========

    #[test]
    fn test_resolve_state_dir_default() {
        // When no CLI flag or env var, should use default (.hegel in cwd)
        let resolved = FileStorage::resolve_state_dir(None).unwrap();
        let expected = std::env::current_dir().unwrap().join(".hegel");
        assert_eq!(resolved, expected);
    }

    #[test]
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
}
