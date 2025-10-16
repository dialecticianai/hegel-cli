//! Test utilities and fixtures for Hegel tests
//!
//! This module provides common test helpers to reduce boilerplate and improve
//! test readability across the codebase.

use crate::engine::{Node, Transition, Workflow};
use crate::storage::{FileStorage, WorkflowState};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test workflow YAML - simple 3-node workflow for testing
pub const TEST_WORKFLOW_YAML: &str = r#"
mode: discovery
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
    prompt: "Complete!"
    transitions: []
"#;

/// Create a temporary directory and FileStorage instance for testing
///
/// # Returns
/// A tuple of (TempDir, FileStorage) where the storage uses the temp directory's path
///
/// # Example
/// ```ignore
/// let (temp_dir, storage) = test_storage();
/// // Use storage for testing...
/// // temp_dir is automatically cleaned up when dropped
/// ```
pub fn test_storage() -> (TempDir, FileStorage) {
    let temp_dir = TempDir::new().unwrap();
    let storage = FileStorage::new(temp_dir.path()).unwrap();
    (temp_dir, storage)
}

/// Create a WorkflowState for testing with common defaults
///
/// # Arguments
/// * `node` - Current node name
/// * `mode` - Workflow mode (e.g., "discovery", "execution")
/// * `history` - History of nodes visited (as strings)
///
/// # Example
/// ```ignore
/// let state = test_workflow_state("spec", "discovery", &["spec"]);
/// ```
pub fn test_workflow_state(node: &str, mode: &str, history: &[&str]) -> WorkflowState {
    WorkflowState {
        current_node: node.to_string(),
        mode: mode.to_string(),
        history: history.iter().map(|s| s.to_string()).collect(),
        workflow_id: None,
        meta_mode: None,
    }
}

/// Read a specific line from a JSONL file and parse as JSON
///
/// # Arguments
/// * `path` - Path to the JSONL file
/// * `line_num` - Zero-indexed line number to read
///
/// # Returns
/// Parsed JSON value from the specified line
///
/// # Panics
/// Panics if file doesn't exist, line doesn't exist, or JSON is invalid
///
/// # Example
/// ```ignore
/// let event = read_jsonl_line(&hooks_file, 0);
/// assert_eq!(event["session_id"], "test");
/// ```
pub fn read_jsonl_line(path: &PathBuf, line_num: usize) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    serde_json::from_str(lines[line_num]).unwrap()
}

/// Read all lines from a JSONL file and parse as JSON array
///
/// # Arguments
/// * `path` - Path to the JSONL file
///
/// # Returns
/// Vector of parsed JSON values, one per line
///
/// # Example
/// ```ignore
/// let events = read_jsonl_all(&hooks_file);
/// assert_eq!(events.len(), 3);
/// ```
pub fn read_jsonl_all(path: &PathBuf) -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string(path).unwrap();
    content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// Count the number of lines in a JSONL file
///
/// # Arguments
/// * `path` - Path to the JSONL file
///
/// # Returns
/// Number of lines in the file, or 0 if file doesn't exist
///
/// # Example
/// ```ignore
/// let count = count_jsonl_lines(&states_file);
/// assert_eq!(count, 2);
/// ```
pub fn count_jsonl_lines(path: &PathBuf) -> usize {
    if !path.exists() {
        return 0;
    }
    let content = std::fs::read_to_string(path).unwrap();
    content.lines().count()
}

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

// ========== Storage Test Helpers ==========

use crate::storage::State;

/// Create a State for testing with optional workflow_state
///
/// # Example
/// ```ignore
/// let state = test_state("spec", "discovery", &["spec"]);
/// ```
pub fn test_state(node: &str, mode: &str, history: &[&str]) -> State {
    State {
        workflow: None,
        workflow_state: Some(test_workflow_state(node, mode, history)),
        session_metadata: None,
    }
}

/// Assert that a loaded workflow state matches expected values
pub fn assert_state_eq(state: &State, node: &str, mode: &str, history: &[&str]) {
    let ws = state.workflow_state.as_ref().unwrap();
    assert_eq!(ws.current_node, node);
    assert_eq!(ws.mode, mode);
    assert_eq!(ws.history, history);
}

// ========== JSONL Test Helpers ==========

