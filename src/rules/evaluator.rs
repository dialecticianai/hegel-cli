use anyhow::Result;
use chrono::DateTime;
use regex::Regex;

use super::types::{RuleConfig, RuleEvaluationContext, RuleViolation};

/// Evaluate all rules and return the first violation (short-circuit)
pub fn evaluate_rules(
    rules: &[RuleConfig],
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    // Evaluate each rule in order, short-circuit on first violation
    for rule in rules {
        let violation = match rule {
            RuleConfig::RepeatedCommand { .. } => evaluate_repeated_command(rule, context)?,
            RuleConfig::RepeatedFileEdit { .. } => evaluate_repeated_file_edit(rule, context)?,
            RuleConfig::PhaseTimeout { .. } => evaluate_phase_timeout(rule, context)?,
            RuleConfig::TokenBudget { .. } => evaluate_token_budget(rule, context)?,
            RuleConfig::RequireCommits { .. } => evaluate_require_commits(rule, context)?,
        };

        if violation.is_some() {
            return Ok(violation); // Short-circuit on first violation
        }
    }

    Ok(None) // No violations
}

/// Evaluate a require_commits rule
pub(crate) fn evaluate_require_commits(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let lookback_phases = match rule {
        RuleConfig::RequireCommits { lookback_phases } => lookback_phases,
        _ => return Ok(None),
    };

    // Check global config: if commit_guard is disabled, skip rule
    if !context.config.commit_guard {
        return Ok(None);
    }

    // Check git availability
    // If use_git is explicitly false, skip rule
    if context.config.use_git == Some(false) {
        return Ok(None);
    }

    // If no git repo detected and use_git is not explicitly true, skip rule
    if let Some(git_info) = context.git_info {
        if !git_info.has_repo && context.config.use_git != Some(true) {
            return Ok(None);
        }
    } else if context.config.use_git != Some(true) {
        // No git_info and use_git not explicitly true, skip
        return Ok(None);
    }

    // Find current phase index in all_phase_metrics
    let current_idx = context
        .all_phase_metrics
        .iter()
        .position(|p| p.phase_name == context.current_phase);

    let current_idx = match current_idx {
        Some(idx) => idx,
        None => return Ok(None), // Current phase not found, skip gracefully
    };

    // Calculate lookback window
    let start_idx = current_idx.saturating_sub(lookback_phases - 1);
    let phases_to_check = &context.all_phase_metrics[start_idx..=current_idx];

    // Collect all git_commits from phases in window
    let all_commits: Vec<_> = phases_to_check
        .iter()
        .flat_map(|p| &p.git_commits)
        .collect();

    // If no commits found, return violation
    if all_commits.is_empty() {
        Ok(Some(RuleViolation {
            rule_type: "Require Commits".to_string(),
            diagnostic: format!("No commits found in last {} phases", lookback_phases),
            suggestion: "Create a commit before advancing. Use `hegel next --force require_commits` to override.".to_string(),
            recent_events: vec![],
        }))
    } else {
        Ok(None) // Commits found, no violation
    }
}

