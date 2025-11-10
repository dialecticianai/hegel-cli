pub mod archive;
pub mod log_cleanup;

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

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub has_repo: bool,
    pub current_branch: Option<String>,
    pub remote_url: Option<String>,
}

/// Meta-mode structure - defines workflow progression pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaMode {
    pub name: String, // "learning" or "standard"
}

/// Workflow state structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowState {
    pub current_node: String,
    pub mode: String,
    pub history: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_mode: Option<MetaMode>,
    /// RFC3339 timestamp when current phase started (for time-based rules)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_start_time: Option<String>,
    /// Whether current prompt uses Handlebars (true) or Markdown (false) template engine
    #[serde(default)]
    pub is_handlebars: bool,
}

/// Complete state including workflow state (not definition - that lives in YAML files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Current workflow state (current node, history, etc.) - NOT the workflow definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_metadata: Option<SessionMetadata>,
    /// Cumulative totals across all archived workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cumulative_totals: Option<archive::WorkflowTotals>,
    /// Cached git repository information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_info: Option<GitInfo>,
}

/// Stash entry structure - stored workflow state for later restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashEntry {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub timestamp: String,
    pub workflow: WorkflowState, // Workflow state (not definition - that lives in YAML files)
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
    #[cfg(test)]
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

    /// Detect git repository information and cache it in state
    pub fn ensure_git_info_cached(&self) -> Result<()> {
        let mut state = self.load()?;

        // If git_info is already cached, nothing to do
        if state.git_info.is_some() {
            return Ok(());
        }

        // Detect git info
        let git_info = detect_git_info(&self.state_dir);

        // Cache it in state
        state.git_info = Some(git_info);
        self.save(&state)?;

        Ok(())
    }

    /// Find .hegel directory by walking up from given starting path (like git)
    ///
    /// # Arguments
    /// * `start_path` - Optional starting path (defaults to current working directory)
    pub fn find_project_root_from(start_path: Option<PathBuf>) -> Result<PathBuf> {
        let mut current = match start_path {
            Some(path) => path,
            None => {
                std::env::current_dir().context("Could not determine current working directory")?
            }
        };

        loop {
            let hegel_dir = current.join(".hegel");
            if hegel_dir.exists() && hegel_dir.is_dir() {
                return Ok(hegel_dir);
            }

            // Try parent directory
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => {
                    anyhow::bail!(
                        "No .hegel directory found in current or parent directories.\n\
                         Run this command from within a Hegel project, or use 'hegel init' to create one."
                    );
                }
            }
        }
    }

    /// Find .hegel directory by walking up from current directory (like git)
    pub fn find_project_root() -> Result<PathBuf> {
        Self::find_project_root_from(None)
    }

    /// Get the default state directory (.hegel in current working directory)
    pub fn default_state_dir() -> Result<PathBuf> {
        Self::find_project_root()
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
                session_metadata: None,
                cumulative_totals: None,
                git_info: None,
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
    #[cfg(test)]
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

    /// Save current workflow state as a stash
    pub fn save_stash(&self, message: Option<String>) -> Result<()> {
        use chrono::Utc;

        // Load current state
        let state = self.load()?;

        // Validate workflow exists
        let workflow = state
            .workflow
            .as_ref()
            .context("No active workflow to stash")?;

        let workflow_state = workflow; // Already checked above

        // Create stash directory if needed
        let stash_dir = self.state_dir.join("stashes");
        fs::create_dir_all(&stash_dir)
            .with_context(|| format!("Failed to create stash directory: {:?}", stash_dir))?;

        // Generate timestamp for filename
        let timestamp = Utc::now().to_rfc3339();
        let filename = format!("{}.json", timestamp.replace(':', "-"));
        let stash_path = stash_dir.join(&filename);

        // Create stash entry (index will be assigned during reindex)
        let entry = StashEntry {
            index: 0, // Placeholder, will be reindexed
            message,
            timestamp: timestamp.clone(),
            workflow: workflow_state.clone(),
        };

        // Write stash file
        let content = serde_json::to_string_pretty(&entry)
            .with_context(|| "Failed to serialize stash entry")?;
        fs::write(&stash_path, content)
            .with_context(|| format!("Failed to write stash file: {:?}", stash_path))?;

        // Reindex all stashes
        self.reindex_stashes()?;

        // Clear workflow state (preserve session metadata and other global fields)
        let cleared_state = State {
            workflow: None,
            session_metadata: state.session_metadata,
            cumulative_totals: state.cumulative_totals,
            git_info: state.git_info,
        };
        self.save(&cleared_state)?;

        Ok(())
    }

    /// List all stashes, sorted newest first
    pub fn list_stashes(&self) -> Result<Vec<StashEntry>> {
        let stash_dir = self.state_dir.join("stashes");

        // Return empty vector if directory doesn't exist
        if !stash_dir.exists() {
            return Ok(vec![]);
        }

        // Read all stash files
        let mut entries = vec![];
        for entry in fs::read_dir(&stash_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip non-JSON files
            if !path.extension().map(|e| e == "json").unwrap_or(false) {
                continue;
            }

            // Parse stash entry
            let content = fs::read_to_string(&path)?;
            let stash: StashEntry = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse stash file: {:?}", path))?;

            entries.push(stash);
        }

        // Sort by timestamp descending (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(entries)
    }

    /// Load stash by index
    pub fn load_stash(&self, index: usize) -> Result<StashEntry> {
        let stashes = self.list_stashes()?;

        if stashes.is_empty() {
            anyhow::bail!("No stashes to restore");
        }

        stashes
            .into_iter()
            .find(|s| s.index == index)
            .with_context(|| {
                let max_index = self.list_stashes().unwrap().len().saturating_sub(1);
                format!(
                    "Stash index {} not found. Available stashes: 0-{}",
                    index, max_index
                )
            })
    }

    /// Delete stash by index and reindex remaining
    pub fn delete_stash(&self, index: usize) -> Result<()> {
        let stash = self.load_stash(index)?;

        // Find and delete the stash file
        let stash_dir = self.state_dir.join("stashes");
        for entry in fs::read_dir(&stash_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.extension().map(|e| e == "json").unwrap_or(false) {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let file_stash: StashEntry = serde_json::from_str(&content)?;

            if file_stash.timestamp == stash.timestamp {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to delete stash file: {:?}", path))?;
                break;
            }
        }

        // Reindex remaining stashes
        self.reindex_stashes()?;

        Ok(())
    }

    /// Reindex all stashes to ensure sequential indices starting from 0
    fn reindex_stashes(&self) -> Result<()> {
        let stash_dir = self.state_dir.join("stashes");

        if !stash_dir.exists() {
            return Ok(());
        }

        // Load all stashes and sort by timestamp
        let mut entries = vec![];
        for entry in fs::read_dir(&stash_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.extension().map(|e| e == "json").unwrap_or(false) {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let stash: StashEntry = serde_json::from_str(&content)?;

            entries.push((path, stash));
        }

        // Sort by timestamp descending (newest first)
        entries.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));

        // Update indices and rewrite files
        for (index, (path, mut stash)) in entries.into_iter().enumerate() {
            stash.index = index;
            let content = serde_json::to_string_pretty(&stash)?;
            fs::write(&path, content)?;
        }

        Ok(())
    }
}

