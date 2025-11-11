mod aggregation;
pub mod cowboy;
pub mod git;
pub mod graph;
mod hooks;
mod states;
mod transcript;

// Re-export public types from submodules
pub use git::GitCommit;
pub use graph::WorkflowDAG;
pub use hooks::{parse_hooks_file, BashCommand, FileModification, HookEvent, HookMetrics};
pub use states::{parse_states_file, StateTransitionEvent};
pub use transcript::{TokenMetrics, TranscriptEvent};

// Re-export aggregation functions
pub use aggregation::{aggregate_tokens_for_range, build_phase_metrics};

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::storage::FileStorage;

/// Debug output data for a single phase
#[derive(Debug, Clone, serde::Serialize)]
pub struct PhaseDebugInfo {
    pub workflow_id: String,
    pub phase_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: u64,
    pub tokens_attributed: u64,
    pub is_archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_events_examined: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_events_matched: Option<usize>,
}

/// Debug configuration for token attribution tracing
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Start of timestamp range (RFC3339)
    pub start: chrono::DateTime<chrono::Utc>,
    /// End of timestamp range (RFC3339)
    pub end: chrono::DateTime<chrono::Utc>,
    /// Show per-event details
    pub verbose: bool,
    /// Output JSON instead of text
    pub json_output: bool,
    /// Collected debug data (when json_output is true)
    pub debug_data: std::sync::Arc<std::sync::Mutex<Vec<PhaseDebugInfo>>>,
}

impl DebugConfig {
    /// Parse debug range from string format "START..END"
    pub fn from_range(range: &str, verbose: bool, json_output: bool) -> Result<Self> {
        let parts: Vec<&str> = range.split("..").collect();
        if parts.len() != 2 {
            anyhow::bail!("Debug range must be in format START..END (RFC3339 timestamps)");
        }

        let start = chrono::DateTime::parse_from_rfc3339(parts[0])?.with_timezone(&chrono::Utc);
        let end = chrono::DateTime::parse_from_rfc3339(parts[1])?.with_timezone(&chrono::Utc);

        if end <= start {
            anyhow::bail!("Debug range end must be after start");
        }

        Ok(Self {
            start,
            end,
            verbose,
            json_output,
            debug_data: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        })
    }

    /// Add phase debug info to collected data
    pub fn add_phase_info(&self, info: PhaseDebugInfo) {
        if self.json_output {
            if let Ok(mut data) = self.debug_data.lock() {
                data.push(info);
            }
        }
    }

    /// Get all collected debug data
    pub fn get_debug_data(&self) -> Vec<PhaseDebugInfo> {
        if let Ok(data) = self.debug_data.lock() {
            data.clone()
        } else {
            Vec::new()
        }
    }

    /// Check if a timestamp overlaps with the debug range
    pub fn overlaps(&self, phase_start: &str, phase_end: Option<&str>) -> bool {
        let phase_start = match chrono::DateTime::parse_from_rfc3339(phase_start) {
            Ok(ts) => ts.with_timezone(&chrono::Utc),
            Err(_) => return false,
        };

        let phase_end = match phase_end {
            Some(end_str) => match chrono::DateTime::parse_from_rfc3339(end_str) {
                Ok(ts) => ts.with_timezone(&chrono::Utc),
                Err(_) => chrono::Utc::now(), // Active phase, use current time
            },
            None => chrono::Utc::now(), // Active phase
        };

        // Check if phase range overlaps with debug range
        // Overlap if: phase_start < debug_end AND phase_end > debug_start
        phase_start < self.end && phase_end > self.start
    }
}

/// Metrics for a single workflow phase
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhaseMetrics {
    pub phase_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: u64,
    pub token_metrics: TokenMetrics,
    pub bash_commands: Vec<BashCommand>,
    pub file_modifications: Vec<FileModification>,
    /// Git commits attributed to this phase
    pub git_commits: Vec<GitCommit>,
    /// Whether this phase is from a synthetic cowboy workflow
    #[serde(default)]
    pub is_synthetic: bool,
    /// Workflow ID this phase belongs to (for proper phase-to-workflow association)
    #[serde(default)]
    pub workflow_id: Option<String>,
}

