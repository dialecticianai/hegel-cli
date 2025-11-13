pub mod archive;
pub mod log_cleanup;
pub mod reviews;

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
    /// Whether state_dir was explicitly provided (vs auto-detected)
    explicit_state_dir: bool,
}

impl FileStorage {
    /// Create a new FileStorage instance
    pub fn new<P: AsRef<Path>>(state_dir: P) -> Result<Self> {
        Self::new_with_explicit(state_dir, false)
    }

    /// Create a new FileStorage instance with explicit state-dir flag
    ///
    /// When `explicit_state_dir` is true, path resolution uses CWD as root (for test isolation).
    /// When false, path resolution uses auto-detected project root.
    pub fn new_with_explicit<P: AsRef<Path>>(
        state_dir: P,
        explicit_state_dir: bool,
    ) -> Result<Self> {
        let state_dir = state_dir.as_ref().to_path_buf();
        fs::create_dir_all(&state_dir)
            .with_context(|| format!("Failed to create state directory: {:?}", state_dir))?;
        Ok(Self {
            state_dir,
            workflows_dir: None,
            guides_dir: None,
            explicit_state_dir,
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
            explicit_state_dir: true, // Tests use explicit temp directories
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

    /// Compute relative path from project root (or CWD) to file
    ///
    /// When `explicit_state_dir` is true (state-dir was explicitly provided):
    /// - Uses CWD as root for path resolution (for test isolation)
    ///
    /// When `explicit_state_dir` is false (state-dir was auto-detected):
    /// - Uses project root (parent of .hegel) as root
    /// - Canonicalizes paths for consistent comparison
    pub fn compute_relative_path(&self, file_path: &Path) -> Result<String> {
        if self.explicit_state_dir {
            // Explicit state-dir: use CWD as root
            let cwd = std::env::current_dir().context("Failed to get current working directory")?;

            // Make file_path absolute if needed
            let abs_file = if file_path.is_absolute() {
                file_path.to_path_buf()
            } else {
                cwd.join(file_path)
            };

            // Canonicalize for consistent path handling
            let canonical_file = abs_file.canonicalize().with_context(|| {
                format!("Failed to canonicalize file path: {}", abs_file.display())
            })?;

            // Try to strip CWD prefix, fall back to absolute path if outside CWD
            let canonical_cwd = cwd
                .canonicalize()
                .context("Failed to canonicalize current working directory")?;

            match canonical_file.strip_prefix(&canonical_cwd) {
                Ok(rel_path) => Ok(rel_path.to_string_lossy().to_string()),
                Err(_) => {
                    // File is outside CWD, use absolute path as key
                    Ok(canonical_file.to_string_lossy().to_string())
                }
            }
        } else {
            // Auto-detected state-dir: use project root as root
            let canonical_hegel = self
                .state_dir
                .canonicalize()
                .context("Failed to canonicalize project root")?;

            // Get parent of .hegel directory (project root)
            let root = canonical_hegel
                .parent()
                .context("Invalid project root path")?;

            // Make file_path absolute and canonical
            let abs_file = if file_path.is_absolute() {
                file_path.to_path_buf()
            } else {
                std::env::current_dir()?.join(file_path)
            };

            let canonical_file = abs_file.canonicalize().with_context(|| {
                format!("Failed to canonicalize file path: {}", abs_file.display())
            })?;

            // Compute relative path
            let rel_path = canonical_file
                .strip_prefix(root)
                .context("File is not within project root")?;

            Ok(rel_path.to_string_lossy().to_string())
        }
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
    mod storage;
}
