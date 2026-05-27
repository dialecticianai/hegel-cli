use std::collections::HashMap;

use crate::metrics::{BashCommand, FileModification, HookMetrics};

use super::{BashCommandSummary, FileModificationSummary, WorkflowTotals};

/// Generic aggregation helper - DRY principle
fn aggregate_by_key<T, K, F>(items: &[T], key_fn: F) -> HashMap<K, Vec<String>>
where
    K: std::hash::Hash + Eq,
    F: Fn(&T) -> (K, Option<String>),
{
    let mut freq: HashMap<K, Vec<String>> = HashMap::new();
    for item in items {
        let (key, timestamp) = key_fn(item);
        freq.entry(key)
            .or_default()
            .push(timestamp.unwrap_or_default());
    }
    freq
}

/// Aggregate bash commands by command string
pub fn aggregate_bash_commands(bash_commands: &[BashCommand]) -> Vec<BashCommandSummary> {
    let freq = aggregate_by_key(bash_commands, |cmd| {
        (cmd.command.clone(), cmd.timestamp.clone())
    });

    freq.into_iter()
        .map(|(command, timestamps)| BashCommandSummary {
            count: timestamps.len(),
            command,
            timestamps,
        })
        .collect()
}

/// Expand archived bash-command summaries back into individual commands.
///
/// The reverse of [`aggregate_bash_commands`]: each summary yields `count`
/// commands so downstream consumers (e.g. the TUI phase table) can count them.
/// `stdout`/`stderr` are not archived, so they are left empty; timestamps are
/// restored positionally where present.
pub fn expand_bash_commands(summaries: &[BashCommandSummary]) -> Vec<BashCommand> {
    summaries
        .iter()
        .flat_map(|s| {
            (0..s.count).map(move |i| BashCommand {
                command: s.command.clone(),
                timestamp: s.timestamps.get(i).filter(|t| !t.is_empty()).cloned(),
                stdout: None,
                stderr: None,
            })
        })
        .collect()
}

/// Expand archived file-modification summaries back into individual
/// modifications. The reverse of [`aggregate_file_modifications`].
pub fn expand_file_modifications(summaries: &[FileModificationSummary]) -> Vec<FileModification> {
    summaries
        .iter()
        .flat_map(|s| {
            (0..s.count).map(move |i| FileModification {
                file_path: s.file_path.clone(),
                tool: s.tool.clone(),
                timestamp: s.timestamps.get(i).filter(|t| !t.is_empty()).cloned(),
            })
        })
        .collect()
}

/// Aggregate file modifications by (file_path, tool)
pub fn aggregate_file_modifications(
    file_modifications: &[FileModification],
) -> Vec<FileModificationSummary> {
    let freq = aggregate_by_key(file_modifications, |file_mod| {
        (
            (file_mod.file_path.clone(), file_mod.tool.clone()),
            file_mod.timestamp.clone(),
        )
    });

    freq.into_iter()
        .map(|((file_path, tool), timestamps)| FileModificationSummary {
            count: timestamps.len(),
            file_path,
            tool,
            timestamps,
        })
        .collect()
}

/// Compute workflow-level totals
pub fn compute_totals(
    phases: &[super::PhaseArchive],
    hook_metrics: &HookMetrics,
) -> WorkflowTotals {
    let mut totals = WorkflowTotals::default();

    // Sum tokens across phases
    for phase in phases {
        totals.tokens += &phase.tokens;
    }

    // Count bash commands and files
    totals.bash_commands = hook_metrics.bash_commands.len();
    totals.file_modifications = hook_metrics.file_modifications.len();

    // Unique counts
    let unique_commands: std::collections::HashSet<_> = hook_metrics
        .bash_commands
        .iter()
        .map(|c| &c.command)
        .collect();
    totals.unique_commands = unique_commands.len();

    let unique_files: std::collections::HashSet<_> = hook_metrics
        .file_modifications
        .iter()
        .map(|f| &f.file_path)
        .collect();
    totals.unique_files = unique_files.len();

    // Count git commits across all phases
    totals.git_commits = phases.iter().map(|p| p.git_commits.len()).sum();

    totals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_bash_commands_round_trips_counts() {
        let cmds = vec![
            BashCommand {
                command: "cargo build".into(),
                timestamp: Some("2025-01-01T10:00:00Z".into()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".into(),
                timestamp: Some("2025-01-01T10:05:00Z".into()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "ls".into(),
                timestamp: None,
                stdout: None,
                stderr: None,
            },
        ];
        let summaries = aggregate_bash_commands(&cmds);
        let expanded = expand_bash_commands(&summaries);
        // Total count is preserved through aggregate -> expand.
        assert_eq!(expanded.len(), 3);
        assert_eq!(
            expanded
                .iter()
                .filter(|c| c.command == "cargo build")
                .count(),
            2
        );
        // A present timestamp survives the round trip.
        assert!(expanded
            .iter()
            .any(|c| c.timestamp.as_deref() == Some("2025-01-01T10:00:00Z")));
    }

    #[test]
    fn test_expand_file_modifications_round_trips_counts() {
        let mods = vec![
            FileModification {
                file_path: "a.rs".into(),
                tool: "Edit".into(),
                timestamp: Some("2025-01-01T10:00:00Z".into()),
            },
            FileModification {
                file_path: "a.rs".into(),
                tool: "Edit".into(),
                timestamp: Some("2025-01-01T10:01:00Z".into()),
            },
        ];
        let summaries = aggregate_file_modifications(&mods);
        let expanded = expand_file_modifications(&summaries);
        assert_eq!(expanded.len(), 2);
        assert!(expanded
            .iter()
            .all(|m| m.file_path == "a.rs" && m.tool == "Edit"));
    }
}
