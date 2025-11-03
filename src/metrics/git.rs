// Git commit tracking and attribution for workflow phases

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

#[cfg(test)]
mod tests {
    use super::*;

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
