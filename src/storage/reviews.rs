//! Reviews management for .hegel/reviews.json
//!
//! Provides types and I/O operations for managing document reviews in Hegel projects.
//! Extracted from hegel-mirror for reuse across tools.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::FileStorage;

/// Review comment with full metadata
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReviewComment {
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub file: String,
    pub selection: SelectionRange,
    pub text: String,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SelectionRange {
    pub start: Position,
    pub end: Position,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl ReviewComment {
    /// Helper constructor for tests - auto-generates timestamp
    #[cfg(test)]
    pub fn new(
        file: String,
        session_id: Option<String>,
        text: String,
        comment: String,
        line_start: usize,
        col_start: usize,
        line_end: usize,
        col_end: usize,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id,
            file,
            selection: SelectionRange {
                start: Position {
                    line: line_start,
                    col: col_start,
                },
                end: Position {
                    line: line_end,
                    col: col_end,
                },
            },
            text,
            comment,
        }
    }
}

/// Single review entry for Hegel projects (stored in .hegel/reviews.json)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HegelReviewEntry {
    pub comments: Vec<ReviewComment>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Map of filename to review entries for Hegel projects
pub type HegelReviewsMap = HashMap<String, Vec<HegelReviewEntry>>;

/// Review storage type detection - determines where reviews are saved
/// TODO: Implement standalone (non-Hegel) project support with sidecar .review.N files
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ReviewStorageType {
    /// Hegel project detected - use .hegel/reviews.json
    Hegel { root: PathBuf },
    /// Standalone project - use sidecar .review.N files (not yet implemented)
    Standalone,
}

/// Detect review storage type from given path (or current working directory)
#[allow(dead_code)]
pub fn detect_review_storage_type_from(start_path: Option<PathBuf>) -> ReviewStorageType {
    match FileStorage::find_project_root_from(start_path) {
        Ok(hegel_dir) => ReviewStorageType::Hegel { root: hegel_dir },
        Err(_) => ReviewStorageType::Standalone,
    }
}

/// Detect review storage type from current working directory
#[allow(dead_code)]
pub fn detect_review_storage_type() -> ReviewStorageType {
    detect_review_storage_type_from(None)
}

/// Read existing .hegel/reviews.json or return empty map
pub fn read_hegel_reviews(hegel_dir: &Path) -> Result<HegelReviewsMap> {
    let reviews_path = hegel_dir.join("reviews.json");

    if !reviews_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&reviews_path).context("Failed to read reviews.json")?;

    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    serde_json::from_str(&content).context("Failed to parse reviews.json")
}

/// Write reviews map atomically to .hegel/reviews.json
pub fn write_hegel_reviews(hegel_dir: &Path, reviews: &HegelReviewsMap) -> Result<()> {
    // Ensure .hegel directory exists
    fs::create_dir_all(hegel_dir).context(format!(
        "Failed to create .hegel directory: {:?}",
        hegel_dir
    ))?;

    let reviews_path = hegel_dir.join("reviews.json");
    let json =
        serde_json::to_string_pretty(reviews).context("Failed to serialize reviews to JSON")?;

    fs::write(&reviews_path, json)
        .context(format!("Failed to write reviews.json: {:?}", reviews_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_hegel_project() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let hegel_dir = temp_dir.path().join(".hegel");
        fs::create_dir(&hegel_dir).unwrap();
        (temp_dir, hegel_dir)
    }

    #[test]
    fn test_detect_hegel_project() {
        let (temp_dir, hegel_dir) = test_hegel_project();

        let storage_type = detect_review_storage_type_from(Some(temp_dir.path().to_path_buf()));

        match storage_type {
            ReviewStorageType::Hegel { root } => {
                let root_canonical = root.canonicalize().unwrap();
                let expected_canonical = hegel_dir.canonicalize().unwrap();
                assert_eq!(root_canonical, expected_canonical);
            }
            ReviewStorageType::Standalone => {
                panic!("Expected Hegel project to be detected");
            }
        }
    }

    #[test]
    fn test_detect_standalone_project() {
        let temp_dir = TempDir::new().unwrap();

        let storage_type = detect_review_storage_type_from(Some(temp_dir.path().to_path_buf()));

        assert_eq!(storage_type, ReviewStorageType::Standalone);
    }

    #[test]
    fn test_read_hegel_reviews_empty() {
        let (_temp_dir, hegel_dir) = test_hegel_project();

        let reviews = read_hegel_reviews(&hegel_dir).unwrap();
        assert!(reviews.is_empty());
    }

    #[test]
    fn test_write_and_read_hegel_reviews() {
        let (_temp_dir, hegel_dir) = test_hegel_project();

        let mut reviews = HashMap::new();
        let entry = HegelReviewEntry {
            comments: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("session123".to_string()),
        };
        reviews.insert("test.md".to_string(), vec![entry.clone()]);

        write_hegel_reviews(&hegel_dir, &reviews).unwrap();

        let read_reviews = read_hegel_reviews(&hegel_dir).unwrap();
        assert_eq!(read_reviews.len(), 1);
        assert_eq!(read_reviews.get("test.md").unwrap()[0], entry);
    }

    #[test]
    fn test_hegel_reviews_multiple_files() {
        let (_temp_dir, hegel_dir) = test_hegel_project();

        let mut reviews = HashMap::new();

        let entry1 = HegelReviewEntry {
            comments: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("session123".to_string()),
        };

        let entry2 = HegelReviewEntry {
            comments: vec![],
            timestamp: "2025-01-01T01:00:00Z".to_string(),
            session_id: Some("session123".to_string()),
        };

        reviews.insert("file1.md".to_string(), vec![entry1.clone()]);
        reviews.insert("file2.md".to_string(), vec![entry2.clone()]);

        write_hegel_reviews(&hegel_dir, &reviews).unwrap();

        let read_reviews = read_hegel_reviews(&hegel_dir).unwrap();
        assert_eq!(read_reviews.len(), 2);
        assert!(read_reviews.contains_key("file1.md"));
        assert!(read_reviews.contains_key("file2.md"));
    }

    #[test]
    fn test_hegel_reviews_multiple_entries_per_file() {
        let (_temp_dir, hegel_dir) = test_hegel_project();

        let mut reviews = HashMap::new();

        let entry1 = HegelReviewEntry {
            comments: vec![],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("session1".to_string()),
        };

        let entry2 = HegelReviewEntry {
            comments: vec![],
            timestamp: "2025-01-01T01:00:00Z".to_string(),
            session_id: Some("session2".to_string()),
        };

        reviews.insert("test.md".to_string(), vec![entry1.clone(), entry2.clone()]);

        write_hegel_reviews(&hegel_dir, &reviews).unwrap();

        let read_reviews = read_hegel_reviews(&hegel_dir).unwrap();
        let entries = read_reviews.get("test.md").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], entry1);
        assert_eq!(entries[1], entry2);
    }

    #[test]
    fn test_review_comment_serialization() {
        let comment = ReviewComment::new(
            "test.md".to_string(),
            Some("session123".to_string()),
            "selected text".to_string(),
            "test comment".to_string(),
            1,
            0,
            1,
            10,
        );

        let json = serde_json::to_string(&comment).unwrap();
        let deserialized: ReviewComment = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.file, "test.md");
        assert_eq!(deserialized.text, "selected text");
        assert_eq!(deserialized.comment, "test comment");
        assert_eq!(deserialized.selection.start.line, 1);
        assert_eq!(deserialized.selection.end.col, 10);
    }
}
