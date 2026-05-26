//! Terminal tree rendering.

use anyhow::Result;

use crate::theme::Theme;

use super::scan::MarkdownFile;
use super::tree::{build_tree_structure, check_for_issues, TreeNode};

/// Output as tree
pub(crate) fn output_tree(
    ddd_files: &[MarkdownFile],
    regular_files: &[MarkdownFile],
    no_ddd: bool,
    ddd: bool,
) -> Result<()> {
    let mut has_issues = false;

    if !no_ddd && !ddd_files.is_empty() {
        println!("{}", Theme::header("DDD Documents:"));
        has_issues = print_tree(ddd_files)?;
        if !ddd && !regular_files.is_empty() {
            println!();
        }
    }

    if !ddd && !regular_files.is_empty() {
        println!("{}", Theme::header("Other Markdown:"));
        print_tree(regular_files)?;
    }

    // Show footer warning if issues found
    if has_issues {
        println!();
        println!(
            "{} Run hegel doctor to fix malformed artifacts",
            Theme::warning("⚠️")
        );
    }

    Ok(())
}

/// Print tree structure for a list of files
/// Returns true if any validation issues were found
fn print_tree(files: &[MarkdownFile]) -> Result<bool> {
    // Build tree structure
    let tree = build_tree_structure(files);

    // Render tree and check for issues
    let has_issues = check_for_issues(&tree);
    render_tree_node(&tree, "", true);

    Ok(has_issues)
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

        let warning_str = if node.is_malformed {
            format!(" {}", Theme::warning("⚠️"))
        } else {
            String::new()
        };

        println!(
            "{}{}{}{}{}{}",
            Theme::secondary(prefix),
            Theme::secondary(connector),
            node.name,
            lines_str,
            ephemeral_str,
            warning_str
        );
    } else {
        // For directories, check if it has artifact file metadata
        let artifact_metadata = if !node.artifact_files.is_empty() {
            let indicators: Vec<String> = node
                .artifact_files
                .iter()
                .filter_map(|file_meta| {
                    // Skip optional files that don't exist
                    if !file_meta.required && !file_meta.exists {
                        return None;
                    }

                    // Look up line count from children
                    let lines = node
                        .children
                        .iter()
                        .find(|child| child.name == file_meta.name)
                        .and_then(|child| child.lines);

                    let file_name_without_ext = file_meta
                        .name
                        .strip_suffix(".md")
                        .unwrap_or(&file_meta.name);

                    // Build colored output
                    let check_mark_colored = if file_meta.exists {
                        Theme::success("✓")
                    } else {
                        Theme::error("✗")
                    };

                    let result = if let Some(line_count) = lines {
                        format!(
                            "{} {} {}",
                            file_name_without_ext,
                            Theme::metric_value(&format!("({} lines)", line_count)),
                            check_mark_colored
                        )
                    } else if file_meta.exists {
                        format!(
                            "{} {} {}",
                            file_name_without_ext,
                            Theme::metric_value("(? lines)"),
                            check_mark_colored
                        )
                    } else {
                        format!("{} {}", file_name_without_ext, check_mark_colored)
                    };

                    Some(result)
                })
                .collect();

            if !indicators.is_empty() {
                format!("  {}", indicators.join(" "))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let warning_str = if node.is_malformed {
            format!(" {}", Theme::warning("⚠️"))
        } else {
            String::new()
        };

        println!(
            "{}{}{}{}{}",
            Theme::secondary(prefix),
            Theme::secondary(connector),
            Theme::secondary(&format!("{}/", node.name)),
            artifact_metadata,
            warning_str
        );

        // Filter out artifact files from children when rendering
        let artifact_file_names: std::collections::HashSet<_> = node
            .artifact_files
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        let new_prefix = format!("{}{}", prefix, child_prefix);
        for (i, child) in node.children.iter().enumerate() {
            // Skip artifact files (they're shown inline with the directory)
            if artifact_file_names.contains(child.name.as_str()) {
                continue;
            }

            let is_last_child = i == node.children.len() - 1;
            render_tree_child(child, &new_prefix, is_last_child);
        }
    }
}
