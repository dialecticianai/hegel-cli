/// Synthetic cowboy workflow detection and creation
///
/// Identifies inter-workflow activity gaps and creates synthetic cowboy workflow archives
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};

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

/// Workflow time range for gap detection
#[derive(Debug, Clone)]
struct WorkflowTimeRange {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

/// Identify inter-workflow activity gaps and group them
pub fn identify_cowboy_workflows(
    hooks: &[BashCommand],
    file_mods: &[FileModification],
    commits: &[GitCommit],
    transcripts: &[TranscriptEvent],
    existing_archives: &[WorkflowArchive],
) -> Result<Vec<CowboyActivityGroup>> {
    // Build timeline of existing workflow ranges
    let mut workflow_ranges = Vec::new();
    for archive in existing_archives {
        // Skip synthetic archives (don't create synthetic archives for gaps within synthetic archives)
        if archive.is_synthetic {
            continue;
        }

        let start_time = parse_timestamp(&archive.workflow_id)?;
        let end_time = parse_timestamp(&archive.completed_at)?;

        workflow_ranges.push(WorkflowTimeRange {
            start_time,
            end_time,
        });
    }

    // Sort workflow ranges by start time
    workflow_ranges.sort_by_key(|r| r.start_time);

    // Collect all timestamped activities
    let mut activities: Vec<(DateTime<Utc>, ActivityType)> = Vec::new();

    // Add bash commands
    for cmd in hooks {
        if let Some(ref ts) = cmd.timestamp {
            if let Ok(time) = parse_timestamp(ts) {
                activities.push((time, ActivityType::BashCommand(cmd.clone())));
            }
        }
    }

    // Add file modifications
    for file_mod in file_mods {
        if let Some(ref ts) = file_mod.timestamp {
            if let Ok(time) = parse_timestamp(ts) {
                activities.push((time, ActivityType::FileModification(file_mod.clone())));
            }
        }
    }

    // Add git commits
    for commit in commits {
        if let Ok(time) = parse_timestamp(&commit.timestamp) {
            activities.push((time, ActivityType::GitCommit(commit.clone())));
        }
    }

    // Add transcript events
    for event in transcripts {
        if let Some(ref ts) = event.timestamp {
            if let Ok(time) = parse_timestamp(ts) {
                activities.push((time, ActivityType::Transcript(event.clone())));
            }
        }
    }

    // Sort activities by timestamp
    activities.sort_by_key(|(time, _)| *time);

    // Filter to only inter-workflow activities
    let inter_workflow_activities: Vec<_> = activities
        .into_iter()
        .filter(|(time, _)| {
            // Activity is inter-workflow if it doesn't fall within any workflow range
            !workflow_ranges
                .iter()
                .any(|range| *time >= range.start_time && *time <= range.end_time)
        })
        .collect();

    // Group consecutive activities within 1-hour window
    let groups = group_activities_by_proximity(inter_workflow_activities)?;

    Ok(groups)
}

/// Group activities by temporal proximity (1-hour threshold)
fn group_activities_by_proximity(
    activities: Vec<(DateTime<Utc>, ActivityType)>,
) -> Result<Vec<CowboyActivityGroup>> {
    if activities.is_empty() {
        return Ok(Vec::new());
    }

    let one_hour = Duration::hours(1);
    let mut groups = Vec::new();
    let mut current_group: Option<CowboyActivityGroup> = None;

    for (time, activity) in activities {
        match &mut current_group {
            None => {
                // Start new group
                current_group = Some(CowboyActivityGroup {
                    start_time: time,
                    end_time: time,
                    bash_commands: Vec::new(),
                    file_modifications: Vec::new(),
                    git_commits: Vec::new(),
                    transcript_events: Vec::new(),
                });
            }
            Some(group) => {
                // Check if this activity is within 1 hour of the last activity
                if time - group.end_time > one_hour {
                    // Gap too large, finish current group and start new one
                    groups.push(group.clone());
                    current_group = Some(CowboyActivityGroup {
                        start_time: time,
                        end_time: time,
                        bash_commands: Vec::new(),
                        file_modifications: Vec::new(),
                        git_commits: Vec::new(),
                        transcript_events: Vec::new(),
                    });
                } else {
                    // Update end time
                    group.end_time = time;
                }
            }
        }

        // Add activity to current group
        if let Some(group) = &mut current_group {
            match activity {
                ActivityType::BashCommand(cmd) => group.bash_commands.push(cmd),
                ActivityType::FileModification(file_mod) => group.file_modifications.push(file_mod),
                ActivityType::GitCommit(commit) => group.git_commits.push(commit),
                ActivityType::Transcript(event) => group.transcript_events.push(event),
            }
        }
    }

    // Push final group
    if let Some(group) = current_group {
        groups.push(group);
    }

    Ok(groups)
}

/// Build synthetic cowboy workflow archive from activity group
pub fn build_synthetic_cowboy_archive(group: &CowboyActivityGroup) -> Result<WorkflowArchive> {
    // Use start time as workflow_id
    let workflow_id = group.start_time.to_rfc3339();

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
    let phase_metrics = vec![PhaseMetrics {
        phase_name: "ride".to_string(),
        start_time: group.start_time.to_rfc3339(),
        end_time: Some(group.end_time.to_rfc3339()),
        duration_seconds,
        token_metrics: token_metrics.clone(),
        bash_commands: group.bash_commands.clone(),
        file_modifications: group.file_modifications.clone(),
        git_commits: group.git_commits.clone(),
        is_synthetic: true, // Cowboy workflows are always synthetic
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

/// Activity type for grouping
#[derive(Debug, Clone)]
enum ActivityType {
    BashCommand(BashCommand),
    FileModification(FileModification),
    GitCommit(GitCommit),
    Transcript(TranscriptEvent),
}

/// Parse ISO 8601 timestamp
fn parse_timestamp(ts: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .context("Failed to parse timestamp")
        .map(|dt| dt.with_timezone(&Utc))
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

    fn test_archive(workflow_id: &str, completed_at: &str) -> WorkflowArchive {
        use crate::storage::archive::{
            PhaseArchive, TokenTotals, TransitionArchive, WorkflowTotals,
        };

        WorkflowArchive {
            workflow_id: workflow_id.to_string(),
            mode: "discovery".to_string(),
            completed_at: completed_at.to_string(),
            session_id: None,
            is_synthetic: false,
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: workflow_id.to_string(),
                end_time: Some(completed_at.to_string()),
                duration_seconds: 900,
                tokens: TokenTotals::default(),
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: workflow_id.to_string(),
            }],
            totals: WorkflowTotals::default(),
        }
    }

    #[test]
    fn test_identify_gaps_with_activity_between() {
        let archives = vec![
            test_archive("2025-01-04T09:00:00Z", "2025-01-04T09:30:00Z"),
            test_archive("2025-01-04T11:00:00Z", "2025-01-04T11:30:00Z"),
        ];

        let commits = vec![test_commit("2025-01-04T10:15:00Z")];

        let groups = identify_cowboy_workflows(&[], &[], &commits, &[], &archives).unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].git_commits.len(), 1);
    }