/// Unified metrics combining all data sources
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnifiedMetrics {
    pub hook_metrics: HookMetrics,
    pub token_metrics: TokenMetrics,
    pub state_transitions: Vec<StateTransitionEvent>,
    pub session_id: Option<String>,
    pub phase_metrics: Vec<PhaseMetrics>,
    /// All git commits in session scope (not phase-specific)
    pub git_commits: Vec<GitCommit>,
}

/// Parse all available metrics from .hegel directory
///
/// By default, archived workflows are NOT loaded to prevent duplication bugs.
/// Only pass `include_archives=true` when you specifically need historical data.
pub fn parse_unified_metrics<P: AsRef<Path>>(
    state_dir: P,
    include_archives: bool,
    debug_config: Option<&DebugConfig>,
) -> Result<UnifiedMetrics> {
    let state_dir = state_dir.as_ref();
    let hooks_path = state_dir.join("hooks.jsonl");
    let states_path = state_dir.join("states.jsonl");

    let mut unified = UnifiedMetrics::default();

    // Read archived workflows only if explicitly requested
    let archives = if include_archives {
        crate::storage::archive::read_archives(state_dir)?
    } else {
        Vec::new()
    };

    // Parse hooks if available
    if hooks_path.exists() {
        unified.hook_metrics = parse_hooks_file(&hooks_path)?;
    }

    // Load current session metadata for session_id tracking
    let storage = FileStorage::new(state_dir)?;
    let state = storage.load()?;

    let mut transcript_files = Vec::new();

    if let Some(session) = state.session_metadata {
        unified.session_id = Some(session.session_id.clone());

        // Try to discover Claude Code transcript directory first (multi-session support)
        let project_root = state_dir.parent().unwrap();
        transcript_files = crate::adapters::list_transcript_files(project_root).unwrap_or_default();

        // Fallback: If no Claude Code transcripts found, use the single session transcript
        // This handles test scenarios and non-Claude Code environments
        if transcript_files.is_empty() && Path::new(&session.transcript_path).exists() {
            transcript_files.push(PathBuf::from(session.transcript_path));
        }
    } else if hooks_path.exists() {
        // Fallback: O(n) scan of hooks.jsonl for backward compatibility
        let content = fs::read_to_string(&hooks_path)?;
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(event) = serde_json::from_str::<HookEvent>(line) {
                if event.hook_event_name == "SessionStart" {
                    unified.session_id = Some(event.session_id);
                    // Also try to get transcript path from hooks
                    if let Some(transcript_path) = event.transcript_path {
                        if Path::new(&transcript_path).exists() {
                            transcript_files.push(PathBuf::from(transcript_path));
                        }
                    }
                    break; // Take the last one found
                }
            }
        }
    }

    // Parse states if available
    if states_path.exists() {
        unified.state_transitions = parse_states_file(&states_path)?;
    }

    // Build phase metrics from state transitions (live workflow)
    // Token metrics will be aggregated from all transcript files per phase
    let live_phase_metrics = build_phase_metrics(
        &unified.state_transitions,
        &unified.hook_metrics,
        &transcript_files,
        debug_config,
    )?;

    // Aggregate token totals from all phases
    if !live_phase_metrics.is_empty() {
        for phase in &live_phase_metrics {
            unified.token_metrics.total_input_tokens += phase.token_metrics.total_input_tokens;
            unified.token_metrics.total_output_tokens += phase.token_metrics.total_output_tokens;
            unified.token_metrics.total_cache_creation_tokens +=
                phase.token_metrics.total_cache_creation_tokens;
            unified.token_metrics.total_cache_read_tokens +=
                phase.token_metrics.total_cache_read_tokens;
            unified.token_metrics.assistant_turns += phase.token_metrics.assistant_turns;
        }
    } else if !transcript_files.is_empty() {
        // Fallback: If no phases but we have transcripts, parse them for overall metrics
        // This handles backward compatibility and test scenarios without state transitions
        let (token_metrics, _, _) =
            aggregate_tokens_for_range(&transcript_files, "1970-01-01T00:00:00Z", None, None)?;
        unified.token_metrics = token_metrics;
    }

    // Merge archived phase metrics with live phase metrics
    let mut all_phase_metrics = Vec::new();

    // Add archived phases first (chronologically older)
    for archive in &archives {
        for phase_archive in &archive.phases {
            // Convert archive phase to PhaseMetrics
            let phase = PhaseMetrics {
                phase_name: phase_archive.phase_name.clone(),
                start_time: phase_archive.start_time.clone(),
                end_time: phase_archive.end_time.clone(),
                duration_seconds: phase_archive.duration_seconds,
                token_metrics: TokenMetrics {
                    total_input_tokens: phase_archive.tokens.input,
                    total_output_tokens: phase_archive.tokens.output,
                    total_cache_creation_tokens: phase_archive.tokens.cache_creation,
                    total_cache_read_tokens: phase_archive.tokens.cache_read,
                    assistant_turns: phase_archive.tokens.assistant_turns,
                },
                bash_commands: vec![], // Archived as summaries, not individual commands
                file_modifications: vec![], // Archived as summaries, not individual modifications
                git_commits: phase_archive.git_commits.clone(),
                is_synthetic: archive.is_synthetic,
                workflow_id: Some(archive.workflow_id.clone()),
            };

            // Debug output for archived phases
            if let Some(cfg) = debug_config {
                if cfg.overlaps(&phase.start_time, phase.end_time.as_deref()) {
                    let total_tokens = phase.token_metrics.total_input_tokens
                        + phase.token_metrics.total_output_tokens;

                    if cfg.json_output {
                        // Collect data for JSON output
                        cfg.add_phase_info(PhaseDebugInfo {
                            workflow_id: archive.workflow_id.clone(),
                            phase_name: phase.phase_name.clone(),
                            start_time: phase.start_time.clone(),
                            end_time: phase.end_time.clone(),
                            duration_seconds: phase.duration_seconds,
                            tokens_attributed: total_tokens,
                            is_archived: true,
                            transcript_events_examined: None,
                            transcript_events_matched: None,
                        });
                    } else {
                        // Print text output
                        eprintln!(
                            "[DEBUG ARCHIVED] Workflow {} | Phase '{}' ({} to {}): {} tokens (from archive)",
                            archive.workflow_id,
                            phase.phase_name,
                            phase.start_time,
                            phase.end_time.as_deref().unwrap_or("active"),
                            total_tokens
                        );
                    }
                }
            }

            all_phase_metrics.push(phase);
        }

        // Add archived git commits to unified total
        for phase_archive in &archive.phases {
            unified
                .git_commits
                .extend(phase_archive.git_commits.clone());
        }

        // Add archived transitions
        for transition_archive in &archive.transitions {
            unified.state_transitions.push(StateTransitionEvent {
                timestamp: transition_archive.timestamp.clone(),
                workflow_id: Some(archive.workflow_id.clone()),
                from_node: transition_archive.from_node.clone(),
                to_node: transition_archive.to_node.clone(),
                phase: transition_archive.to_node.clone(), // Phase is same as to_node
                mode: archive.mode.clone(),
            });
        }

        // Aggregate tokens from archive totals
        unified.token_metrics.total_input_tokens += archive.totals.tokens.input;
        unified.token_metrics.total_output_tokens += archive.totals.tokens.output;
        unified.token_metrics.total_cache_creation_tokens += archive.totals.tokens.cache_creation;
        unified.token_metrics.total_cache_read_tokens += archive.totals.tokens.cache_read;
        unified.token_metrics.assistant_turns += archive.totals.tokens.assistant_turns;
    }

    // Add live phases
    all_phase_metrics.extend(live_phase_metrics);

    // Git commit parsing and attribution ONLY for live data (archives already have git data)
    if !include_archives {
        let git_commits = if git::has_git_repository(state_dir) {
            let project_root = state_dir.parent().unwrap();

            // Use first state transition timestamp as session start (if available)
            let since_timestamp = unified
                .state_transitions
                .first()
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.timestamp).ok())
                .map(|dt| dt.timestamp());

            git::parse_git_commits(project_root, since_timestamp).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse git commits: {}", e);
                Vec::new()
            })
        } else {
            Vec::new()
        };

        // Attribute commits to phases
        git::attribute_commits_to_phases(git_commits.clone(), &mut all_phase_metrics);
        unified.git_commits = git_commits;
    }

    unified.phase_metrics = all_phase_metrics;

    Ok(unified)
}

#[cfg(test)]
mod tests {
    mod git;
    mod unified;
}