/// Evaluate a repeated_file_edit rule
pub(crate) fn evaluate_repeated_file_edit(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let (path_pattern, threshold, window) = match rule {
        RuleConfig::RepeatedFileEdit {
            path_pattern,
            threshold,
            window,
        } => (path_pattern, threshold, window),
        _ => return Ok(None),
    };

    // Calculate window bounds: [phase_start, phase_start + window]
    // If phase_start_time is missing, use current time (graceful fallback for old state files)
    let phase_start = context
        .phase_start_time
        .as_ref()
        .map(|s| DateTime::parse_from_rfc3339(s))
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().into());
    let window_end = phase_start + chrono::Duration::seconds(*window as i64);

    // Compile regex if pattern provided
    let regex = if let Some(pat) = path_pattern {
        Some(Regex::new(pat)?)
    } else {
        None
    };

    // Filter file modifications by time window and pattern
    let matching_edits: Vec<_> = context
        .hook_metrics
        .file_modifications
        .iter()
        .filter(|file_mod| {
            // Filter by timestamp (within window: phase_start <= ts <= window_end)
            if let Some(ts) = &file_mod.timestamp {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(ts) {
                    if timestamp < phase_start || timestamp > window_end {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }

            // Filter by pattern (if provided)
            if let Some(ref re) = regex {
                re.is_match(&file_mod.file_path)
            } else {
                true // No pattern = match all
            }
        })
        .collect();

    let count = matching_edits.len();

    if count >= *threshold {
        // Build recent events list (last 5)
        let recent_events: Vec<String> = matching_edits
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|file_mod| {
                format!(
                    "{}: {} ({})",
                    file_mod
                        .timestamp
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())[11..19]
                        .to_string(),
                    file_mod.file_path,
                    file_mod.tool
                )
            })
            .collect();

        Ok(Some(RuleViolation {
            rule_type: "Repeated File Edit".to_string(),
            diagnostic: format!("Files edited {} times in last {}s", count, window),
            suggestion: "You're thrashing the same files. Step back and write a failing test that captures the desired behavior, then implement the fix.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}
/// Evaluate a token_budget rule
pub(crate) fn evaluate_token_budget(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let max_tokens = match rule {
        RuleConfig::TokenBudget { max_tokens } => max_tokens,
        _ => return Ok(None),
    };

    // Find current phase in all_phase_metrics by name
    let phase_metrics = context
        .all_phase_metrics
        .iter()
        .find(|p| p.phase_name == context.current_phase);

    let phase_metrics = match phase_metrics {
        Some(pm) => pm,
        None => return Ok(None), // No matching phase metrics available
    };

    // Calculate total tokens (input + output, per SPEC cache tokens excluded)
    let total_tokens = phase_metrics.token_metrics.total_input_tokens
        + phase_metrics.token_metrics.total_output_tokens;

    if total_tokens > *max_tokens {
        let recent_events = vec![
            format!(
                "Input tokens: {}",
                phase_metrics.token_metrics.total_input_tokens
            ),
            format!(
                "Output tokens: {}",
                phase_metrics.token_metrics.total_output_tokens
            ),
            format!("Total: {} (limit: {})", total_tokens, max_tokens),
            format!("Turns: {}", phase_metrics.token_metrics.assistant_turns),
        ];

        Ok(Some(RuleViolation {
            rule_type: "Token Budget".to_string(),
            diagnostic: format!(
                "{} phase used {} tokens (limit: {})",
                phase_metrics.phase_name, total_tokens, max_tokens
            ),
            suggestion: "You've exceeded the token budget for this phase. Consider simplifying scope, deferring non-critical work, or transitioning to document progress before resetting.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}

/// Evaluate a phase_timeout rule
pub(crate) fn evaluate_phase_timeout(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let max_duration = match rule {
        RuleConfig::PhaseTimeout { max_duration } => max_duration,
        _ => return Ok(None),
    };

    // Find current phase in all_phase_metrics by name
    let phase_metrics = context
        .all_phase_metrics
        .iter()
        .find(|p| p.phase_name == context.current_phase);

    let phase_metrics = match phase_metrics {
        Some(pm) => pm,
        None => return Ok(None), // No matching phase metrics available
    };

    // Calculate duration
    let duration_secs = if let Some(end_time) = &phase_metrics.end_time {
        // Completed phase - calculate from timestamps
        let start = DateTime::parse_from_rfc3339(&phase_metrics.start_time)?;
        let end = DateTime::parse_from_rfc3339(end_time)?;
        (end - start).num_seconds() as u64
    } else {
        // Active phase - calculate from current time
        let start = DateTime::parse_from_rfc3339(&phase_metrics.start_time)?;
        let now = chrono::Utc::now().with_timezone(start.offset());
        (now - start).num_seconds() as u64
    };

    if duration_secs > *max_duration {
        let minutes = duration_secs / 60;
        let seconds = duration_secs % 60;
        let limit_minutes = max_duration / 60;

        let recent_events = vec![
            format!("Phase start: {}", &phase_metrics.start_time[11..19]),
            format!("Duration: {}m {}s", minutes, seconds),
            format!("Limit: {}m", limit_minutes),
        ];

        Ok(Some(RuleViolation {
            rule_type: "Phase Timeout".to_string(),
            diagnostic: format!(
                "{} phase running for {}s (limit: {}s)",
                phase_metrics.phase_name, duration_secs, max_duration
            ),
            suggestion: "This phase is taking too long. Consider breaking the task into smaller steps, transitioning to LEARNINGS to document blockers, or resetting with simplified scope.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}

/// Evaluate a repeated_command rule
pub(crate) fn evaluate_repeated_command(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let (pattern, threshold, window) = match rule {
        RuleConfig::RepeatedCommand {
            pattern,
            threshold,
            window,
        } => (pattern, threshold, window),
        _ => return Ok(None),
    };

    // Skip evaluation if no phase_start_time available (gracefully handled in fallback below)
    if context.phase_start_time.is_none() {
        // Will use current time as fallback
    }

    // Calculate window bounds: [phase_start, phase_start + window]
    // If phase_start_time is missing, use current time (graceful fallback for old state files)
    let phase_start = context
        .phase_start_time
        .as_ref()
        .map(|s| DateTime::parse_from_rfc3339(s))
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().into());
    let window_end = phase_start + chrono::Duration::seconds(*window as i64);

    // Compile regex if pattern provided
    let regex = if let Some(pat) = pattern {
        Some(Regex::new(pat)?)
    } else {
        None
    };

    // Filter commands by time window and pattern
    let matching_commands: Vec<_> = context
        .hook_metrics
        .bash_commands
        .iter()
        .filter(|cmd| {
            // Filter by timestamp (within window: phase_start <= ts <= window_end)
            if let Some(ts) = &cmd.timestamp {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(ts) {
                    if timestamp < phase_start || timestamp > window_end {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }

            // Filter by pattern (if provided)
            if let Some(ref re) = regex {
                re.is_match(&cmd.command)
            } else {
                true // No pattern = match all
            }
        })
        .collect();

    let count = matching_commands.len();

    if count >= *threshold {
        // Build recent events list (last 5)
        let recent_events: Vec<String> = matching_commands
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|cmd| {
                format!(
                    "{}: {}",
                    cmd.timestamp.as_ref().unwrap_or(&"unknown".to_string())[11..19].to_string(),
                    cmd.command
                )
            })
            .collect();

        Ok(Some(RuleViolation {
            rule_type: "Repeated Command".to_string(),
            diagnostic: format!("Command executed {} times in last {}s", count, window),
            suggestion: "You're stuck in a build loop. Review the error message carefully. Consider using TDD: write a failing test first, then fix the specific issue.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}