/// Detect git repository information
fn detect_git_info(state_dir: &Path) -> GitInfo {
    let project_root = match state_dir.parent() {
        Some(p) => p,
        None => {
            return GitInfo {
                has_repo: false,
                current_branch: None,
                remote_url: None,
            }
        }
    };

    // Try to open git repository (searches upwards for monorepos)
    let repo = match git2::Repository::open(project_root) {
        Ok(r) => r,
        Err(_) => {
            return GitInfo {
                has_repo: false,
                current_branch: None,
                remote_url: None,
            }
        }
    };

    // Get current branch
    let current_branch = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(|s| s.to_string()));

    // Get remote URL (try "origin" first, then first available remote)
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()))
        .or_else(|| {
            repo.remotes()
                .ok()
                .and_then(|remotes| remotes.get(0).map(|name| name.to_string()))
                .and_then(|name| repo.find_remote(&name).ok())
                .and_then(|r| r.url().map(|s| s.to_string()))
        });

    GitInfo {
        has_repo: true,
        current_branch,
        remote_url,
    }
}

#[cfg(test)]
#[derive(Debug, Deserialize)]
pub struct CommandLogEntry {
    pub command: String,
    pub args: Vec<String>,
    pub success: bool,
    #[allow(dead_code)] // Used for deserialization
    pub blocked: bool,
    #[allow(dead_code)] // Used for deserialization
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
        fs::create_dir(&hegel_dir).unwrap();

        // Find project root starting from temp_dir (no cwd mutation!)
        let found =
            FileStorage::find_project_root_from(Some(temp_dir.path().to_path_buf())).unwrap();

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
        fs::create_dir(&hegel_dir).unwrap();

        let subdir1 = temp_dir.path().join("subdir");
        let subdir2 = subdir1.join("subdir2");
        fs::create_dir_all(&subdir2).unwrap();

        // Should find .hegel in ancestor directory starting from subdir2 (no cwd mutation!)
        let found = FileStorage::find_project_root_from(Some(subdir2)).unwrap();
        assert_eq!(
            found.canonicalize().unwrap(),
            hegel_dir.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_find_project_root_not_found() {
        // Create temp dir WITHOUT .hegel
        let temp_dir = TempDir::new().unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Should error with helpful message
        let result = FileStorage::find_project_root();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No .hegel directory found"));
        assert!(err_msg.contains("current or parent directories"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_find_project_root_stops_at_first_hegel() {
        // Create nested structure with multiple .hegel dirs
        // temp/.hegel and temp/project/.hegel
        let temp_dir = TempDir::new().unwrap();
        let outer_hegel = temp_dir.path().join(".hegel");
        fs::create_dir(&outer_hegel).unwrap();

        let project_dir = temp_dir.path().join("project");
        fs::create_dir(&project_dir).unwrap();
        let inner_hegel = project_dir.join(".hegel");
        fs::create_dir(&inner_hegel).unwrap();

        let subdir = project_dir.join("src");
        fs::create_dir(&subdir).unwrap();

        // From project/src, should find project/.hegel (closest one)
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&subdir).unwrap();

        let found = FileStorage::find_project_root().unwrap();
        assert_eq!(
            found.canonicalize().unwrap(),
            inner_hegel.canonicalize().unwrap()
        );

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
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
}
