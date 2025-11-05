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
            .or_insert_with(Vec::new)
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
        totals.tokens.input += phase.tokens.input;
        totals.tokens.output += phase.tokens.output;
        totals.tokens.cache_creation += phase.tokens.cache_creation;
        totals.tokens.cache_read += phase.tokens.cache_read;
        totals.tokens.assistant_turns += phase.tokens.assistant_turns;
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
