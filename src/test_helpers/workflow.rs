//! Workflow building and setup helpers

use crate::engine::{Node, Transition, Workflow};
use crate::storage::FileStorage;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test workflow YAML - simple 3-node workflow for testing
#[allow(dead_code)] // Reserved for engine tests (see test_helpers/README.md)
pub const TEST_WORKFLOW_YAML: &str = r#"
mode: test_mode
start_node: spec
nodes:
  spec:
    prompt: "Write SPEC.md"
    transitions:
      - when: spec_complete
        to: plan
  plan:
    prompt: "Write PLAN.md"
    transitions:
      - when: plan_complete
        to: done
  done:
    transitions: []
"#;

// ========== Workflow Builder Helpers ==========

/// Create a Transition for workflow tests
///
/// # Example
/// ```ignore
/// let t = transition("spec_complete", "plan");
/// ```
pub fn transition(when: &str, to: &str) -> Transition {
    Transition {
        when: when.to_string(),
        to: to.to_string(),
    }
}

/// Create a Node for workflow tests
///
/// # Example
/// ```ignore
/// let node = node("Write SPEC.md", vec![transition("spec_complete", "plan")]);
/// ```
pub fn node(prompt: &str, transitions: Vec<Transition>) -> Node {
    Node {
        prompt: prompt.to_string(),
        transitions,
        rules: vec![],
    }
}

/// Create a Workflow for testing with a fluent builder pattern
///
/// # Example
/// ```ignore
/// let wf = workflow("discovery", "spec")
///     .with_node("spec", node("Write SPEC", vec![transition("done", "plan")]))
///     .with_node("plan", node("Write PLAN", vec![]))
///     .build();
/// ```
pub struct WorkflowBuilder {
    mode: String,
    start_node: String,
    nodes: HashMap<String, Node>,
}

impl WorkflowBuilder {
    pub fn new(mode: &str, start_node: &str) -> Self {
        Self {
            mode: mode.to_string(),
            start_node: start_node.to_string(),
            nodes: HashMap::new(),
        }
    }

    pub fn with_node(mut self, name: &str, node: Node) -> Self {
        self.nodes.insert(name.to_string(), node);
        self
    }

    pub fn build(self) -> Workflow {
        Workflow {
            mode: self.mode,
            start_node: self.start_node,
            nodes: self.nodes,
        }
    }
}

/// Create a workflow builder for testing
///
/// # Example
/// ```ignore
/// let wf = workflow("discovery", "spec")
///     .with_node("spec", node("Write SPEC", vec![]))
///     .build();
/// ```
pub fn workflow(mode: &str, start_node: &str) -> WorkflowBuilder {
    WorkflowBuilder::new(mode, start_node)
}

// ========== Template Test Helpers ==========

/// Create a context HashMap builder for template tests
///
/// # Example
/// ```ignore
/// let context = ctx().add("name", "World").add("task", "test").build();
/// ```
pub struct CtxBuilder {
    map: HashMap<String, String>,
}

impl CtxBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(mut self, key: &str, value: &str) -> Self {
        self.map.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> HashMap<String, String> {
        self.map
    }
}

/// Create a context builder
pub fn ctx() -> CtxBuilder {
    CtxBuilder::new()
}

/// Create a test guides directory with common guide files
///
/// Returns (TempDir, guides_path)
pub fn test_guides() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let guides_path = temp_dir.path().join("guides");
    std::fs::create_dir(&guides_path).unwrap();

    std::fs::write(
        guides_path.join("SPEC_WRITING.md"),
        "# SPEC Writing Guide\n\nWrite a specification document.",
    )
    .unwrap();

    std::fs::write(
        guides_path.join("PLAN_WRITING.md"),
        "# PLAN Writing Guide\n\nWrite an implementation plan.",
    )
    .unwrap();

    (temp_dir, guides_path)
}

// ========== Claims Helpers ==========

#[allow(dead_code)] // Reserved for transition tests (see test_helpers/README.md)
pub fn claim(key: &str, _value: bool) -> HashSet<String> {
    HashSet::from([key.to_string()])
}

