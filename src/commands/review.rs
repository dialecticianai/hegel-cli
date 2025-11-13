use crate::storage::reviews::{
    read_hegel_reviews, write_hegel_reviews, HegelReviewEntry, ReviewComment,
};
use crate::storage::FileStorage;
use anyhow::{Context, Result};
use std::io::{self, BufRead, IsTerminal};
use std::path::{Path, PathBuf};

/// Handle review command - read or write reviews for files
pub fn handle_review(file_paths: &[PathBuf], storage: &FileStorage) -> Result<()> {
    // Check if stdin is available and has data (write mode)
    let stdin_available = !io::stdin().is_terminal();

    if stdin_available {
        // Write mode - only works with single file for now
        if file_paths.len() != 1 {
            anyhow::bail!("Write mode only supports a single file");
        }
        let resolved_path = resolve_file_path(&file_paths[0])?;

        // Try write mode, but if there's no data, fall back to read mode
        match write_reviews(&resolved_path, storage) {
            Ok(_) => {}
            Err(e) if e.to_string().contains("No valid review comments found") => {
                // No data in stdin, treat as read mode
                read_reviews_for_file(&resolved_path, storage)?;
            }
            Err(e) => return Err(e),
        }
        return Ok(());
    }

    // Read mode - resolve all file paths and return existing reviews
    let resolved_paths: Vec<PathBuf> = file_paths
        .iter()
        .map(|p| resolve_file_path(p))
        .collect::<Result<Vec<_>>>()?;

    for path in &resolved_paths {
        read_reviews_for_file(path, storage)?;
    }

    Ok(())
}

/// Resolve file path with optional .md extension
/// Tries path as-is first, then with .md appended
fn resolve_file_path(file_path: &Path) -> Result<std::path::PathBuf> {
    // Try the path as-is
    if file_path.exists() {
        return Ok(file_path.to_path_buf());
    }

    // Try with .md extension
    let with_md = file_path.with_extension("md");
    if with_md.exists() {
        return Ok(with_md);
    }

    // Neither exists
    anyhow::bail!(
        "File not found: {} (also tried with .md extension)",
        file_path.display()
    )
}

/// Write mode: parse JSONL from stdin and save to reviews.json
fn write_reviews(file_path: &Path, storage: &FileStorage) -> Result<()> {
    // Get project root
    let hegel_dir = storage.state_dir();

    // Compute relative path (uses CWD for explicit state-dir, project root otherwise)
    let relative_path = storage.compute_relative_path(file_path).with_context(|| {
        format!(
            "Failed to compute relative path for {}",
            file_path.display()
        )
    })?;

    // Parse JSONL from stdin
    let stdin = io::stdin();
    let mut comments = Vec::new();

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read line from stdin")?;
        if line.trim().is_empty() {
            continue;
        }

        let comment: ReviewComment =
            serde_json::from_str(&line).context("Failed to parse ReviewComment from JSONL")?;
        comments.push(comment);
    }

    if comments.is_empty() {
        anyhow::bail!("No valid review comments found in stdin");
    }

    // Create new review entry
    let entry = HegelReviewEntry {
        comments: comments.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        session_id: None,
    };

    // Load existing reviews
    let mut reviews = read_hegel_reviews(hegel_dir).context("Failed to read existing reviews")?;

    // Append new entry
    reviews
        .entry(relative_path.clone())
        .or_insert_with(Vec::new)
        .push(entry);

    // Write back
    write_hegel_reviews(hegel_dir, &reviews).context("Failed to write reviews")?;

    // Output success JSON
    let result = serde_json::json!({
        "file": relative_path,
        "comments": comments.len()
    });
    println!("{}", serde_json::to_string(&result)?);

    Ok(())
}

