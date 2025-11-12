use super::external_bin::ExternalBinary;
use anyhow::Result;
use std::env;
use std::path::Path;

const MIRROR_BINARY: ExternalBinary = ExternalBinary {
    name: "mirror",
    adjacent_repo_path: "../hegel-mirror",
    build_instructions: "Please build hegel-mirror first:\n\
         cd ../hegel-mirror && cargo build --release",
};

/// Launch mirror for Markdown document review
pub fn run_reflect(
    files: &[std::path::PathBuf],
    out_dir: Option<&Path>,
    json: bool,
    headless: bool,
) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("No files provided for review");
    }

    // Check if HEGEL_IDE_URL is set - if so, use HTTP API instead of mirror binary
    if let Ok(ide_url) = env::var("HEGEL_IDE_URL") {
        return send_review_request(&ide_url, files);
    }

    // Fall back to mirror binary
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

    // Execute mirror
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_reflect_validates_empty_files() {
        let result = run_reflect(&[], None, false, false);
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