// ========== Workflow Command Test Helpers ==========

/// Setup test environment for workflow commands
///
/// Creates temp directory with workflows/, guides/, and test storage.
/// Uses FileStorage::with_dirs for thread-safe directory isolation.
/// DOES NOT use env vars or change working directory.
///
/// Returns (TempDir, FileStorage) for test isolation
#[allow(dead_code)] // Reserved for command integration tests (see test_helpers/README.md)
pub fn setup_workflow_env() -> (TempDir, FileStorage) {
    let temp_dir = TempDir::new().unwrap();

    // Create workflows and guides directories
    let workflows_dir = temp_dir.path().join("workflows");
    let guides_dir = temp_dir.path().join("guides");
    let state_dir = temp_dir.path().join("state");

    std::fs::create_dir(&workflows_dir).unwrap();
    std::fs::create_dir(&guides_dir).unwrap();

    // Write test workflow
    std::fs::write(workflows_dir.join("test_workflow.yaml"), TEST_WORKFLOW_YAML).unwrap();

    // Create storage with custom directories (thread-safe, no env vars!)
    let storage =
        FileStorage::with_dirs(&state_dir, Some(&workflows_dir), Some(&guides_dir)).unwrap();

    (temp_dir, storage)
}

/// Setup test environment with research + discovery workflows for meta-mode transitions
#[allow(dead_code)] // Reserved for meta-mode tests (see test_helpers/README.md)
pub fn setup_meta_mode_workflows() -> (TempDir, FileStorage) {
    let temp_dir = TempDir::new().unwrap();

    let workflows_dir = temp_dir.path().join("workflows");
    let guides_dir = temp_dir.path().join("guides");
    let state_dir = temp_dir.path().join("state");

    std::fs::create_dir(&workflows_dir).unwrap();
    std::fs::create_dir(&guides_dir).unwrap();

    // Write research workflow (learning mode)
    std::fs::write(
        workflows_dir.join("research.yaml"),
        r#"mode: research
start_node: plan
nodes:
  plan:
    prompt: "Plan research"
    transitions:
      - when: plan_complete
        to: study
  study:
    prompt: "Study sources"
    transitions:
      - when: study_complete
        to: done
  done:
    transitions: []
"#,
    )
    .unwrap();

    // Write discovery workflow (learning mode)
    std::fs::write(
        workflows_dir.join("discovery.yaml"),
        r#"mode: discovery
start_node: spec
nodes:
  spec:
    prompt: "Write SPEC"
    transitions:
      - when: spec_complete
        to: plan
  plan:
    prompt: "Write PLAN"
    transitions:
      - when: plan_complete
        to: done
  done:
    transitions: []
"#,
    )
    .unwrap();

    let storage =
        FileStorage::with_dirs(&state_dir, Some(&workflows_dir), Some(&guides_dir)).unwrap();

    (temp_dir, storage)
}

/// Setup test environment with production workflows
///
/// Copies production workflow files (discovery.yaml, execution.yaml) from project root
/// to temp directory for isolated testing. Uses compile-time path to avoid working directory races.
/// Returns storage with workflows_dir set directly (thread-safe, no env vars).
/// DOES NOT use env vars or change working directory.
///
/// Returns TempDir for cleanup
#[allow(dead_code)] // Reserved for production workflow tests (see test_helpers/README.md)
pub fn setup_production_workflows() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create workflows directory
    let workflows_dir = temp_dir.path().join("workflows");
    std::fs::create_dir(&workflows_dir).unwrap();

    // Use compile-time project root (immune to runtime cwd changes)
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Copy production workflow files
    for workflow in &["discovery.yaml", "execution.yaml"] {
        let src = project_root.join("workflows").join(workflow);
        let dst = workflows_dir.join(workflow);

        if src.exists() {
            std::fs::copy(&src, &dst)
                .unwrap_or_else(|e| panic!("Failed to copy {}: {}", workflow, e));
        } else {
            panic!("Production workflow not found: {}", src.display());
        }
    }

    temp_dir
}
