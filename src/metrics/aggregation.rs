use crate::metrics::{
    BashCommand, FileModification, HookMetrics, PhaseMetrics, StateTransitionEvent, TokenMetrics,
};
use anyhow::Result;
use std::fs;

/// Build per-phase metrics from state transitions, hooks, and transcript
pub fn build_phase_metrics(
    transitions: &[StateTransitionEvent],
    hook_metrics: &HookMetrics,
    transcript_path: Option<&str>,
) -> Result<Vec<PhaseMetrics>> {
    use chrono::DateTime;

    if transitions.is_empty() {
        return Ok(Vec::new());
    }

    let mut phases = Vec::new();

    // Build phase boundaries: each transition marks a phase
    for (i, transition) in transitions.iter().enumerate() {
        let phase_name = transition.phase.clone();
        let start_time = transition.timestamp.clone();
        let end_time = transitions.get(i + 1).map(|t| t.timestamp.clone());

        // Calculate duration
        let duration_seconds = if let Some(ref end) = end_time {
            let start = DateTime::parse_from_rfc3339(&start_time)?;
            let end = DateTime::parse_from_rfc3339(end)?;
            (end - start).num_seconds() as u64
        } else {
            0 // Active phase has no duration yet
        };

        // Bucket hooks by timestamp
        let bash_commands: Vec<BashCommand> = hook_metrics
            .bash_commands
            .iter()
            .filter(|cmd| {
                cmd.timestamp.as_ref().map_or(false, |ts| {
                    is_in_range(ts, &start_time, end_time.as_deref())
                })
            })
            .cloned()
            .collect();

        let file_modifications: Vec<FileModification> = hook_metrics
            .file_modifications
            .iter()
            .filter(|file_mod| {
                file_mod.timestamp.as_ref().map_or(false, |ts| {
                    is_in_range(ts, &start_time, end_time.as_deref())
                })
            })
            .cloned()
            .collect();

        // Aggregate tokens from transcript for this phase
        let token_metrics = if let Some(transcript_path) = transcript_path {
            aggregate_tokens_for_phase(transcript_path, &start_time, end_time.as_deref())?
        } else {
            TokenMetrics::default()
        };

        phases.push(PhaseMetrics {
            phase_name,
            start_time,
            end_time,
            duration_seconds,
            token_metrics,
            bash_commands,
            file_modifications,
        });
    }

    Ok(phases)
}

/// Check if timestamp is in range [start, end)
fn is_in_range(timestamp: &str, start: &str, end: Option<&str>) -> bool {
    if let (Ok(ts), Ok(start_ts)) = (
        chrono::DateTime::parse_from_rfc3339(timestamp),
        chrono::DateTime::parse_from_rfc3339(start),
    ) {
        let after_start = ts >= start_ts;
        let before_end = end.map_or(true, |e| {
            chrono::DateTime::parse_from_rfc3339(e).map_or(false, |end_ts| ts < end_ts)
        });
        after_start && before_end
    } else {
        false
    }
}

/// Aggregate token usage from transcript for a specific phase
fn aggregate_tokens_for_phase(
    transcript_path: &str,
    start_time: &str,
    end_time: Option<&str>,
) -> Result<TokenMetrics> {
    use crate::metrics::transcript::TranscriptEvent;

    let content = fs::read_to_string(transcript_path)?;
    let mut metrics = TokenMetrics::default();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let event: TranscriptEvent = serde_json::from_str(line)?;

        // Only process assistant events
        if event.event_type != "assistant" {
            continue;
        }

        // Check if timestamp is in phase range
        if let Some(ref timestamp) = event.timestamp {
            if !is_in_range(timestamp, start_time, end_time) {
                continue;
            }
        } else {
            continue; // Skip events without timestamps
        }

        // Extract token usage
        let usage = event
            .usage
            .or_else(|| event.message.as_ref().and_then(|m| m.usage.clone()));

        if let Some(usage) = usage {
            metrics.total_input_tokens += usage.input_tokens;
            metrics.total_output_tokens += usage.output_tokens;
            metrics.total_cache_creation_tokens += usage.cache_creation_input_tokens.unwrap_or(0);
            metrics.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
            metrics.assistant_turns += 1;
        }
    }

    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_phase_metrics_empty_transitions() {
        let metrics = build_phase_metrics(&[], &HookMetrics::default(), None).unwrap();
        assert!(metrics.is_empty());
    }

    #[test]
    fn test_phase_metrics_single_active_phase() {
        let transitions = vec![StateTransitionEvent {
            timestamp: "2025-01-01T10:00:00Z".to_string(),
            workflow_id: Some("test".to_string()),
            from_node: "START".to_string(),
            to_node: "spec".to_string(),
            phase: "spec".to_string(),
            mode: "discovery".to_string(),
        }];

        let metrics = build_phase_metrics(&transitions, &HookMetrics::default(), None).unwrap();

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].phase_name, "spec");
        assert_eq!(metrics[0].end_time, None);
    }

    #[test]
    fn test_is_in_range() {
        // Within range
        assert!(is_in_range(
            "2025-01-01T10:05:00Z",
            "2025-01-01T10:00:00Z",
            Some("2025-01-01T10:15:00Z")
        ));

        // Before range
        assert!(!is_in_range(
            "2025-01-01T09:55:00Z",
            "2025-01-01T10:00:00Z",
            Some("2025-01-01T10:15:00Z")
        ));

        // After range
        assert!(!is_in_range(
            "2025-01-01T10:20:00Z",
            "2025-01-01T10:00:00Z",
            Some("2025-01-01T10:15:00Z")
        ));

        // No end (active phase) - should accept anything after start
        assert!(is_in_range(
            "2025-01-01T10:20:00Z",
            "2025-01-01T10:00:00Z",
            None
        ));
    }
}
