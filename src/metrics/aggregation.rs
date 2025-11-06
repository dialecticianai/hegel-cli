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
    debug_config: Option<&crate::metrics::DebugConfig>,
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
        let (token_metrics, examined_count, matched_count) =
            if let Some(transcript_path) = transcript_path {
                aggregate_tokens_for_phase(
                    transcript_path,
                    &start_time,
                    end_time.as_deref(),
                    &phase_name,
                    debug_config,
                )?
            } else {
                (TokenMetrics::default(), 0, 0)
            };

        // Handle debug output/collection for live phases
        if let Some(cfg) = debug_config {
            if cfg.overlaps(&start_time, end_time.as_deref()) {
                let total_tokens =
                    token_metrics.total_input_tokens + token_metrics.total_output_tokens;

                if cfg.json_output {
                    // Collect data for JSON output
                    cfg.add_phase_info(crate::metrics::PhaseDebugInfo {
                        workflow_id: "live".to_string(), // Live phases don't have workflow_id yet
                        phase_name: phase_name.clone(),
                        start_time: start_time.clone(),
                        end_time: end_time.clone(),
                        duration_seconds,
                        tokens_attributed: total_tokens,
                        is_archived: false,
                        transcript_events_examined: Some(examined_count),
                        transcript_events_matched: Some(matched_count),
                    });
                }
                // Text output already printed in aggregate_tokens_for_phase
            }
        }

        phases.push(PhaseMetrics {
            phase_name,
            start_time,
            end_time,
            duration_seconds,
            token_metrics,
            bash_commands,
            file_modifications,
            git_commits: vec![],
            is_synthetic: false, // Live phases are never synthetic
            workflow_id: None,   // Live phases don't have workflow_id yet (set later if archived)
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
/// Returns (TokenMetrics, examined_count, matched_count)
fn aggregate_tokens_for_phase(
    transcript_path: &str,
    start_time: &str,
    end_time: Option<&str>,
    phase_name: &str,
    debug_config: Option<&crate::metrics::DebugConfig>,
) -> Result<(TokenMetrics, usize, usize)> {
    use crate::metrics::transcript::TranscriptEvent;

    let content = fs::read_to_string(transcript_path)?;
    let mut metrics = TokenMetrics::default();

    // Debug: Check if we should output for this phase
    let should_debug = debug_config.map_or(false, |cfg| cfg.overlaps(start_time, end_time));

    let mut examined_count = 0;
    let mut matched_count = 0;

    if should_debug {
        eprintln!(
            "\n[DEBUG LIVE] Phase '{}' ({} to {})",
            phase_name,
            start_time,
            end_time.unwrap_or("active")
        );
    }

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let event: TranscriptEvent = serde_json::from_str(line)?;

        // Only process assistant events
        if event.event_type != "assistant" {
            continue;
        }

        examined_count += 1;

        // Check if timestamp is in phase range
        let event_timestamp = match &event.timestamp {
            Some(ts) => ts,
            None => {
                if should_debug && debug_config.map_or(false, |cfg| cfg.verbose) {
                    eprintln!("  - Event #{}: NO TIMESTAMP → SKIPPED", examined_count);
                }
                continue;
            }
        };

        let in_range = is_in_range(event_timestamp, start_time, end_time);

        // Extract token usage
        let usage = event
            .usage
            .or_else(|| event.message.as_ref().and_then(|m| m.usage.clone()));

        if in_range {
            if let Some(usage) = usage {
                matched_count += 1;
                metrics.total_input_tokens += usage.input_tokens;
                metrics.total_output_tokens += usage.output_tokens;
                metrics.total_cache_creation_tokens +=
                    usage.cache_creation_input_tokens.unwrap_or(0);
                metrics.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
                metrics.assistant_turns += 1;

                if should_debug && debug_config.map_or(false, |cfg| cfg.verbose) {
                    eprintln!(
                        "  - Event #{} at {}: {} in + {} out → MATCHED",
                        examined_count, event_timestamp, usage.input_tokens, usage.output_tokens
                    );
                }
            } else if should_debug && debug_config.map_or(false, |cfg| cfg.verbose) {
                eprintln!(
                    "  - Event #{} at {}: NO USAGE DATA → MATCHED (no tokens)",
                    examined_count, event_timestamp
                );
            }
        } else if should_debug && debug_config.map_or(false, |cfg| cfg.verbose) {
            eprintln!(
                "  - Event #{} at {}: OUT OF RANGE → SKIPPED",
                examined_count, event_timestamp
            );
        }
    }

    if should_debug && !debug_config.map_or(false, |cfg| cfg.json_output) {
        // Only print text summary if not in JSON mode
        let total_tokens = metrics.total_input_tokens + metrics.total_output_tokens;
        eprintln!(
            "  Summary: examined {} events, matched {}, attributed {} tokens",
            examined_count, matched_count, total_tokens
        );
    }

    Ok((metrics, examined_count, matched_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_metrics_empty_transitions() {
        let metrics = build_phase_metrics(&[], &HookMetrics::default(), None, None).unwrap();
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

        let metrics =
            build_phase_metrics(&transitions, &HookMetrics::default(), None, None).unwrap();

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
