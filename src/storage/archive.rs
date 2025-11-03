use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::metrics::UnifiedMetrics;

/// Archived workflow with pre-computed aggregates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowArchive {
    pub workflow_id: String,
    pub mode: String,
    pub completed_at: String,
    pub phases: Vec<PhaseArchive>,
    pub transitions: Vec<TransitionArchive>,
    pub totals: WorkflowTotals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Per-phase metrics in archive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseArchive {
    pub phase_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: u64,
    pub tokens: TokenTotals,
    pub bash_commands: Vec<BashCommandSummary>,
    pub file_modifications: Vec<FileModificationSummary>,
}

/// State transition in archive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionArchive {
    pub from_node: String,
    pub to_node: String,
    pub timestamp: String,
}

/// Token usage totals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TokenTotals {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
    pub assistant_turns: usize,
}

/// Bash command summary (command + frequency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BashCommandSummary {
    pub command: String,
    pub count: usize,
    pub timestamps: Vec<String>,
}

/// File modification summary (file + tool + frequency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileModificationSummary {
    pub file_path: String,
    pub tool: String,
    pub count: usize,
    pub timestamps: Vec<String>,
}

/// Workflow-level totals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkflowTotals {
    pub tokens: TokenTotals,
    pub bash_commands: usize,
    pub file_modifications: usize,
    pub unique_files: usize,
    pub unique_commands: usize,
}

