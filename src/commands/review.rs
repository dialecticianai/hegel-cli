use crate::storage::reviews::{
    compute_relative_path, read_hegel_reviews, write_hegel_reviews, HegelReviewEntry, ReviewComment,
};
use crate::storage::FileStorage;
use anyhow::{Context, Result};
use std::io::{self, BufRead, IsTerminal};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Handle review command - read or write reviews for files
pub fn handle_review(file_paths: &[PathBuf], storage: &FileStorage, immediate: bool) -> Result<()> {
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

    // Read mode - resolve all file paths
    let resolved_paths: Vec<PathBuf> = file_paths
        .iter()
        .map(|p| resolve_file_path(p))
        .collect::<Result<Vec<_>>>()?;

    if immediate {
        // Immediate mode - return existing reviews
        for path in &resolved_paths {
            read_reviews_for_file(path, storage)?;
        }
    } else {
        // Polling mode - wait for new reviews
        let start_time = chrono::Utc::now();
        poll_for_reviews(&resolved_paths, storage, &start_time, POLL_TIMEOUT_SECS)?;
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

    // Compute relative path (handles canonicalization internally)
    let relative_path = compute_relative_path(hegel_dir, file_path).with_context(|| {
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

    // Compute relative path (handles canonicalization internally)
    let relative_path = compute_relative_path(hegel_dir, file_path).with_context(|| {
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

// Polling configuration
const POLL_INTERVAL_MS: u64 = 200;
const POLL_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// Poll for new reviews with timestamps after start_time
fn poll_for_reviews(
    file_paths: &[PathBuf],
    storage: &FileStorage,
    start_time: &chrono::DateTime<chrono::Utc>,
    timeout_secs: u64,
) -> Result<()> {
    let hegel_dir = storage.state_dir();
    let timeout = Duration::from_secs(timeout_secs);
    let poll_start = std::time::Instant::now();

    // Compute relative paths for all files
    let relative_paths: Vec<String> = file_paths
        .iter()
        .map(|p| compute_relative_path(hegel_dir, p))
        .collect::<Result<Vec<_>>>()?;

    loop {
        // Check timeout
        if poll_start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for reviews after {} seconds", timeout_secs);
        }

        // Load current reviews
        let reviews = read_hegel_reviews(hegel_dir).context("Failed to read reviews")?;

        // Check each file for new reviews
        let mut found_reviews = Vec::new();
        for (path, relative_path) in file_paths.iter().zip(&relative_paths) {
            if let Some(entries) = reviews.get(relative_path) {
                // Look for entries with timestamp > start_time
                for entry in entries {
                    if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                        if entry_time.with_timezone(&chrono::Utc) > *start_time {
                            // Found new review for this file
                            found_reviews.push((path, entry));
                            break; // Only need one review per file
                        }
                    }
                }
            }
        }

        // If we found reviews for all files, output and exit
        if found_reviews.len() == file_paths.len() {
            for (_path, entry) in found_reviews {
                for comment in &entry.comments {
                    let json = serde_json::to_string(comment)?;
                    println!("{}", json);
                }
            }
            return Ok(());
        }

        // Sleep before next poll
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::reviews::{Position, ReviewComment, SelectionRange};
    use crate::test_helpers::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

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

    #[test]
    fn test_poll_for_reviews_finds_new_review() {
        let (_temp_dir, storage) = test_storage();
        let test_file = storage.state_dir().parent().unwrap().join("test.md");
        fs::write(&test_file, "content").unwrap();

        let start_time = chrono::Utc::now();

        // Spawn thread to write review after a short delay
        let hegel_dir = storage.state_dir().to_path_buf();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            // Write review with timestamp after start_time
            let comment = ReviewComment {
                timestamp: chrono::Utc::now().to_rfc3339(),
                session_id: Some("test".to_string()),
                file: "test.md".to_string(),
                selection: SelectionRange {
                    start: Position { line: 1, col: 0 },
                    end: Position { line: 1, col: 5 },
                },
                text: "text".to_string(),
                comment: "new review".to_string(),
            };
            let entry = crate::storage::reviews::HegelReviewEntry {
                comments: vec![comment],
                timestamp: chrono::Utc::now().to_rfc3339(),
                session_id: Some("test".to_string()),
            };
            let mut reviews = std::collections::HashMap::new();
            reviews.insert("test.md".to_string(), vec![entry]);
            crate::storage::reviews::write_hegel_reviews(&hegel_dir, &reviews).unwrap();
        });

        // Poll for new reviews - should find the one written above
        let result = poll_for_reviews(&[test_file.clone()], &storage, &start_time, 2);
        if let Err(e) = &result {
            eprintln!("Poll error: {}", e);
        }
        assert!(result.is_ok(), "Expected success but got: {:?}", result);
    }

    #[test]
    fn test_poll_for_reviews_timeout() {
        let (temp_dir, storage) = test_storage();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "content").unwrap();

        let start_time = chrono::Utc::now();

        // Poll with very short timeout - should timeout
        let result = poll_for_reviews(&[test_file.clone()], &storage, &start_time, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }

    #[test]
    fn test_poll_ignores_old_reviews() {
        let (temp_dir, storage) = test_storage();
        let test_file = temp_dir.path().join("old.md");
        fs::write(&test_file, "content").unwrap();

        // Write review with old timestamp
        let old_comment = ReviewComment {
            timestamp: "2020-01-01T00:00:00Z".to_string(),
            session_id: Some("old".to_string()),
            file: "old.md".to_string(),
            selection: SelectionRange {
                start: Position { line: 1, col: 0 },
                end: Position { line: 1, col: 5 },
            },
            text: "old".to_string(),
            comment: "old review".to_string(),
        };
        let entry = crate::storage::reviews::HegelReviewEntry {
            comments: vec![old_comment],
            timestamp: "2020-01-01T00:00:00Z".to_string(),
            session_id: Some("old".to_string()),
        };
        let mut reviews = std::collections::HashMap::new();
        reviews.insert("old.md".to_string(), vec![entry]);
        crate::storage::reviews::write_hegel_reviews(storage.state_dir(), &reviews).unwrap();

        let start_time = chrono::Utc::now();

        // Poll with short timeout - should timeout because review is too old
        let result = poll_for_reviews(&[test_file.clone()], &storage, &start_time, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }
}
