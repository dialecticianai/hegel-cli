use anyhow::Result;
use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::theme::Theme;

/// Arguments for the markdown command
#[derive(Debug, Clone)]
pub struct MarkdownArgs {
    pub json: bool,
    pub no_ddd: bool,
}

/// File category classification
#[derive(Debug, Clone, PartialEq)]
enum FileCategory {
    Ddd { ephemeral: bool },
    Regular,
}

/// Markdown file with metadata
#[derive(Debug, Clone)]
struct MarkdownFile {
    path: PathBuf,
    category: FileCategory,
    lines: Option<usize>,
    size_bytes: u64,
    last_modified: DateTime<Utc>,
}

/// JSON output schema
#[derive(Debug, Serialize, Deserialize)]
struct MarkdownTree {
    #[serde(skip_serializing_if = "Option::is_none")]
    ddd_documents: Option<Vec<FileEntry>>,
    other_markdown: Vec<FileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    path: String,
    lines: usize,
    size_bytes: u64,
    last_modified: String,
    ephemeral: bool,
}

/// Execute the markdown tree command
pub fn run_markdown(args: MarkdownArgs) -> Result<()> {
    // Scan current directory for markdown files
    let files = scan_markdown_files()?;

    if files.is_empty() {
        println!("No markdown files found in current directory");
        return Ok(());
    }

    // Categorize files
    let (ddd_files, regular_files) = categorize_files(files, args.no_ddd);

    // Output based on mode
    if args.json {
        output_json(&ddd_files, &regular_files, args.no_ddd)?;
    } else {
        output_tree(&ddd_files, &regular_files, args.no_ddd)?;
    }

    Ok(())
}

/// Scan directory for markdown files with gitignore support
fn scan_markdown_files() -> Result<Vec<MarkdownFile>> {
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

/// Check if a file is a markdown file
fn is_markdown_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("md")
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

/// Categorize files into DDD and regular
fn categorize_files(
    files: Vec<MarkdownFile>,
    no_ddd: bool,
) -> (Vec<MarkdownFile>, Vec<MarkdownFile>) {
    if no_ddd {
        let regular: Vec<_> = files
            .into_iter()
            .filter(|f| matches!(f.category, FileCategory::Regular))
            .collect();
        (vec![], regular)
    } else {
        let mut ddd = Vec::new();
        let mut regular = Vec::new();

        for file in files {
            match file.category {
                FileCategory::Ddd { .. } => ddd.push(file),
                FileCategory::Regular => regular.push(file),
            }
        }

        (ddd, regular)
    }
}

/// Output as JSON
fn output_json(
    ddd_files: &[MarkdownFile],
    regular_files: &[MarkdownFile],
    no_ddd: bool,
) -> Result<()> {
    let ddd_documents = if no_ddd {
        None
    } else {
        Some(
            ddd_files
                .iter()
                .map(|f| FileEntry {
                    path: f.path.display().to_string(),
                    lines: f.lines.unwrap_or(0),
                    size_bytes: f.size_bytes,
                    last_modified: f.last_modified.to_rfc3339(),
                    ephemeral: matches!(f.category, FileCategory::Ddd { ephemeral: true }),
                })
                .collect(),
        )
    };

    let other_markdown = regular_files
        .iter()
        .map(|f| FileEntry {
            path: f.path.display().to_string(),
            lines: f.lines.unwrap_or(0),
            size_bytes: f.size_bytes,
            last_modified: f.last_modified.to_rfc3339(),
            ephemeral: false,
        })
        .collect();

    let tree = MarkdownTree {
        ddd_documents,
        other_markdown,
    };

    let json = serde_json::to_string_pretty(&tree)?;
    println!("{}", json);

    Ok(())
}

/// Output as tree
fn output_tree(
    ddd_files: &[MarkdownFile],
    regular_files: &[MarkdownFile],
    no_ddd: bool,
) -> Result<()> {
    if !no_ddd && !ddd_files.is_empty() {
        println!("{}", Theme::header("DDD Documents:"));
        print_tree(ddd_files)?;
        println!();
    }

    if !regular_files.is_empty() {
        println!("{}", Theme::header("Other Markdown:"));
        print_tree(regular_files)?;
    }

    Ok(())
}

/// Print tree structure for a list of files
fn print_tree(files: &[MarkdownFile]) -> Result<()> {
    // Build tree structure
    let tree = build_tree_structure(files);

    // Render tree
    render_tree_node(&tree, "", true);

    Ok(())
}

/// Tree node structure
#[derive(Debug)]
struct TreeNode {
    name: String,
    is_file: bool,
    lines: Option<usize>,
    ephemeral: bool,
    children: Vec<TreeNode>,
}

/// Build tree structure from flat file list
fn build_tree_structure(files: &[MarkdownFile]) -> TreeNode {
    let mut root = TreeNode {
        name: String::new(),
        is_file: false,
        lines: None,
        ephemeral: false,
        children: Vec::new(),
    };

    for file in files {
        let mut current = &mut root;
        let components: Vec<_> = file.path.components().collect();

        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_string_lossy().to_string();
            let is_last = i == components.len() - 1;

            // Find or create child
            let child_idx = current
                .children
                .iter()
                .position(|c| c.name == name)
                .unwrap_or_else(|| {
                    current.children.push(TreeNode {
                        name: name.clone(),
                        is_file: is_last,
                        lines: if is_last { file.lines } else { None },
                        ephemeral: if is_last {
                            matches!(file.category, FileCategory::Ddd { ephemeral: true })
                        } else {
                            false
                        },
                        children: Vec::new(),
                    });
                    current.children.len() - 1
                });

            current = &mut current.children[child_idx];
        }
    }

    // Sort children alphabetically
    sort_tree(&mut root);

    root
}

/// Sort tree nodes alphabetically
fn sort_tree(node: &mut TreeNode) {
    node.children.sort_by(|a, b| a.name.cmp(&b.name));
    for child in &mut node.children {
        sort_tree(child);
    }
}

/// Render tree node with proper characters
fn render_tree_node(node: &TreeNode, prefix: &str, is_root: bool) {
    if is_root {
        for (i, child) in node.children.iter().enumerate() {
            let is_last = i == node.children.len() - 1;
            render_tree_child(child, "", is_last);
        }
    } else {
        for (i, child) in node.children.iter().enumerate() {
            let is_last = i == node.children.len() - 1;
            render_tree_child(child, prefix, is_last);
        }
    }
}

/// Render a tree child
fn render_tree_child(node: &TreeNode, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = if is_last { "    " } else { "│   " };

    if node.is_file {
        let lines_str = if let Some(lines) = node.lines {
            format!(" {}", Theme::metric_value(format!("({} lines)", lines)))
        } else {
            format!(" {}", Theme::metric_value("(? lines)"))
        };

        let ephemeral_str = if node.ephemeral {
            format!(" {}", Theme::warning("[ephemeral]"))
        } else {
            String::new()
        };

        println!(
            "{}{}{}{}{}",
            Theme::secondary(prefix),
            Theme::secondary(connector),
            node.name,
            lines_str,
            ephemeral_str
        );
    } else {
        println!(
            "{}{}{}",
            Theme::secondary(prefix),
            Theme::secondary(connector),
            Theme::secondary(&format!("{}/", node.name))
        );

        let new_prefix = format!("{}{}", prefix, child_prefix);
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            render_tree_child(child, &new_prefix, is_last_child);
        }
    }
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
