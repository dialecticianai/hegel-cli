use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

use crate::storage::FileStorage;

/// Directories to exclude when detecting project type
/// These are tool/config directories that don't indicate user code
const EXCLUDED_DIRS: &[&str] = &[".git", ".hegel", ".claude"];

/// Detect project type and route to appropriate init workflow
pub fn init_project(storage: &FileStorage, override_type: Option<&str>) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Use override if provided, otherwise auto-detect
    let project_type = match override_type {
        Some("greenfield") => ProjectType::Greenfield,
        Some("retrofit") => ProjectType::Retrofit,
        Some(other) => anyhow::bail!(
            "Invalid project type: {}. Must be 'greenfield' or 'retrofit'",
            other
        ),
        None => detect_project_type(&current_dir)?,
    };

    match project_type {
        ProjectType::Greenfield => {
            if override_type.is_some() {
                println!("Using manual override: greenfield project");
            } else {
                println!("Detected greenfield project (no non-.md files found)");
            }
            println!("Starting init-greenfield workflow...\n");
            crate::commands::start_workflow("init-greenfield", None, storage)?;
        }
        ProjectType::Retrofit => {
            if override_type.is_some() {
                println!("Using manual override: retrofit project");
            } else {
                println!("Detected existing project (non-.md files found)");
            }
            println!("Starting init-retrofit workflow...\n");
            crate::commands::start_workflow("init-retrofit", None, storage)?;
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
/// Logic: If any non-.md files exist (excluding tool directories), it's a retrofit.
/// Otherwise, it's greenfield (empty or docs-only).
fn detect_project_type(project_dir: &Path) -> Result<ProjectType> {
    // Walk the directory tree looking for non-.md files
    for entry in WalkDir::new(project_dir)
        .max_depth(10) // Reasonable depth limit
        .into_iter()
        .filter_entry(|e| {
            // Skip excluded directories (tool/config directories)
            let path = e.path();
            let is_excluded = path.components().any(|c| {
                EXCLUDED_DIRS
                    .iter()
                    .any(|&excluded| c.as_os_str() == excluded)
            });
            !is_excluded
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
    use crate::test_helpers::test_storage;
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
    fn test_ignores_excluded_directories() {
        // Test each excluded directory to ensure they're all ignored
        for excluded_dir in EXCLUDED_DIRS {
            let temp_dir = TempDir::new().unwrap();

            // Create excluded directory with a non-.md file
            let dir = temp_dir.path().join(excluded_dir);
            fs::create_dir(&dir).unwrap();
            fs::write(dir.join("config.json"), "{}").unwrap();

            // Should still be greenfield (excluded directory ignored)
            assert_eq!(
                detect_project_type(temp_dir.path()).unwrap(),
                ProjectType::Greenfield,
                "Failed to ignore {} directory",
                excluded_dir
            );
        }
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

    #[test]
    fn test_manual_override_greenfield() {
        let (_temp_dir, storage) = test_storage();

        // Even with code files present, override should force greenfield
        let current_dir = std::env::current_dir().unwrap();
        let src_dir = current_dir.join("src");
        if !src_dir.exists() {
            fs::create_dir(&src_dir).ok();
            fs::write(src_dir.join("test.rs"), "fn test() {}").ok();
        }

        let result = init_project(&storage, Some("greenfield"));
        // Should start init-greenfield workflow
        assert!(result.is_ok());
    }

    #[test]
    fn test_manual_override_retrofit() {
        let (_temp_dir, storage) = test_storage();

        // Even with no code files, override should force retrofit
        let result = init_project(&storage, Some("retrofit"));
        // Should start init-retrofit workflow
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_override() {
        let (_temp_dir, storage) = test_storage();

        let result = init_project(&storage, Some("invalid"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid project type"));
    }
}
