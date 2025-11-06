/// Synthetic cowboy workflow detection and creation
///
/// Identifies inter-workflow activity gaps and creates synthetic cowboy workflow archives
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::metrics::{
    git::GitCommit,
    hooks::{BashCommand, FileModification, HookMetrics},
    transcript::TranscriptEvent,
    PhaseMetrics, StateTransitionEvent, TokenMetrics, UnifiedMetrics,
};
use crate::storage::archive::WorkflowArchive;

/// Activity group representing inter-workflow activity
#[derive(Debug, Clone)]
pub struct CowboyActivityGroup {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub bash_commands: Vec<BashCommand>,
    pub file_modifications: Vec<FileModification>,
    pub git_commits: Vec<GitCommit>,
    pub transcript_events: Vec<TranscriptEvent>,
}

/// Build synthetic cowboy workflow archive from activity group
pub fn build_synthetic_cowboy_archive(group: &CowboyActivityGroup) -> Result<WorkflowArchive> {
    // Build UnifiedMetrics from the activity group
    let hook_metrics = HookMetrics {
        total_events: group.bash_commands.len() + group.file_modifications.len(),
        bash_commands: group.bash_commands.clone(),
        file_modifications: group.file_modifications.clone(),
        session_start_time: Some(group.start_time.to_rfc3339()),
        session_end_time: Some(group.end_time.to_rfc3339()),
    };

    // Aggregate token metrics from transcript events
    let mut total_input = 0;
    let mut total_output = 0;
    let mut total_cache_creation = 0;
    let mut total_cache_read = 0;
    let assistant_turns = group.transcript_events.len();

    for event in &group.transcript_events {
        // Extract token usage from either old or new format
        let usage = event
            .message
            .as_ref()
            .and_then(|m| m.usage.as_ref())
            .or(event.usage.as_ref());

        if let Some(usage) = usage {
            total_input += usage.input_tokens;
            total_output += usage.output_tokens;
            total_cache_creation += usage.cache_creation_input_tokens.unwrap_or(0);
            total_cache_read += usage.cache_read_input_tokens.unwrap_or(0);
        }
    }

    let token_metrics = TokenMetrics {
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cache_creation_tokens: total_cache_creation,
        total_cache_read_tokens: total_cache_read,
        assistant_turns,
    };

    // Create single "ride" phase
    let duration_seconds = (group.end_time - group.start_time).num_seconds() as u64;
    let workflow_id = group.start_time.to_rfc3339();
    let phase_metrics = vec![PhaseMetrics {
        phase_name: "ride".to_string(),
        start_time: group.start_time.to_rfc3339(),
        end_time: Some(group.end_time.to_rfc3339()),
        duration_seconds,
        token_metrics: token_metrics.clone(),
        bash_commands: group.bash_commands.clone(),
        file_modifications: group.file_modifications.clone(),
        git_commits: group.git_commits.clone(),
        is_synthetic: true, // Gap-detected cowboy workflows are synthetic (vs explicit `hegel start cowboy`)
        workflow_id: Some(workflow_id.clone()),
    }];

    // Create minimal state transitions
    let state_transitions = vec![
        StateTransitionEvent {
            timestamp: group.start_time.to_rfc3339(),
            workflow_id: Some(workflow_id.clone()),
            from_node: "START".to_string(),
            to_node: "ride".to_string(),
            phase: "ride".to_string(),
            mode: "cowboy".to_string(),
        },
        StateTransitionEvent {
            timestamp: group.end_time.to_rfc3339(),
            workflow_id: Some(workflow_id.clone()),
            from_node: "ride".to_string(),
            to_node: "done".to_string(),
            phase: "ride".to_string(),
            mode: "cowboy".to_string(),
        },
    ];

    let metrics = UnifiedMetrics {
        hook_metrics,
        token_metrics,
        state_transitions,
        session_id: None,
        phase_metrics,
        git_commits: group.git_commits.clone(),
    };

    // Create archive with is_synthetic=true
    WorkflowArchive::from_metrics(&metrics, &workflow_id, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bash_command(timestamp: &str) -> BashCommand {
        BashCommand {
            command: "echo test".to_string(),
            timestamp: Some(timestamp.to_string()),
            stdout: None,
            stderr: None,
        }
    }

    fn test_file_mod(timestamp: &str) -> FileModification {
        FileModification {
            file_path: "test.rs".to_string(),
            tool: "Edit".to_string(),
            timestamp: Some(timestamp.to_string()),
        }
    }

    fn test_commit(timestamp: &str) -> GitCommit {
        GitCommit {
            hash: "abc123".to_string(),
            author: "test@example.com".to_string(),
            timestamp: timestamp.to_string(),
            message: "test commit".to_string(),
            files_changed: 1,
            insertions: 10,
            deletions: 5,
        }
    }

    fn test_transcript(timestamp: &str) -> TranscriptEvent {
        use crate::metrics::transcript::{MessageWrapper, TokenUsage};
        use std::collections::HashMap;

        TranscriptEvent {
            event_type: "assistant".to_string(),
            timestamp: Some(timestamp.to_string()),
            usage: None,
            message: Some(MessageWrapper {
                usage: Some(TokenUsage {
                    input_tokens: 100,
                    output_tokens: 50,
                    cache_creation_input_tokens: Some(10),
                    cache_read_input_tokens: Some(5),
                }),
            }),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_build_synthetic_cowboy_archive() {
        use chrono::DateTime;
        let parse_ts = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .unwrap()
                .with_timezone(&chrono::Utc)
        };

        let group = CowboyActivityGroup {
            start_time: parse_ts("2025-01-04T10:00:00Z"),
            end_time: parse_ts("2025-01-04T10:30:00Z"),
            bash_commands: vec![test_bash_command("2025-01-04T10:05:00Z")],
            file_modifications: vec![test_file_mod("2025-01-04T10:10:00Z")],
            git_commits: vec![test_commit("2025-01-04T10:15:00Z")],
            transcript_events: vec![test_transcript("2025-01-04T10:20:00Z")],
        };

        let archive = build_synthetic_cowboy_archive(&group).unwrap();

        assert_eq!(archive.mode, "cowboy");
        assert_eq!(archive.is_synthetic, true);
        assert_eq!(archive.phases.len(), 1);
        assert_eq!(archive.phases[0].phase_name, "ride");
        assert_eq!(archive.phases[0].git_commits.len(), 1);
        assert_eq!(archive.phases[0].bash_commands.len(), 1);
        assert_eq!(archive.phases[0].file_modifications.len(), 1);
        assert_eq!(archive.totals.git_commits, 1);
    }
}
