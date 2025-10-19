use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

use crate::storage::FileStorage;

/// Detect project type and route to appropriate init workflow
pub fn init_project(storage: &FileStorage) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let project_type = detect_project_type(&current_dir)?;

    match project_type {
        ProjectType::Greenfield => {
            println!("Detected greenfield project (no non-.md files found)");
            println!("Starting init-greenfield workflow...\n");
            crate::commands::start_workflow("init-greenfield", storage)?;
        }
        ProjectType::Retrofit => {
            println!("Detected existing project (non-.md files found)");
            println!("Starting init-retrofit workflow...\n");
            crate::commands::start_workflow("init-retrofit", storage)?;
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum ProjectType {
    Greenfield,
    Retrofit,
}

/// Detect whether this is a greenfield or retrofit scenario
///
/// Logic: If any non-.md files exist (excluding .git/ and .hegel/), it's a retrofit.
/// Otherwise, it's greenfield (empty or docs-only).
fn detect_project_type(project_dir: &Path) -> Result<ProjectType> {
    // Walk the directory tree looking for non-.md files
    for entry in WalkDir::new(project_dir)
        .max_depth(10) // Reasonable depth limit
        .into_iter()
        .filter_entry(|e| {
            // Skip .git and .hegel directories
            let path = e.path();
            let is_git = path.components().any(|c| c.as_os_str() == ".git");
            let is_hegel = path.components().any(|c| c.as_os_str() == ".hegel");
            !is_git && !is_hegel
        })
    {
        let entry = entry.context("Failed to read directory entry")?;

        // Only check files, not directories
        if entry.file_type().is_file() {
            let path = entry.path();

            // Check if it's NOT a markdown file
            if !is_markdown_file(path) {
                return Ok(ProjectType::Retrofit);
            }
        }
    }

    // No non-.md files found → greenfield
    Ok(ProjectType::Greenfield)
}

/// Check if a file is a markdown file
fn is_markdown_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_greenfield_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Greenfield
        );
    }

    #[test]
    fn test_detect_greenfield_markdown_only() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("VISION.md"), "Vision").unwrap();

        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Greenfield
        );
    }

    #[test]
    fn test_detect_retrofit_with_code_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Retrofit
        );
    }

    #[test]
    fn test_detect_retrofit_with_config_file() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();

        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Retrofit
        );
    }

    #[test]
    fn test_ignores_git_directory() {
        let temp_dir = TempDir::new().unwrap();
        // Create .git directory with files
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "git config").unwrap();

        // Should still be greenfield (git files ignored)
        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Greenfield
        );
    }

    #[test]
    fn test_ignores_hegel_directory() {
        let temp_dir = TempDir::new().unwrap();
        // Create .hegel directory with files
        let hegel_dir = temp_dir.path().join(".hegel");
        fs::create_dir(&hegel_dir).unwrap();
        fs::write(hegel_dir.join("state.json"), "{}").unwrap();

        // Should still be greenfield (hegel files ignored)
        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Greenfield
        );
    }

    #[test]
    fn test_detects_nested_code_files() {
        let temp_dir = TempDir::new().unwrap();
        // Create nested structure
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        assert_eq!(
            detect_project_type(temp_dir.path()).unwrap(),
            ProjectType::Retrofit
        );
    }
}