/// Create a JSONL file for testing with given events
///
/// # Arguments
/// * `events` - Array of JSON strings (one per line)
/// * `filename` - Name of the JSONL file to create
///
/// # Returns
/// A tuple of (TempDir, PathBuf) where PathBuf points to the created file
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"type":"test","value":1}"#];
/// let (_temp_dir, path) = create_jsonl_file(&events, "data.jsonl");
/// ```
pub fn create_jsonl_file(events: &[&str], filename: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join(filename);
    let content = events.join("\n");
    std::fs::write(&file_path, content).unwrap();
    (temp_dir, file_path)
}

/// Create a transcript.jsonl file for testing
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"type":"assistant","usage":{"input_tokens":100}}"#];
/// let (_temp_dir, path) = create_transcript_file(&events);
/// ```
pub fn create_transcript_file(events: &[&str]) -> (TempDir, PathBuf) {
    create_jsonl_file(events, "transcript.jsonl")
}

/// Create a states.jsonl file for testing
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"timestamp":"2025-01-01T00:00:00Z","from_node":"spec","to_node":"plan"}"#];
/// let (_temp_dir, path) = create_states_file(&events);
/// ```
pub fn create_states_file(events: &[&str]) -> (TempDir, PathBuf) {
    create_jsonl_file(events, "states.jsonl")
}

/// Create a hooks.jsonl file for testing
///
/// # Example
/// ```ignore
/// let events = vec![r#"{"session_id":"test","hook_event_name":"SessionStart"}"#];
/// let (_temp_dir, path) = create_hooks_file(&events);
/// ```
pub fn create_hooks_file(events: &[&str]) -> (TempDir, PathBuf) {
    create_jsonl_file(events, "hooks.jsonl")
}

// ========== Metrics Test Helpers ==========

/// Create a state directory with JSONL files already in place
///
/// # Arguments
/// * `hooks` - Optional hooks.jsonl events
/// * `states` - Optional states.jsonl events
///
/// # Returns
/// (TempDir, state_dir_path) with files already copied into state_dir
///
/// # Example
/// ```ignore
/// let (temp_dir, state_dir) = setup_state_dir_with_files(
///     Some(&[r#"{"session_id":"test","hook_event_name":"SessionStart"}"#]),
///     Some(&[r#"{"timestamp":"2025-01-01T10:00:00Z",...}"#]),
/// );
/// ```
pub fn setup_state_dir_with_files(
    hooks: Option<&[&str]>,
    states: Option<&[&str]>,
) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path().to_path_buf();

    if let Some(hook_events) = hooks {
        let (_hooks_temp, hooks_path) = create_hooks_file(hook_events);
        std::fs::copy(&hooks_path, state_dir.join("hooks.jsonl")).unwrap();
    }

    if let Some(state_events) = states {
        let (_states_temp, states_path) = create_states_file(state_events);
        std::fs::copy(&states_path, state_dir.join("states.jsonl")).unwrap();
    }

    (temp_dir, state_dir)
}

/// Create FileStorage with JSONL files already in place
///
/// # Arguments
/// * `hooks` - Optional hooks.jsonl events
/// * `states` - Optional states.jsonl events
///
/// # Returns
/// (TempDir, FileStorage) with files already in the storage directory
///
/// # Example
/// ```ignore
/// let (_temp_dir, storage) = test_storage_with_files(
///     Some(&[r#"{"session_id":"test",...}"#]),
///     None,
/// );
/// let result = analyze_metrics(&storage);
/// ```
pub fn test_storage_with_files(
    hooks: Option<&[&str]>,
    states: Option<&[&str]>,
) -> (TempDir, crate::storage::FileStorage) {
    let (temp_dir, state_dir) = setup_state_dir_with_files(hooks, states);
    let storage = crate::storage::FileStorage::new(&state_dir).unwrap();
    (temp_dir, storage)
}

/// Create a hook event that references a transcript file
///
/// # Arguments
/// * `transcript_path` - Path to transcript file
/// * `session_id` - Session ID for the hook
/// * `timestamp` - ISO 8601 timestamp
///
/// # Returns
/// Formatted hook JSON string
///
/// # Example
/// ```ignore
/// let (_transcript_temp, transcript_path) = create_transcript_file(&events);
/// let hook = hook_with_transcript(&transcript_path, "test", "2025-01-01T10:00:00Z");
/// ```
pub fn hook_with_transcript(transcript_path: &Path, session_id: &str, timestamp: &str) -> String {
    format!(
        r#"{{"session_id":"{}","hook_event_name":"SessionStart","timestamp":"{}","transcript_path":"{}"}}"#,
        session_id,
        timestamp,
        transcript_path.display()
    )
}

// ========== Workflow Command Test Helpers ==========

