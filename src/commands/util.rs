//! Small shared helpers for command implementations.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Resolve a file path, trying it as-is first, then with a `.md` extension.
pub(crate) fn resolve_file_path(file_path: &Path) -> Result<PathBuf> {
    if file_path.exists() {
        return Ok(file_path.to_path_buf());
    }

    let with_md = file_path.with_extension("md");
    if with_md.exists() {
        return Ok(with_md);
    }

    anyhow::bail!(
        "File not found: {} (also tried with .md extension)",
        file_path.display()
    )
}

/// Whether `path` has a `.md` extension.
pub(crate) fn is_markdown_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("md")
}
