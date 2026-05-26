//! Tree model construction and DDD-metadata attachment.

use std::path::{Path, PathBuf};

use crate::ddd::{parse_ddd_structure, DddArtifact};

use super::scan::{FileCategory, MarkdownFile};

/// Metadata for an artifact file within a directory
#[derive(Debug, Clone)]
pub(crate) struct ArtifactFileMeta {
    pub name: String,
    pub exists: bool,
    pub required: bool,
}

/// Tree node structure
#[derive(Debug)]
pub(crate) struct TreeNode {
    pub(crate) name: String,
    pub(crate) is_file: bool,
    pub(crate) lines: Option<usize>,
    pub(crate) ephemeral: bool,
    /// For artifact directories: metadata about constituent files (e.g., SPEC.md, PLAN.md)
    pub(crate) artifact_files: Vec<ArtifactFileMeta>,
    /// Whether this artifact has validation issues
    pub(crate) is_malformed: bool,
    pub(crate) children: Vec<TreeNode>,
}

/// Build tree structure from flat file list
pub(crate) fn build_tree_structure(files: &[MarkdownFile]) -> TreeNode {
    let mut root = TreeNode {
        name: String::new(),
        is_file: false,
        lines: None,
        ephemeral: false,
        artifact_files: Vec::new(),
        is_malformed: false,
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
                        artifact_files: Vec::new(),
                        is_malformed: false,
                        children: Vec::new(),
                    });
                    current.children.len() - 1
                });

            current = &mut current.children[child_idx];
        }
    }

    // Sort children alphabetically
    sort_tree(&mut root);

    // Attach DDD metadata (only if .ddd/ exists)
    if PathBuf::from(".ddd").exists() {
        if let Ok(scan_result) = parse_ddd_structure() {
            attach_ddd_metadata(&mut root, &scan_result);
        }
    }

    root
}

/// Attach DDD metadata to tree nodes
fn attach_ddd_metadata(node: &mut TreeNode, scan_result: &crate::ddd::DddScanResult) {
    // Create a map of malformed paths for quick lookup
    let malformed_paths: std::collections::HashSet<PathBuf> = scan_result
        .issues
        .iter()
        .map(|issue| issue.path.clone())
        .collect();

    // Recursively traverse and attach metadata
    attach_ddd_metadata_recursive(
        node,
        Path::new(""),
        &scan_result.artifacts,
        &malformed_paths,
    );
}

/// Recursively attach DDD metadata to tree nodes
fn attach_ddd_metadata_recursive(
    node: &mut TreeNode,
    current_path: &Path,
    artifacts: &[DddArtifact],
    malformed_paths: &std::collections::HashSet<PathBuf>,
) {
    let node_path = if current_path.as_os_str().is_empty() {
        PathBuf::from(&node.name)
    } else {
        current_path.join(&node.name)
    };

    // Check if this path is malformed
    for malformed_path in malformed_paths {
        // Compare just the file/dir name for malformed checks
        if node_path.ends_with(malformed_path) || &node_path == malformed_path {
            node.is_malformed = true;
            break;
        }
    }

    // Check if this is a feat/toy directory and attach artifact file metadata
    if !node.is_file {
        for artifact in artifacts {
            match artifact {
                DddArtifact::Feat(feat) => {
                    if node.name == feat.dir_name() {
                        use crate::ddd::FeatArtifact;
                        node.artifact_files = FeatArtifact::FILES
                            .iter()
                            .map(|spec| {
                                let exists = match spec.name {
                                    "SPEC.md" => feat.spec_exists,
                                    "PLAN.md" => feat.plan_exists,
                                    _ => false,
                                };
                                ArtifactFileMeta {
                                    name: spec.name.to_string(),
                                    exists,
                                    required: spec.required,
                                }
                            })
                            .collect();
                        break;
                    }
                }
                DddArtifact::Toy(toy) => {
                    if node.name == toy.dir_name() {
                        use crate::ddd::ToyArtifact;
                        node.artifact_files = ToyArtifact::FILES
                            .iter()
                            .map(|spec| {
                                let exists = match spec.name {
                                    "SPEC.md" => toy.spec_exists,
                                    "PLAN.md" => toy.plan_exists,
                                    "LEARNINGS.md" => toy.learnings_exists,
                                    "README.md" => toy.readme_exists,
                                    _ => false,
                                };
                                ArtifactFileMeta {
                                    name: spec.name.to_string(),
                                    exists,
                                    required: spec.required,
                                }
                            })
                            .collect();
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    // Recurse into children
    for child in &mut node.children {
        attach_ddd_metadata_recursive(child, &node_path, artifacts, malformed_paths);
    }
}

/// Sort tree nodes alphabetically
fn sort_tree(node: &mut TreeNode) {
    node.children.sort_by(|a, b| a.name.cmp(&b.name));
    for child in &mut node.children {
        sort_tree(child);
    }
}

/// Check if tree contains any validation issues
pub(crate) fn check_for_issues(node: &TreeNode) -> bool {
    if node.is_malformed {
        return true;
    }
    for child in &node.children {
        if check_for_issues(child) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_ddd_metadata_to_feat_directory() {
        use crate::ddd::{DddArtifact, DddScanResult, FeatArtifact};

        // Create a tree node for a feat directory
        let mut node = TreeNode {
            name: "20251104-1-test-feature".to_string(),
            is_file: false,
            lines: None,
            ephemeral: false,
            artifact_files: Vec::new(),
            is_malformed: false,
            children: Vec::new(),
        };

        // Create scan result with a matching artifact
        let scan_result = DddScanResult {
            artifacts: vec![DddArtifact::Feat(FeatArtifact {
                date: "20251104".to_string(),
                index: Some(1),
                name: "test-feature".to_string(),
                spec_exists: true,
                plan_exists: false,
            })],
            issues: Vec::new(),
        };

        // Attach metadata
        attach_ddd_metadata(&mut node, &scan_result);

        // Verify metadata was attached
        assert_eq!(node.artifact_files.len(), 2);
        assert_eq!(node.artifact_files[0].name, "SPEC.md");
        assert!(node.artifact_files[0].exists);
        assert!(node.artifact_files[0].required);
        assert_eq!(node.artifact_files[1].name, "PLAN.md");
        assert!(!node.artifact_files[1].exists);
        assert!(node.artifact_files[1].required);
        assert!(!node.is_malformed);
    }
}
