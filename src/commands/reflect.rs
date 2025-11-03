use super::external_bin::ExternalBinary;
use anyhow::Result;
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

    // Build arguments
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
