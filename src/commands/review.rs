use crate::storage::reviews::{
    compute_relative_path, read_hegel_reviews, write_hegel_reviews, HegelReviewEntry, ReviewComment,
};
use crate::storage::FileStorage;
use anyhow::{Context, Result};
use std::io::{self, BufRead, IsTerminal};
use std::path::Path;

/// Handle review command - read or write reviews for a file
pub fn handle_review(file_path: &Path, storage: &FileStorage) -> Result<()> {
    // Resolve file path (handle optional .md extension)
    let resolved_path = resolve_file_path(file_path)?;

    // Check if stdin is available and has data (write mode) or not (read mode)
    // We check is_terminal first - if it's a terminal, definitely read mode
    // If it's not a terminal (pipe), try to read to see if there's data
    let stdin_available = !io::stdin().is_terminal();

    if stdin_available {
        // Try write mode, but if there's no data, fall back to read mode
        match write_reviews(&resolved_path, storage) {
            Ok(_) => {}
            Err(e) if e.to_string().contains("No valid review comments found") => {
                // No data in stdin, treat as read mode
                read_reviews_for_file(&resolved_path, storage)?;
            }
            Err(e) => return Err(e),
        }
    } else {
        read_reviews_for_file(&resolved_path, storage)?;
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
