//! Filesystem scanning and classification of markdown files.

use anyhow::Result;
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use crate::commands::util::is_markdown_file;

/// File category classification
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FileCategory {
    Ddd { ephemeral: bool },
    Regular,
}

/// Markdown file with metadata
#[derive(Debug, Clone)]
pub(crate) struct MarkdownFile {
    pub(crate) path: PathBuf,
    pub(crate) category: FileCategory,
    pub(crate) lines: Option<usize>,
    pub(crate) size_bytes: u64,
    pub(crate) last_modified: DateTime<Utc>,
}

/// Scan directory for markdown files with gitignore support
pub(crate) fn scan_markdown_files() -> Result<Vec<MarkdownFile>> {
    let mut files = Vec::new();
    let cwd = std::env::current_dir()?;

    // Build walker with gitignore support
    let walker = WalkBuilder::new(&cwd)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .build();

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // Skip permission errors
        };

        // Only process files (not directories)
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }

        let path = entry.path();

        // Check if it's a markdown file
        if !is_markdown_file(path) {
            continue;
        }

        // Classify the file
        let category = classify_file(path);

        // Get metadata
        let metadata = std::fs::metadata(path)?;
        let size_bytes = metadata.len();
        let last_modified = metadata.modified()?.into();

        // Count lines
        let lines = count_lines(path);

        files.push(MarkdownFile {
            path: path.strip_prefix(&cwd).unwrap_or(path).to_path_buf(),
            category,
            lines,
            size_bytes,
            last_modified,
        });
    }

    Ok(files)
}

/// Classify a file as DDD or regular
fn classify_file(path: &Path) -> FileCategory {
    let path_str = path.to_string_lossy();

    // Check for HANDOFF.md (ephemeral DDD)
    if path.file_name().and_then(|n| n.to_str()) == Some("HANDOFF.md") {
        return FileCategory::Ddd { ephemeral: true };
    }

    // Check for .ddd/ prefix
    if path_str.contains("/.ddd/") || path_str.starts_with(".ddd/") {
        return FileCategory::Ddd { ephemeral: false };
    }

    // Check for toys/ prefix
    if path_str.starts_with("toys/") {
        return FileCategory::Ddd { ephemeral: false };
    }

    FileCategory::Regular
}

/// Count lines in a file
fn count_lines(path: &Path) -> Option<usize> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(content.lines().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("README.md")));
        assert!(is_markdown_file(Path::new("path/to/file.md")));
        assert!(!is_markdown_file(Path::new("README.txt")));
        assert!(!is_markdown_file(Path::new("file")));
    }

    #[test]
    fn test_classify_ddd_directory() {
        let path = Path::new(".ddd/SPEC.md");
        assert!(matches!(
            classify_file(path),
            FileCategory::Ddd { ephemeral: false }
        ));
    }

    #[test]
    fn test_classify_ddd_nested() {
        let path = Path::new(".ddd/feat/workflow-stash/SPEC.md");
        assert!(matches!(
            classify_file(path),
            FileCategory::Ddd { ephemeral: false }
        ));
    }

    #[test]
    fn test_classify_toys_directory() {
        let path = Path::new("toys/toy1_example/README.md");
        assert!(matches!(
            classify_file(path),
            FileCategory::Ddd { ephemeral: false }
        ));
    }

    #[test]
    fn test_classify_handoff_ephemeral() {
        let path = Path::new("HANDOFF.md");
        assert!(matches!(
            classify_file(path),
            FileCategory::Ddd { ephemeral: true }
        ));
    }

    #[test]
    fn test_classify_handoff_in_subdirectory() {
        let path = Path::new("some/path/HANDOFF.md");
        assert!(matches!(
            classify_file(path),
            FileCategory::Ddd { ephemeral: true }
        ));
    }

    #[test]
    fn test_classify_regular_file() {
        let path = Path::new("README.md");
        assert!(matches!(classify_file(path), FileCategory::Regular));
    }

    #[test]
    fn test_classify_regular_in_subdirectory() {
        let path = Path::new("guides/SPEC_WRITING.md");
        assert!(matches!(classify_file(path), FileCategory::Regular));
    }

    #[test]
    fn test_count_lines_simple() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        writeln!(file, "line 3").unwrap();

        let lines = count_lines(file.path());
        assert_eq!(lines, Some(3));
    }

    #[test]
    fn test_count_lines_empty() {
        use tempfile::NamedTempFile;

        let file = NamedTempFile::new().unwrap();
        let lines = count_lines(file.path());
        assert_eq!(lines, Some(0));
    }
}
