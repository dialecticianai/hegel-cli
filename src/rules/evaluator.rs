use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use regex::Regex;

use super::types::{RuleConfig, RuleEvaluationContext, RuleViolation};

/// Extract the `HH:MM:SS` time portion (chars 11..19) from an RFC3339
/// timestamp, falling back to the whole string when it is too short (e.g. a
/// missing-timestamp placeholder). Never panics on the slice.
pub(crate) fn hhmmss(ts: &str) -> &str {
    ts.get(11..19).unwrap_or(ts)
}

/// The phase metrics for the context's current phase, if present.
fn current_phase_metrics<'a>(
    context: &'a RuleEvaluationContext,
) -> Option<&'a crate::metrics::PhaseMetrics> {
    context
        .all_phase_metrics
        .iter()
        .find(|p| p.phase_name == context.current_phase)
}

/// Compute the `[phase_start, phase_start + window]` bounds and compile the
/// optional pattern. Falls back to "now" as the start for old state files
/// missing `phase_start_time`.
#[allow(clippy::type_complexity)]
fn phase_window(
    context: &RuleEvaluationContext,
    pattern: &Option<String>,
    window: u64,
) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>, Option<Regex>)> {
    let phase_start = context
        .phase_start_time
        .as_ref()
        .map(|s| DateTime::parse_from_rfc3339(s))
        .transpose()?
        .unwrap_or_else(|| chrono::Utc::now().into());
    let window_end = phase_start + chrono::Duration::seconds(window as i64);
    let regex = pattern.as_ref().map(|p| Regex::new(p)).transpose()?;
    Ok((phase_start, window_end, regex))
}

/// Items whose timestamp falls within `[phase_start, window_end]` and whose
/// `text` matches `regex` (or all, when no pattern).
fn matches_in_window<'a, T>(
    items: &'a [T],
    phase_start: DateTime<FixedOffset>,
    window_end: DateTime<FixedOffset>,
    regex: &Option<Regex>,
    timestamp: impl Fn(&T) -> Option<&String>,
    text: impl Fn(&T) -> &str,
) -> Vec<&'a T> {
    items
        .iter()
        .filter(|item| {
            let in_window = timestamp(item)
                .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                .is_some_and(|ts| ts >= phase_start && ts <= window_end);
            in_window && regex.as_ref().is_none_or(|re| re.is_match(text(item)))
        })
        .collect()
}

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

    let (phase_start, window_end, regex) = phase_window(context, path_pattern, *window)?;

    // Filter file modifications by time window and pattern
    let matching_edits = matches_in_window(
        &context.hook_metrics.file_modifications,
        phase_start,
        window_end,
        &regex,
        |f| f.timestamp.as_ref(),
        |f| &f.file_path,
    );

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
                    hhmmss(file_mod.timestamp.as_deref().unwrap_or("unknown")),
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

    let phase_metrics = match current_phase_metrics(context) {
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

    let phase_metrics = match current_phase_metrics(context) {
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
            format!("Phase start: {}", hhmmss(&phase_metrics.start_time)),
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

    let (phase_start, window_end, regex) = phase_window(context, pattern, *window)?;

    // Filter commands by time window and pattern
    let matching_commands = matches_in_window(
        &context.hook_metrics.bash_commands,
        phase_start,
        window_end,
        &regex,
        |c| c.timestamp.as_ref(),
        |c| &c.command,
    );

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
                    hhmmss(cmd.timestamp.as_deref().unwrap_or("unknown")),
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
