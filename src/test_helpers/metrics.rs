//! Metrics test helpers and builders

use crate::metrics::{
    BashCommand, FileModification, HookMetrics, PhaseMetrics, StateTransitionEvent, TokenMetrics,
    UnifiedMetrics,
};
use crate::storage::FileStorage;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use super::jsonl::{create_hooks_file, create_states_file};

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
#[allow(dead_code)] // Reserved for metrics integration tests (see test_helpers/README.md)
pub fn test_storage_with_files(
    hooks: Option<&[&str]>,
    states: Option<&[&str]>,
) -> (TempDir, FileStorage) {
    let (temp_dir, state_dir) = setup_state_dir_with_files(hooks, states);
    let storage = FileStorage::new(&state_dir).unwrap();
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

// ========== Metrics Builder ==========

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
#[allow(dead_code)] // Reserved for comprehensive metrics analysis tests (see test_helpers/README.md)
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
                git_commits: vec![],
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
            git_commits: vec![],
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
#[allow(dead_code)] // Reserved for metrics rendering/analysis tests (see test_helpers/README.md)
pub fn test_unified_metrics() -> UnifiedMetrics {
    UnifiedMetricsBuilder::new()
        .with_session("test-session")
        .with_phases(3)
        .with_events(10, 5)
        .build()
}
