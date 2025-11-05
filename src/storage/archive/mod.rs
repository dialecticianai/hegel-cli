mod aggregation;
mod builder;
mod validation;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::metrics::git::GitCommit;

// Re-export for backwards compatibility

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
    /// Whether this archive was auto-detected from inter-workflow activity
    #[serde(default)]
    pub is_synthetic: bool,
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
    #[serde(default)]
    pub git_commits: Vec<GitCommit>,
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
    #[serde(default)]
    pub git_commits: usize,
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
    use tempfile::TempDir;

    /// Helper to create test archive with default values
    fn test_archive() -> WorkflowArchive {
        WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: None,
            is_synthetic: false,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        }
    }

    #[test]
    fn test_archive_serialization() {
        let archive = WorkflowArchive {
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
                git_commits: vec![],
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
                git_commits: 0,
            },
            ..test_archive()
        };

        // Serialize
        let json = serde_json::to_string(&archive).unwrap();

        // Deserialize
        let deserialized: WorkflowArchive = serde_json::from_str(&json).unwrap();

        // Verify round-trip
        assert_eq!(archive, deserialized);
    }

    #[test]
    fn test_write_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive = test_archive();

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
        let archive = test_archive();

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
        let archive1 = test_archive();
        write_archive(&archive1, temp_dir.path()).unwrap();

        let archive2 = WorkflowArchive {
            workflow_id: "2025-10-24T14:00:00Z".to_string(),
            mode: "execution".to_string(),
            completed_at: "2025-10-24T16:00:00Z".to_string(),
            ..test_archive()
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
        let archive = test_archive();
        write_archive(&archive, temp_dir.path()).unwrap();

        // Create corrupted archive
        let archive_dir = temp_dir.path().join("archive");
        fs::write(archive_dir.join("corrupted.json"), "not valid json").unwrap();

        // Should read 1 valid archive, skip corrupted
        let archives = read_archives(temp_dir.path()).unwrap();
        assert_eq!(archives.len(), 1);
    }

    #[test]
    fn test_is_synthetic_default_false() {
        // Test backward compatibility: archives without is_synthetic load as is_synthetic=false
        let json = r#"{
            "workflow_id": "2025-10-24T10:00:00Z",
            "mode": "discovery",
            "completed_at": "2025-10-24T12:00:00Z",
            "phases": [],
            "transitions": [],
            "totals": {
                "tokens": {"input": 0, "output": 0, "cache_creation": 0, "cache_read": 0, "assistant_turns": 0},
                "bash_commands": 0,
                "file_modifications": 0,
                "unique_files": 0,
                "unique_commands": 0,
                "git_commits": 0
            }
        }"#;

        let archive: WorkflowArchive = serde_json::from_str(json).unwrap();
        assert_eq!(archive.is_synthetic, false);
    }

    #[test]
    fn test_is_synthetic_serialization() {
        // Test is_synthetic=true serializes and deserializes correctly
        let archive = WorkflowArchive {
            is_synthetic: true,
            ..test_archive()
        };

        let json = serde_json::to_string(&archive).unwrap();
        let deserialized: WorkflowArchive = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.is_synthetic, true);
    }
}
