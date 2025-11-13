use super::external_bin::ExternalBinary;
use crate::storage::reviews::read_hegel_reviews;
use crate::storage::FileStorage;
use anyhow::Result;
use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

const MIRROR_BINARY: ExternalBinary = ExternalBinary {
    name: "mirror",
    adjacent_repo_path: "../hegel-mirror",
    build_instructions: "Please build hegel-mirror first:\n\
         cd ../hegel-mirror && cargo build --release",
};

// Polling configuration
const POLL_INTERVAL_MS: u64 = 200;
const POLL_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// Launch mirror for Markdown document review
pub fn run_reflect(
    files: &[std::path::PathBuf],
    out_dir: Option<&Path>,
    storage: &FileStorage,
    immediate: bool,
    json: bool,
    headless: bool,
) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("No files provided for review");
    }

    // Capture start time for polling
    let start_time = chrono::Utc::now();

    // Check if HEGEL_IDE_URL is set - if so, use HTTP API instead of mirror binary
    if let Ok(ide_url) = env::var("HEGEL_IDE_URL") {
        send_review_request(&ide_url, files)?;

        // If immediate mode, return after sending request
        if immediate {
            return Ok(());
        }

        // Otherwise poll for reviews
        return poll_for_reviews(files, storage, &start_time, POLL_TIMEOUT_SECS);
    }

    // Fall back to mirror binary - always runs normally, ignore immediate flag
    let mut args: Vec<String> = files.iter().map(|f| f.display().to_string()).collect();

    if let Some(dir) = out_dir {
        args.push("--out-dir".to_string());
        args.push(dir.display().to_string());
    }

    if json {
        args.push("--json".to_string());
    }

    if headless {
        args.push("--headless".to_string());
    }

    // Execute mirror (immediate flag doesn't apply to mirror binary)
    MIRROR_BINARY.execute(&args)
}

/// Send review request to Hegel IDE server
fn send_review_request(base_url: &str, files: &[std::path::PathBuf]) -> Result<()> {
    use serde_json::json;

    // Convert all paths to absolute paths
    let absolute_paths: Vec<String> = files
        .iter()
        .map(|p| {
            p.canonicalize()
                .unwrap_or_else(|_| p.to_path_buf())
                .display()
                .to_string()
        })
        .collect();

    // Build request payload
    let payload = json!({
        "files": absolute_paths
    });

    // Make POST request to /review endpoint
    let url = format!("{}/review", base_url.trim_end_matches('/'));
    let response = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(&payload)?;

    // Handle response
    match response.status() {
        200 => {
            println!("✓ Review request sent successfully");
            Ok(())
        }
        404 => {
            let body: serde_json::Value = response.into_json()?;
            if let Some(missing) = body.get("missing").and_then(|m| m.as_array()) {
                let missing_files: Vec<&str> = missing.iter().filter_map(|v| v.as_str()).collect();
                anyhow::bail!("Missing files: {}", missing_files.join(", "));
            } else {
                anyhow::bail!("Some files were not found");
            }
        }
        status => {
            anyhow::bail!("Unexpected response status: {}", status);
        }
    }
}

/// Poll for new reviews with timestamps after start_time
fn poll_for_reviews(
    file_paths: &[std::path::PathBuf],
    storage: &FileStorage,
    start_time: &chrono::DateTime<chrono::Utc>,
    timeout_secs: u64,
) -> Result<()> {
    use anyhow::Context;

    let hegel_dir = storage.state_dir();
    let timeout = Duration::from_secs(timeout_secs);
    let poll_start = std::time::Instant::now();

    // Compute relative paths for all files (uses CWD for explicit state-dir, project root otherwise)
    let relative_paths: Vec<String> = file_paths
        .iter()
        .map(|p| {
            // Resolve file path with optional .md extension
            let resolved = resolve_file_path(p)?;
            storage.compute_relative_path(&resolved)
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_run_reflect_validates_empty_files() {
        let (_temp_dir, storage) = test_storage();
        let result = run_reflect(&[], None, &storage, false, false, false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No files provided"));
    }

    #[test]
    fn test_find_mirror_binary_checks_adjacent_repo() {
        // This test documents the search behavior without requiring mirror to exist
        let result = MIRROR_BINARY.find();
        // Will fail in CI/most environments, but documents expected behavior
        if result.is_ok() {
            let path = result.unwrap();
            assert!(path.to_str().unwrap().contains("mirror"));
        }
    }
}
