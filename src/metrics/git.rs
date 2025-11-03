// Git commit tracking and attribution for workflow phases

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Git commit metadata with diff statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitCommit {
    /// Commit SHA (abbreviated, 7 chars)
    pub hash: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Commit message (first line only)
    pub message: String,
    /// Author name
    pub author: String,
    /// Number of files changed
    pub files_changed: usize,
    /// Lines inserted
    pub insertions: usize,
    /// Lines deleted
    pub deletions: usize,
}

/// Check if a git repository exists at the project root
///
/// Returns `true` if a git repository can be opened at the parent directory
/// of the provided state directory (assumed to be `.hegel`).
///
/// Never panics or propagates errors - returns `false` for any failure case.
pub fn has_git_repository(state_dir: &Path) -> bool {
    let project_root = match state_dir.parent() {
        Some(p) => p,
        None => return false,
    };

    git2::Repository::open(project_root).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_has_git_repository_detects_valid_repo() {
        // Create temporary directory with git repo
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let state_dir = project_root.join(".hegel");
        std::fs::create_dir(&state_dir).unwrap();

        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(project_root)
            .output()
            .unwrap();

        assert!(has_git_repository(&state_dir));
    }

    #[test]
    fn test_has_git_repository_returns_false_for_no_repo() {
        // Create temporary directory without git repo
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let state_dir = project_root.join(".hegel");
        std::fs::create_dir(&state_dir).unwrap();

        assert!(!has_git_repository(&state_dir));
    }

    #[test]
    fn test_has_git_repository_handles_invalid_path() {
        // Non-existent path
        let invalid_path = Path::new("/non/existent/path/.hegel");
        assert!(!has_git_repository(invalid_path));
    }

    #[test]
    fn test_has_git_repository_handles_root_path() {
        // Path with no parent
        let root_path = Path::new("/");
        assert!(!has_git_repository(root_path));
    }

    #[test]
    fn test_git_commit_construction() {
        let commit = GitCommit {
            hash: "a310c04".to_string(),
            timestamp: "2025-11-02T17:50:00Z".to_string(),
            message: "fix(lib): include test_helpers module".to_string(),
            author: "Emily Madum".to_string(),
            files_changed: 4,
            insertions: 21,
            deletions: 15,
        };

        assert_eq!(commit.hash, "a310c04");
        assert_eq!(commit.hash.len(), 7);
        assert_eq!(commit.files_changed, 4);
        assert_eq!(commit.insertions, 21);
        assert_eq!(commit.deletions, 15);
    }

    #[test]
    fn test_git_commit_serialization() {
        let commit = GitCommit {
            hash: "abc1234".to_string(),
            timestamp: "2025-01-01T10:00:00Z".to_string(),
            message: "test commit".to_string(),
            author: "Test Author".to_string(),
            files_changed: 2,
            insertions: 10,
            deletions: 5,
        };

        let json = serde_json::to_string(&commit).unwrap();
        let deserialized: GitCommit = serde_json::from_str(&json).unwrap();

        assert_eq!(commit, deserialized);
    }

    #[test]
    fn test_git_commit_default_values() {
        let commit = GitCommit {
            hash: String::new(),
            timestamp: String::new(),
            message: String::new(),
            author: String::new(),
            files_changed: 0,
            insertions: 0,
            deletions: 0,
        };

        assert_eq!(commit.files_changed, 0);
        assert_eq!(commit.insertions, 0);
        assert_eq!(commit.deletions, 0);
    }
}