/// Setup test environment for workflow commands
///
/// Creates temp directory with workflows/, guides/, and test storage.
/// Uses FileStorage::with_dirs for thread-safe directory isolation.
/// DOES NOT use env vars or change working directory.
///
/// Returns (TempDir, FileStorage) for test isolation
pub fn setup_workflow_env() -> (TempDir, FileStorage) {
    let temp_dir = TempDir::new().unwrap();

    // Create workflows and guides directories
    let workflows_dir = temp_dir.path().join("workflows");
    let guides_dir = temp_dir.path().join("guides");
    let state_dir = temp_dir.path().join("state");

    std::fs::create_dir(&workflows_dir).unwrap();
    std::fs::create_dir(&guides_dir).unwrap();

    // Write test workflow
    std::fs::write(workflows_dir.join("discovery.yaml"), TEST_WORKFLOW_YAML).unwrap();

    // Create storage with custom directories (thread-safe, no env vars!)
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

// ========== TUI Test Helpers ==========

#[cfg(test)]
pub mod tui {
    //! TUI testing utilities for ratatui snapshot tests

    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::Terminal;

    /// Standard terminal sizes for consistent testing
    pub const SMALL_TERM: (u16, u16) = (40, 10);
    pub const MEDIUM_TERM: (u16, u16) = (80, 24);
    pub const LARGE_TERM: (u16, u16) = (120, 40);

    /// Create test terminal with specified size
    ///
    /// # Arguments
    /// * `width` - Terminal width in columns
    /// * `height` - Terminal height in rows
    ///
    /// # Example
    /// ```ignore
    /// let mut terminal = test_terminal(80, 24);
    /// terminal.draw(|f| { ... }).unwrap();
    /// ```
    pub fn test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    /// Convert buffer to string for snapshot comparison
    ///
    /// Preserves exact layout including whitespace and newlines.
    /// Useful for golden file testing.
    ///
    /// # Example
    /// ```ignore
    /// let buffer = terminal.backend().buffer();
    /// let output = buffer_to_string(buffer);
    /// assert_eq!(output, expected_snapshot);
    /// ```
    pub fn buffer_to_string(buffer: &Buffer) -> String {
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                output.push_str(cell.symbol());
            }
            output.push('\n');
        }
        output
    }

    /// Render widget to buffer and return string representation
    ///
    /// Convenience wrapper that creates a terminal, renders widget,
    /// and returns string output in one call.
    ///
    /// # Example
    /// ```ignore
    /// let widget = Paragraph::new("Hello");
    /// let output = render_to_string(widget, 80, 24);
    /// assert!(output.contains("Hello"));
    /// ```
    pub fn render_to_string<W>(widget: W, width: u16, height: u16) -> String
    where
        W: ratatui::widgets::Widget,
    {
        let mut terminal = test_terminal(width, height);
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }
}

// ========== Metrics Builder ==========

use crate::metrics::{
    BashCommand, FileModification, HookMetrics, PhaseMetrics, StateTransitionEvent, TokenMetrics,
    UnifiedMetrics,
};

/// Fluent builder for creating test UnifiedMetrics with realistic data
///
/// # Example
/// ```ignore
/// let metrics = UnifiedMetricsBuilder::new()
///     .with_session("test-session")
///     .with_phases(3)
///     .with_events(10, 5)
///     .build();
/// ```
pub struct UnifiedMetricsBuilder {
    session_id: Option<String>,
    hook_metrics: HookMetrics,
    state_transitions: Vec<StateTransitionEvent>,
    phase_metrics: Vec<PhaseMetrics>,
}

impl Default for UnifiedMetricsBuilder {
    fn default() -> Self {
        Self {
            session_id: None,
            hook_metrics: HookMetrics::default(),
            state_transitions: Vec::new(),
            phase_metrics: Vec::new(),
        }
    }
}

impl UnifiedMetricsBuilder {
    /// Create a new metrics builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set session ID
    pub fn with_session(mut self, id: &str) -> Self {
        self.session_id = Some(id.to_string());
        self
    }

