use crate::metrics::{
    BashCommand, FileModification, HookMetrics, PhaseMetrics, StateTransitionEvent, TokenMetrics,
};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Build per-phase metrics from state transitions, hooks, and transcripts
pub fn build_phase_metrics(
    transitions: &[StateTransitionEvent],
    hook_metrics: &HookMetrics,
    transcript_files: &[PathBuf],
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

        // Aggregate tokens from all transcript files for this phase
        let (token_metrics, examined_count, matched_count) = if !transcript_files.is_empty() {
            aggregate_tokens_for_range(
                transcript_files,
                &start_time,
                end_time.as_deref(),
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
// TODO: Investigate if this function is still needed or can be removed
#[allow(dead_code)]
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

/// Aggregate token usage from multiple transcript files for a time range
///
/// Agent-agnostic function that scans a list of transcript files and
/// accumulates token metrics for events within [start, end) time range.
///
/// Returns (TokenMetrics, total_examined, total_matched)
pub fn aggregate_tokens_for_range(
    transcript_files: &[PathBuf],
    start_time: &str,
    end_time: Option<&str>,
    debug_config: Option<&crate::metrics::DebugConfig>,
) -> Result<(TokenMetrics, usize, usize)> {
    use crate::metrics::transcript::TranscriptEvent;

    let mut total_metrics = TokenMetrics::default();
    let mut total_examined = 0;
    let mut total_matched = 0;

    // Debug: Check if we should output for this time range
    let should_debug = debug_config.map_or(false, |cfg| cfg.overlaps(start_time, end_time));

    if should_debug && !debug_config.map_or(false, |cfg| cfg.json_output) {
        eprintln!(
            "\n[DEBUG] Scanning {} transcript files for range {} to {}",
            transcript_files.len(),
            start_time,
            end_time.unwrap_or("active")
        );
    }

    // Stream each transcript file
    for transcript_path in transcript_files {
        if should_debug && debug_config.map_or(false, |cfg| cfg.verbose) {
            eprintln!("  Scanning file: {}", transcript_path.display());
        }

        let content = match fs::read_to_string(transcript_path) {
            Ok(c) => c,
            Err(e) => {
                if should_debug {
                    eprintln!("    Warning: Failed to read file: {}", e);
                }
                continue;
            }
        };

        let mut file_examined = 0;
        let mut file_matched = 0;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let event: TranscriptEvent = match serde_json::from_str(line) {
                Ok(e) => e,
                Err(_) => continue, // Skip malformed lines
            };

            // Only process assistant events
            if event.event_type != "assistant" {
                continue;
            }

            file_examined += 1;

            // Check if timestamp is in time range
            let event_timestamp = match &event.timestamp {
                Some(ts) => ts,
                None => continue,
            };

            if !is_in_range(event_timestamp, start_time, end_time) {
                continue;
            }

            // Extract token usage (handle both formats)
            let usage = event
                .usage
                .or_else(|| event.message.as_ref().and_then(|m| m.usage.clone()));

            if let Some(usage) = usage {
                file_matched += 1;
                total_metrics.total_input_tokens += usage.input_tokens;
                total_metrics.total_output_tokens += usage.output_tokens;
                total_metrics.total_cache_creation_tokens +=
                    usage.cache_creation_input_tokens.unwrap_or(0);
                total_metrics.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
                total_metrics.assistant_turns += 1;
            }
        }

        if should_debug && debug_config.map_or(false, |cfg| cfg.verbose) {
            eprintln!("    Examined: {}, Matched: {}", file_examined, file_matched);
        }

        total_examined += file_examined;
        total_matched += file_matched;
    }

    if should_debug && !debug_config.map_or(false, |cfg| cfg.json_output) {
        let total_tokens = total_metrics.total_input_tokens + total_metrics.total_output_tokens;
        eprintln!(
            "  Total: examined {} events across {} files, matched {}, attributed {} tokens",
            total_examined,
            transcript_files.len(),
            total_matched,
            total_tokens
        );
    }

    Ok((total_metrics, total_examined, total_matched))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_metrics_empty_transitions() {
        let metrics = build_phase_metrics(&[], &HookMetrics::default(), &[], None).unwrap();
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
            build_phase_metrics(&transitions, &HookMetrics::default(), &[], None).unwrap();

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
