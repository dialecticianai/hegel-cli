use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

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

    // Look for mirror binary in known locations
    let mirror_bin = find_mirror_binary()?;

    // Build command
    let mut cmd = Command::new(&mirror_bin);
    cmd.args(files);

    if let Some(dir) = out_dir {
        cmd.arg("--out-dir").arg(dir);
    }

    if json {
        cmd.arg("--json");
    }

    if headless {
        cmd.arg("--headless");
    }

    // Pass through HEGEL_SESSION_ID if present
    if let Ok(session_id) = std::env::var("HEGEL_SESSION_ID") {
        cmd.env("HEGEL_SESSION_ID", session_id);
    }

    // Execute mirror
    let status = cmd.status().context("Failed to execute mirror")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Find mirror binary in known locations
fn find_mirror_binary() -> Result<std::path::PathBuf> {
    // Check common locations
    let candidates = vec![
        // Adjacent repo (development)
        "../hegel-mirror/target/release/mirror",
        // System PATH
        "mirror",
    ];

    for candidate in &candidates {
        let path = std::path::Path::new(candidate);
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }

    // Try to find in PATH
    if let Ok(output) = Command::new("which").arg("mirror").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = path_str.trim();
            if !path.is_empty() {
                return Ok(std::path::PathBuf::from(path));
            }
        }
    }

    anyhow::bail!(
        "mirror binary not found. Please build hegel-mirror first:\n\
         cd ../hegel-mirror && cargo build --release"
    )
}