/// Read mode: display existing reviews as JSONL
fn read_reviews_for_file(file_path: &Path, storage: &FileStorage) -> Result<()> {
    // Get project root
    let hegel_dir = storage.state_dir();

    // Compute relative path (uses CWD for explicit state-dir, project root otherwise)
    let relative_path = storage.compute_relative_path(file_path).with_context(|| {
        format!(
            "Failed to compute relative path for {}",
            file_path.display()
        )
    })?;

    // Load reviews
    let reviews = read_hegel_reviews(hegel_dir).context("Failed to read reviews")?;

    // Get reviews for this file
    if let Some(entries) = reviews.get(&relative_path) {
        // Flatten all comments from all entries
        for entry in entries {
            for comment in &entry.comments {
                // Output each comment as JSONL
                let json = serde_json::to_string(comment)?;
                println!("{}", json);
            }
        }
    }

    // If no reviews for file, output nothing (empty output)
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::reviews::{Position, ReviewComment, SelectionRange};
    use crate::test_helpers::*;
    use std::fs;

    fn test_review_comment(file: &str, text: &str, comment: &str) -> ReviewComment {
        ReviewComment {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("test-session".to_string()),
            file: file.to_string(),
            selection: SelectionRange {
                start: Position { line: 1, col: 0 },
                end: Position { line: 1, col: 10 },
            },
            text: text.to_string(),
            comment: comment.to_string(),
        }
    }

    #[test]
    fn test_resolve_file_path_exact_match() {
        let (temp_dir, _storage) = test_storage();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "test content").unwrap();

        let resolved = resolve_file_path(&test_file).unwrap();
        assert_eq!(resolved, test_file);
    }

    #[test]
    fn test_resolve_file_path_adds_md_extension() {
        let (temp_dir, _storage) = test_storage();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "test content").unwrap();

        let path_without_ext = temp_dir.path().join("test");
        let resolved = resolve_file_path(&path_without_ext).unwrap();
        assert_eq!(resolved, test_file);
    }

    #[test]
    fn test_resolve_file_path_not_found() {
        let (temp_dir, _storage) = test_storage();
        let nonexistent = temp_dir.path().join("nonexistent.md");

        let result = resolve_file_path(&nonexistent);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));
    }

    #[test]
    fn test_write_reviews_creates_entry() {
        let (_temp_dir, storage) = test_storage();

        // Create test comment and write directly via storage layer
        // Note: Testing write_reviews() directly would require mocking stdin
        let comment = test_review_comment("test.md", "selected", "test comment");
        let mut reviews = crate::storage::reviews::read_hegel_reviews(storage.state_dir()).unwrap();
        let entry = crate::storage::reviews::HegelReviewEntry {
            comments: vec![comment.clone()],
            timestamp: chrono::Utc::now().to_rfc3339(),
            session_id: None,
        };
        reviews
            .entry("test.md".to_string())
            .or_insert_with(Vec::new)
            .push(entry);
        crate::storage::reviews::write_hegel_reviews(storage.state_dir(), &reviews).unwrap();

        // Verify written
        let loaded = crate::storage::reviews::read_hegel_reviews(storage.state_dir()).unwrap();
        assert!(loaded.contains_key("test.md"));
        assert_eq!(loaded["test.md"][0].comments[0].comment, "test comment");
    }

    #[test]
    fn test_read_reviews_for_nonexistent_file() {
        let (temp_dir, storage) = test_storage();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "test content").unwrap();

        // Read reviews for file with no reviews - should succeed with empty output
        let result = read_reviews_for_file(&test_file, &storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_reviews_for_file_with_reviews() {
        let (temp_dir, storage) = test_storage();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "test content").unwrap();

        // Write review directly via storage layer
        let comment = test_review_comment("test.md", "selected text", "needs improvement");
        let entry = crate::storage::reviews::HegelReviewEntry {
            comments: vec![comment],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("test-session".to_string()),
        };
        let mut reviews = std::collections::HashMap::new();
        reviews.insert("test.md".to_string(), vec![entry]);
        crate::storage::reviews::write_hegel_reviews(storage.state_dir(), &reviews).unwrap();

        // Read reviews - should output JSONL (testing via storage layer)
        let loaded = crate::storage::reviews::read_hegel_reviews(storage.state_dir()).unwrap();
        assert!(loaded.contains_key("test.md"));
        assert_eq!(loaded["test.md"][0].comments.len(), 1);
        assert_eq!(
            loaded["test.md"][0].comments[0].comment,
            "needs improvement"
        );
    }

    #[test]
    fn test_read_reviews_multiple_entries() {
        let (temp_dir, storage) = test_storage();
        let test_file = temp_dir.path().join("doc.md");
        fs::write(&test_file, "content").unwrap();

        // Create two separate review sessions
        let comment1 = test_review_comment("doc.md", "line 1", "first review");
        let comment2 = test_review_comment("doc.md", "line 2", "second review");

        let entry1 = crate::storage::reviews::HegelReviewEntry {
            comments: vec![comment1],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("session-1".to_string()),
        };

        let entry2 = crate::storage::reviews::HegelReviewEntry {
            comments: vec![comment2],
            timestamp: "2025-01-01T01:00:00Z".to_string(),
            session_id: Some("session-2".to_string()),
        };

        let mut reviews = std::collections::HashMap::new();
        reviews.insert("doc.md".to_string(), vec![entry1, entry2]);
        crate::storage::reviews::write_hegel_reviews(storage.state_dir(), &reviews).unwrap();

        // Verify both entries are stored
        let loaded = crate::storage::reviews::read_hegel_reviews(storage.state_dir()).unwrap();
        assert_eq!(loaded["doc.md"].len(), 2);
        assert_eq!(loaded["doc.md"][0].comments[0].comment, "first review");
        assert_eq!(loaded["doc.md"][1].comments[0].comment, "second review");
    }

    #[test]
    fn test_read_reviews_multiple_comments_per_entry() {
        let (temp_dir, storage) = test_storage();
        let test_file = temp_dir.path().join("spec.md");
        fs::write(&test_file, "spec content").unwrap();

        let comment1 = test_review_comment("spec.md", "section 1", "comment 1");
        let comment2 = test_review_comment("spec.md", "section 2", "comment 2");

        let entry = crate::storage::reviews::HegelReviewEntry {
            comments: vec![comment1, comment2],
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: Some("test-session".to_string()),
        };

        let mut reviews = std::collections::HashMap::new();
        reviews.insert("spec.md".to_string(), vec![entry]);
        crate::storage::reviews::write_hegel_reviews(storage.state_dir(), &reviews).unwrap();

        // Verify multiple comments in single entry
        let loaded = crate::storage::reviews::read_hegel_reviews(storage.state_dir()).unwrap();
        assert_eq!(loaded["spec.md"][0].comments.len(), 2);
        assert_eq!(loaded["spec.md"][0].comments[0].comment, "comment 1");
        assert_eq!(loaded["spec.md"][0].comments[1].comment, "comment 2");
    }
}