impl WorkflowArchive {
    /// Create archive from unified metrics
    pub fn from_metrics(metrics: &UnifiedMetrics, workflow_id: &str) -> Result<Self> {
        validate_workflow_id(workflow_id)?;

        // Extract mode from first transition
        let mode = metrics
            .state_transitions
            .first()
            .map(|t| t.mode.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Completed_at is the last transition timestamp
        let completed_at = metrics
            .state_transitions
            .last()
            .map(|t| t.timestamp.clone())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        // Convert phases
        let phases: Vec<PhaseArchive> = metrics
            .phase_metrics
            .iter()
            .map(|phase| {
                // Aggregate bash commands by command string
                let mut bash_freq: HashMap<String, Vec<String>> = HashMap::new();
                for cmd in &phase.bash_commands {
                    bash_freq
                        .entry(cmd.command.clone())
                        .or_insert_with(Vec::new)
                        .push(cmd.timestamp.clone().unwrap_or_default());
                }
                let bash_commands: Vec<BashCommandSummary> = bash_freq
                    .into_iter()
                    .map(|(command, timestamps)| BashCommandSummary {
                        count: timestamps.len(),
                        command,
                        timestamps,
                    })
                    .collect();

                // Aggregate file modifications by file_path
                let mut file_freq: HashMap<(String, String), Vec<String>> = HashMap::new();
                for file_mod in &phase.file_modifications {
                    file_freq
                        .entry((file_mod.file_path.clone(), file_mod.tool.clone()))
                        .or_insert_with(Vec::new)
                        .push(file_mod.timestamp.clone().unwrap_or_default());
                }
                let file_modifications: Vec<FileModificationSummary> = file_freq
                    .into_iter()
                    .map(|((file_path, tool), timestamps)| FileModificationSummary {
                        count: timestamps.len(),
                        file_path,
                        tool,
                        timestamps,
                    })
                    .collect();

                PhaseArchive {
                    phase_name: phase.phase_name.clone(),
                    start_time: phase.start_time.clone(),
                    end_time: phase.end_time.clone(),
                    duration_seconds: phase.duration_seconds,
                    tokens: TokenTotals {
                        input: phase.token_metrics.total_input_tokens,
                        output: phase.token_metrics.total_output_tokens,
                        cache_creation: phase.token_metrics.total_cache_creation_tokens,
                        cache_read: phase.token_metrics.total_cache_read_tokens,
                        assistant_turns: phase.token_metrics.assistant_turns,
                    },
                    bash_commands,
                    file_modifications,
                }
            })
            .collect();

        // Convert transitions
        let transitions: Vec<TransitionArchive> = metrics
            .state_transitions
            .iter()
            .map(|t| TransitionArchive {
                from_node: t.from_node.clone(),
                to_node: t.to_node.clone(),
                timestamp: t.timestamp.clone(),
            })
            .collect();

        // Compute totals
        let totals = compute_totals(&phases, &metrics.hook_metrics);

        Ok(Self {
            workflow_id: workflow_id.to_string(),
            mode,
            completed_at,
            session_id: metrics.session_id.clone(),
            phases,
            transitions,
            totals,
        })
    }
}

/// Validate workflow_id for path safety
fn validate_workflow_id(workflow_id: &str) -> Result<()> {
    // Must not contain path separators
    if workflow_id.contains('/') || workflow_id.contains('\\') {
        bail!("Invalid workflow_id: contains path separator");
    }

    // Must not contain path traversal
    if workflow_id.contains("..") {
        bail!("Invalid workflow_id: contains path traversal");
    }

    // Must be valid ISO 8601 timestamp
    if chrono::DateTime::parse_from_rfc3339(workflow_id).is_err() {
        bail!("Invalid workflow_id: not a valid ISO 8601 timestamp");
    }

    Ok(())
}

/// Compute workflow-level totals
fn compute_totals(
    phases: &[PhaseArchive],
    hook_metrics: &crate::metrics::HookMetrics,
) -> WorkflowTotals {
    let mut totals = WorkflowTotals::default();

    // Sum tokens across phases
    for phase in phases {
        totals.tokens.input += phase.tokens.input;
        totals.tokens.output += phase.tokens.output;
        totals.tokens.cache_creation += phase.tokens.cache_creation;
        totals.tokens.cache_read += phase.tokens.cache_read;
        totals.tokens.assistant_turns += phase.tokens.assistant_turns;
    }

    // Count bash commands and files
    totals.bash_commands = hook_metrics.bash_commands.len();
    totals.file_modifications = hook_metrics.file_modifications.len();

    // Unique counts
    let unique_commands: std::collections::HashSet<_> = hook_metrics
        .bash_commands
        .iter()
        .map(|c| &c.command)
        .collect();
    totals.unique_commands = unique_commands.len();

    let unique_files: std::collections::HashSet<_> = hook_metrics
        .file_modifications
        .iter()
        .map(|f| &f.file_path)
        .collect();
    totals.unique_files = unique_files.len();

    totals
}

/// Write archive to disk with atomic operation
pub fn write_archive(archive: &WorkflowArchive, state_dir: &Path) -> Result<()> {
    let archive_dir = state_dir.join("archive");
    fs::create_dir_all(&archive_dir)
        .with_context(|| format!("Failed to create archive directory: {:?}", archive_dir))?;

    let archive_path = archive_dir.join(format!("{}.json", archive.workflow_id));

    // Check for existing archive
    if archive_path.exists() {
        bail!("Archive already exists: {}", archive.workflow_id);
    }

    // Atomic write: temp file + rename
    let temp_path = archive_path.with_extension("tmp");
    let json =
        serde_json::to_string_pretty(archive).context("Failed to serialize archive to JSON")?;

    fs::write(&temp_path, json)
        .with_context(|| format!("Failed to write temp archive: {:?}", temp_path))?;

    fs::rename(&temp_path, &archive_path).with_context(|| {
        format!(
            "Failed to rename archive: {:?} -> {:?}",
            temp_path, archive_path
        )
    })?;

    Ok(())
}

/// Read all archives from archive directory
pub fn read_archives(state_dir: &Path) -> Result<Vec<WorkflowArchive>> {
    let archive_dir = state_dir.join("archive");

    // Return empty if archive directory doesn't exist
    if !archive_dir.exists() {
        return Ok(Vec::new());
    }

    let mut archives = Vec::new();

    for entry in fs::read_dir(&archive_dir)
        .with_context(|| format!("Failed to read archive directory: {:?}", archive_dir))?
    {
        let entry = entry?;
        let path = entry.path();

        // Only process .json files
        if path.extension().map_or(false, |e| e == "json") {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<WorkflowArchive>(&content) {
                    Ok(archive) => archives.push(archive),
                    Err(e) => {
                        eprintln!("Warning: skipping corrupted archive {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read archive {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(archives)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{HookMetrics, PhaseMetrics, StateTransitionEvent, TokenMetrics};

    use tempfile::TempDir;

    #[test]
    fn test_validate_workflow_id() {
        // Valid ISO 8601 timestamp
        assert!(validate_workflow_id("2025-10-24T10:00:00Z").is_ok());

        // Invalid: contains slash
        assert!(validate_workflow_id("2025-10-24/foo").is_err());

        // Invalid: contains path traversal
        assert!(validate_workflow_id("../2025-10-24T10:00:00Z").is_err());

        // Invalid: not ISO 8601
        assert!(validate_workflow_id("not-a-timestamp").is_err());
    }

    #[test]
    fn test_archive_serialization() {
        let archive = WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: Some("test-session".to_string()),
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: "2025-10-24T10:00:00Z".to_string(),
                end_time: Some("2025-10-24T10:15:00Z".to_string()),
                duration_seconds: 900,
                tokens: TokenTotals {
                    input: 1000,
                    output: 500,
                    cache_creation: 200,
                    cache_read: 300,
                    assistant_turns: 5,
                },
                bash_commands: vec![],
                file_modifications: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: "2025-10-24T10:00:00Z".to_string(),
            }],
            totals: WorkflowTotals {
                tokens: TokenTotals {
                    input: 1000,
                    output: 500,
                    cache_creation: 200,
                    cache_read: 300,
                    assistant_turns: 5,
                },
                bash_commands: 0,
                file_modifications: 0,
                unique_files: 0,
                unique_commands: 0,
            },
        };

        // Serialize
        let json = serde_json::to_string(&archive).unwrap();

        // Deserialize
        let deserialized: WorkflowArchive = serde_json::from_str(&json).unwrap();

        // Verify round-trip
        assert_eq!(archive, deserialized);
    }

    #[test]
    fn test_from_metrics() {
        let metrics = UnifiedMetrics {
            hook_metrics: HookMetrics::default(),
            token_metrics: TokenMetrics::default(),
            state_transitions: vec![StateTransitionEvent {
                timestamp: "2025-10-24T10:00:00Z".to_string(),
                workflow_id: Some("2025-10-24T10:00:00Z".to_string()),
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                phase: "spec".to_string(),
                mode: "discovery".to_string(),
            }],
            session_id: Some("test-session".to_string()),
            phase_metrics: vec![PhaseMetrics {
                phase_name: "spec".to_string(),
                start_time: "2025-10-24T10:00:00Z".to_string(),
                end_time: Some("2025-10-24T10:15:00Z".to_string()),
                duration_seconds: 900,
                token_metrics: TokenMetrics {
                    total_input_tokens: 1000,
                    total_output_tokens: 500,
                    total_cache_creation_tokens: 200,
                    total_cache_read_tokens: 300,
                    assistant_turns: 5,
                },
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            git_commits: vec![],
        };

        let archive = WorkflowArchive::from_metrics(&metrics, "2025-10-24T10:00:00Z").unwrap();

        assert_eq!(archive.workflow_id, "2025-10-24T10:00:00Z");
        assert_eq!(archive.mode, "discovery");
        assert_eq!(archive.session_id, Some("test-session".to_string()));
        assert_eq!(archive.phases.len(), 1);
        assert_eq!(archive.transitions.len(), 1);
    }

    #[test]
    fn test_write_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive = WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: None,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        };

        write_archive(&archive, temp_dir.path()).unwrap();

        let archive_path = temp_dir
            .path()
            .join("archive")
            .join("2025-10-24T10:00:00Z.json");
        assert!(archive_path.exists());

        // Verify content
        let content = fs::read_to_string(&archive_path).unwrap();
        let loaded: WorkflowArchive = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded, archive);
    }

    #[test]
    fn test_write_archive_duplicate() {
        let temp_dir = TempDir::new().unwrap();
        let archive = WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: None,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        };

        // First write succeeds
        write_archive(&archive, temp_dir.path()).unwrap();

        // Second write fails
        let result = write_archive(&archive, temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_read_archives() {
        let temp_dir = TempDir::new().unwrap();

        // Write 2 archives
        let archive1 = WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: None,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        };
        write_archive(&archive1, temp_dir.path()).unwrap();

        let archive2 = WorkflowArchive {
            workflow_id: "2025-10-24T14:00:00Z".to_string(),
            mode: "execution".to_string(),
            completed_at: "2025-10-24T16:00:00Z".to_string(),
            session_id: None,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        };
        write_archive(&archive2, temp_dir.path()).unwrap();

        // Read all archives
        let archives = read_archives(temp_dir.path()).unwrap();
        assert_eq!(archives.len(), 2);
    }

    #[test]
    fn test_read_archives_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        // No archives - should return empty vec
        let archives = read_archives(temp_dir.path()).unwrap();
        assert!(archives.is_empty());
    }

    #[test]
    fn test_read_archives_skip_corrupted() {
        let temp_dir = TempDir::new().unwrap();

        // Create valid archive
        let archive = WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: None,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        };
        write_archive(&archive, temp_dir.path()).unwrap();

        // Create corrupted archive
        let archive_dir = temp_dir.path().join("archive");
        fs::write(archive_dir.join("corrupted.json"), "not valid json").unwrap();

        // Should read 1 valid archive, skip corrupted
        let archives = read_archives(temp_dir.path()).unwrap();
        assert_eq!(archives.len(), 1);
    }
}