    #[test]
    fn test_no_gaps_when_workflows_cover_timeline() {
        let archives = vec![
            test_archive("2025-01-04T09:00:00Z", "2025-01-04T09:30:00Z"),
            test_archive("2025-01-04T09:30:00Z", "2025-01-04T10:00:00Z"),
        ];

        let commits = vec![test_commit("2025-01-04T09:45:00Z")];

        let groups = identify_cowboy_workflows(&[], &[], &commits, &[], &archives).unwrap();

        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_activity_before_first_workflow() {
        let archives = vec![test_archive("2025-01-04T09:00:00Z", "2025-01-04T09:30:00Z")];

        let commits = vec![
            test_commit("2025-01-04T08:30:00Z"),
            test_commit("2025-01-04T08:45:00Z"),
        ];

        let groups = identify_cowboy_workflows(&[], &[], &commits, &[], &archives).unwrap();

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].git_commits.len(), 2);
    }

    #[test]
    fn test_multiple_gaps_grouped_by_hour() {
        let archives = vec![
            test_archive("2025-01-04T09:00:00Z", "2025-01-04T09:30:00Z"),
            test_archive("2025-01-04T11:00:00Z", "2025-01-04T11:30:00Z"),
            test_archive("2025-01-04T15:00:00Z", "2025-01-04T15:30:00Z"),
        ];

        let commits = vec![
            test_commit("2025-01-04T10:15:00Z"),
            test_commit("2025-01-04T10:20:00Z"), // Within 1 hour, same group
            test_commit("2025-01-04T13:00:00Z"), // >1 hour gap, new group
            test_commit("2025-01-04T13:05:00Z"), // Within 1 hour, same group
        ];

        let groups = identify_cowboy_workflows(&[], &[], &commits, &[], &archives).unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].git_commits.len(), 2);
        assert_eq!(groups[1].git_commits.len(), 2);
    }

    #[test]
    fn test_build_synthetic_cowboy_archive() {
        let group = CowboyActivityGroup {
            start_time: parse_timestamp("2025-01-04T10:00:00Z").unwrap(),
            end_time: parse_timestamp("2025-01-04T10:30:00Z").unwrap(),
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

    #[test]
    fn test_empty_activity_results_in_no_archives() {
        let archives = vec![test_archive("2025-01-04T09:00:00Z", "2025-01-04T09:30:00Z")];

        let groups = identify_cowboy_workflows(&[], &[], &[], &[], &archives).unwrap();

        assert_eq!(groups.len(), 0);
    }
}
