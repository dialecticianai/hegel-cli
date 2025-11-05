use crate::metrics::{PhaseMetrics, UnifiedMetrics, WorkflowDAG};
use crate::theme::Theme;
use colored::Colorize;
use std::fmt::Display;

/// Format a numeric metric with right alignment and theme styling
fn format_metric(value: impl Display) -> String {
    Theme::metric_value(format!("{:>10}", value)).to_string()
}

/// Format a total metric with theme styling
fn format_total(value: impl Display) -> String {
    Theme::metric_total(format!("{:>10}", value)).to_string()
}

/// Render session information section
pub fn render_session(metrics: &UnifiedMetrics) {
    println!("{}", Theme::header("Session"));
    if let Some(session_id) = &metrics.session_id {
        println!("  ID: {}", Theme::secondary(session_id));
    } else {
        println!("  {}", Theme::warning("No session data found"));
    }
    println!();
}

/// Render token usage section
pub fn render_tokens(metrics: &UnifiedMetrics) {
    println!("{}", Theme::label("Token Usage"));
    if metrics.token_metrics.assistant_turns > 0 {
        println!(
            "  Input tokens:        {}",
            format_metric(metrics.token_metrics.total_input_tokens)
        );
        println!(
            "  Output tokens:       {}",
            format_metric(metrics.token_metrics.total_output_tokens)
        );
        println!(
            "  Cache creation:      {}",
            Theme::secondary(format!(
                "{:>10}",
                metrics.token_metrics.total_cache_creation_tokens
            ))
        );
        println!(
            "  Cache reads:         {}",
            Theme::secondary(format!(
                "{:>10}",
                metrics.token_metrics.total_cache_read_tokens
            ))
        );
        println!(
            "  Assistant turns:     {}",
            Theme::success(format!("{:>10}", metrics.token_metrics.assistant_turns))
        );

        let total_tokens = metrics.token_metrics.total_input_tokens
            + metrics.token_metrics.total_output_tokens
            + metrics.token_metrics.total_cache_creation_tokens
            + metrics.token_metrics.total_cache_read_tokens;
        println!(
            "  {}            {}",
            Theme::label("Total:"),
            format_total(total_tokens)
        );
    } else {
        println!("  {}", Theme::warning("No token data found"));
    }
    println!();
}

/// Render activity summary section
pub fn render_activity(metrics: &UnifiedMetrics) {
    println!("{}", Theme::label("Activity"));
    println!(
        "  Total events:        {}",
        format_metric(metrics.hook_metrics.total_events)
    );
    println!(
        "  Bash commands:       {}",
        format_metric(metrics.hook_metrics.bash_commands.len())
    );
    println!(
        "  File modifications:  {}",
        format_metric(metrics.hook_metrics.file_modifications.len())
    );
    println!();
}

/// Render top bash commands section
pub fn render_top_bash_commands(metrics: &UnifiedMetrics) {
    if !metrics.hook_metrics.bash_commands.is_empty() {
        println!("{}", Theme::label("Top Bash Commands"));
        let mut freq = metrics.hook_metrics.bash_command_frequency();
        let mut sorted: Vec<_> = freq.drain().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (cmd, count) in sorted.iter().take(10) {
            let truncated = if cmd.len() > 60 {
                format!("{}...", &cmd[..57])
            } else {
                cmd.clone()
            };
            println!(
                "  {:>3}x {}",
                Theme::success(count.to_string()),
                Theme::secondary(truncated)
            );
        }
        println!();
    }
}