    /// Add realistic phase metrics
    ///
    /// Generates phases with increasing timestamps, token usage, and activity.
    /// Phases cycle through: spec, plan, code
    ///
    /// # Arguments
    /// * `count` - Number of phases to generate
    pub fn with_phases(mut self, count: usize) -> Self {
        let phases = vec!["spec", "plan", "code"];

        for i in 0..count {
            let phase_name = phases[i % phases.len()].to_string();
            let start_time = format!("2025-01-01T10:{:02}:00Z", i * 15);
            let end_time = if i == count - 1 {
                None // Last phase is active
            } else {
                Some(format!("2025-01-01T10:{:02}:00Z", (i + 1) * 15))
            };

            // Add phase metrics
            self.phase_metrics.push(PhaseMetrics {
                phase_name: phase_name.clone(),
                start_time: start_time.clone(),
                end_time: end_time.clone(),
                duration_seconds: if end_time.is_some() { 900 } else { 0 },
                token_metrics: TokenMetrics {
                    total_input_tokens: 1000 + (i as u64 * 500),
                    total_output_tokens: 500 + (i as u64 * 250),
                    total_cache_creation_tokens: 200,
                    total_cache_read_tokens: 300,
                    assistant_turns: 5,
                },
                bash_commands: vec![],
                file_modifications: vec![],
            });

            // Add corresponding state transition
            self.state_transitions.push(StateTransitionEvent {
                timestamp: start_time,
                workflow_id: Some("test-workflow".to_string()),
                from_node: if i == 0 {
                    "START".to_string()
                } else {
                    phases[(i - 1) % phases.len()].to_string()
                },
                to_node: phase_name.clone(),
                phase: phase_name,
                mode: "discovery".to_string(),
            });
        }

        self
    }

    /// Add bash commands and file modifications
    ///
    /// # Arguments
    /// * `bash_count` - Number of bash commands to generate
    /// * `file_count` - Number of file modifications to generate
    pub fn with_events(mut self, bash_count: usize, file_count: usize) -> Self {
        // Add bash commands
        for i in 0..bash_count {
            self.hook_metrics.bash_commands.push(BashCommand {
                command: format!("cargo build #{}", i),
                timestamp: Some(format!("2025-01-01T10:05:{:02}Z", i)),
                stdout: None,
                stderr: None,
            });
        }

        // Add file modifications
        for i in 0..file_count {
            self.hook_metrics.file_modifications.push(FileModification {
                file_path: format!("src/file{}.rs", i),
                tool: "Edit".to_string(),
                timestamp: Some(format!("2025-01-01T10:10:{:02}Z", i)),
            });
        }

        self.hook_metrics.total_events = bash_count + file_count;
        self
    }

    /// Build the UnifiedMetrics
    pub fn build(self) -> UnifiedMetrics {
        // Aggregate token metrics from phases
        let token_metrics = self
            .phase_metrics
            .iter()
            .fold(TokenMetrics::default(), |acc, p| TokenMetrics {
                total_input_tokens: acc.total_input_tokens + p.token_metrics.total_input_tokens,
                total_output_tokens: acc.total_output_tokens + p.token_metrics.total_output_tokens,
                total_cache_creation_tokens: acc.total_cache_creation_tokens
                    + p.token_metrics.total_cache_creation_tokens,
                total_cache_read_tokens: acc.total_cache_read_tokens
                    + p.token_metrics.total_cache_read_tokens,
                assistant_turns: acc.assistant_turns + p.token_metrics.assistant_turns,
            });

        UnifiedMetrics {
            session_id: self.session_id,
            hook_metrics: self.hook_metrics,
            token_metrics,
            state_transitions: self.state_transitions,
            phase_metrics: self.phase_metrics,
        }
    }
}

/// Create standard test metrics with sensible defaults
///
/// Equivalent to:
/// ```ignore
/// UnifiedMetricsBuilder::new()
///     .with_session("test-session")
///     .with_phases(3)
///     .with_events(10, 5)
///     .build()
/// ```
///
/// # Example
/// ```ignore
/// let metrics = test_unified_metrics();
/// let app = AppState::new(metrics);
/// ```
pub fn test_unified_metrics() -> UnifiedMetrics {
    UnifiedMetricsBuilder::new()
        .with_session("test-session")
        .with_phases(3)
        .with_events(10, 5)
        .build()
}

// ========== Adapter Test Fixtures ==========

/// Load JSON fixture from tests/fixtures/ directory
///
/// # Arguments
/// * `path` - Relative path from tests/fixtures/ (e.g., "adapters/codex_token_count.json")
///
/// # Returns
/// Parsed JSON value
///
/// # Panics
/// Panics if file doesn't exist or JSON is invalid
///
/// # Example
/// ```ignore
/// let event = load_fixture("adapters/codex_token_count.json");
/// let adapter = CodexAdapter::new();
/// let result = adapter.normalize(event).unwrap();
/// ```
pub fn load_fixture(path: &str) -> serde_json::Value {
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = project_root.join("tests/fixtures").join(path);

    let content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", fixture_path.display(), e));

    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", fixture_path.display(), e))
}