/// Render command output summary section
pub fn render_command_output_summary(metrics: &UnifiedMetrics) {
    if !metrics.hook_metrics.bash_commands.is_empty() {
        let commands_with_output = metrics
            .hook_metrics
            .bash_commands
            .iter()
            .filter(|cmd| cmd.stdout.is_some() || cmd.stderr.is_some())
            .count();

        let commands_with_errors = metrics
            .hook_metrics
            .bash_commands
            .iter()
            .filter(|cmd| {
                cmd.stderr
                    .as_ref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
            .count();

        if commands_with_output > 0 {
            println!("{}", Theme::label("Command Output Summary"));
            println!(
                "  Commands with output: {}",
                format_metric(commands_with_output)
            );
            println!(
                "  Commands with stderr: {}",
                if commands_with_errors > 0 {
                    Theme::warning(commands_with_errors.to_string())
                } else {
                    Theme::success("0")
                }
            );
            println!();
        }
    }
}

/// Render top file modifications section
pub fn render_top_file_modifications(metrics: &UnifiedMetrics) {
    if !metrics.hook_metrics.file_modifications.is_empty() {
        println!("{}", Theme::label("Top File Modifications"));
        let mut freq = metrics.hook_metrics.file_modification_frequency();
        let mut sorted: Vec<_> = freq.drain().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (file, count) in sorted.iter().take(10) {
            println!(
                "  {:>3}x {}",
                Theme::success(count.to_string()),
                Theme::secondary(file)
            );
        }
        println!();
    }
}

/// Render state transitions section
pub fn render_state_transitions(metrics: &UnifiedMetrics) {
    if !metrics.state_transitions.is_empty() {
        println!("{}", Theme::label("Workflow Transitions"));
        println!(
            "  Total transitions:   {}",
            format_metric(metrics.state_transitions.len())
        );

        if let Some(first) = metrics.state_transitions.first() {
            println!("  Mode:                {}", Theme::highlight(&first.mode));
        }

        println!();
        println!("{}", Theme::label("  Transition History:"));
        for transition in &metrics.state_transitions {
            println!(
                "    {} {} {}  ({})",
                Theme::secondary(&transition.from_node),
                Theme::secondary("→"),
                Theme::highlight(&transition.to_node),
                Theme::secondary(&transition.timestamp)
            );
        }
        println!();
    } else {
        println!("{}", Theme::label("Workflow Transitions"));
        println!("  {}", Theme::warning("No workflow transitions found"));
        println!();
    }
}

/// Render phase breakdown section
pub fn render_phase_breakdown(phase_metrics: &[PhaseMetrics]) {
    if !phase_metrics.is_empty() {
        println!("{}", Theme::label("Phase Breakdown"));
        for phase in phase_metrics {
            let is_done_node = phase.phase_name.to_lowercase() == "done";
            let status = if phase.end_time.is_none() && !is_done_node {
                Theme::success("active")
            } else if phase.is_synthetic {
                Theme::secondary("completed, synthetic")
            } else {
                Theme::secondary("completed")
            };

            // Format duration
            let duration_str = if phase.duration_seconds > 0 {
                let minutes = phase.duration_seconds / 60;
                let seconds = phase.duration_seconds % 60;
                Theme::highlight(format!("{}m {:02}s", minutes, seconds)).to_string()
            } else {
                Theme::secondary("-").to_string()
            };

            println!();
            println!(
                "  {} ({})",
                Theme::header(phase.phase_name.to_uppercase()),
                status
            );
            println!("    Duration:          {}", duration_str);

            // Tokens
            if phase.token_metrics.assistant_turns > 0 {
                let total_tokens = phase.token_metrics.total_input_tokens
                    + phase.token_metrics.total_output_tokens;
                println!(
                    "    Tokens:            {} ({} in, {} out)",
                    format_metric(total_tokens),
                    phase.token_metrics.total_input_tokens,
                    phase.token_metrics.total_output_tokens
                );
                println!(
                    "    Assistant turns:   {}",
                    Theme::success(format!("{:>10}", phase.token_metrics.assistant_turns))
                );
            } else {
                println!("    Tokens:            {}", Theme::secondary("-"));
            }

            // Activity
            println!(
                "    Bash commands:     {}",
                if phase.bash_commands.is_empty() {
                    Theme::secondary("-").to_string()
                } else {
                    format_metric(phase.bash_commands.len())
                }
            );
            println!(
                "    File edits:        {}",
                if phase.file_modifications.is_empty() {
                    Theme::secondary("-").to_string()
                } else {
                    format_metric(phase.file_modifications.len())
                }
            );

            // Git commits
            if !phase.git_commits.is_empty() {
                let total_files: usize = phase.git_commits.iter().map(|c| c.files_changed).sum();
                let total_insertions: usize = phase.git_commits.iter().map(|c| c.insertions).sum();
                let total_deletions: usize = phase.git_commits.iter().map(|c| c.deletions).sum();

                println!(
                    "    Commits:           {} ({} files, +{} -{})",
                    format_metric(phase.git_commits.len()),
                    total_files,
                    total_insertions,
                    total_deletions
                );
            } else {
                println!("    Commits:           {}", Theme::secondary("-"));
            }
        }
        println!();
    }
}

/// Render workflow graph section
pub fn render_workflow_graph(metrics: &UnifiedMetrics) {
    if !metrics.state_transitions.is_empty() && !metrics.phase_metrics.is_empty() {
        println!("{}", Theme::label("Workflow Graph"));
        println!();

        let graph =
            WorkflowDAG::from_transitions(&metrics.state_transitions, &metrics.phase_metrics);

        // ASCII visualization
        print!("{}", graph.render_ascii());

        // Cycle detection
        let cycles = graph.find_cycles();
        if !cycles.is_empty() {
            println!("{}", Theme::warning("⚠ Cycles Detected:").bold());
            for cycle in cycles {
                println!("  {}", Theme::warning(cycle.join(" → ")));
            }
            println!();
        }

        println!("{}", Theme::secondary("Export Options:"));
        println!(
            "  {} hegel analyze --export-dot > workflow.dot",
            Theme::secondary("Run")
        );
        println!(
            "  {} dot -Tpng workflow.dot -o workflow.png",
            Theme::secondary("Then")
        );
        println!();
    }
}

/// Export workflow graph as DOT format
pub fn render_workflow_graph_dot(metrics: &UnifiedMetrics) -> anyhow::Result<()> {
    if metrics.state_transitions.is_empty() || metrics.phase_metrics.is_empty() {
        eprintln!("No workflow data to export. Run a workflow first.");
        return Ok(());
    }

    let graph = WorkflowDAG::from_transitions(&metrics.state_transitions, &metrics.phase_metrics);
    println!("{}", graph.export_dot());
    Ok(())
}
