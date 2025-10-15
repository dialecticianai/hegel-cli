use crate::metrics::{PhaseMetrics, UnifiedMetrics, WorkflowDAG};
use colored::Colorize;
use std::fmt::Display;

/// Format a numeric metric with right alignment and cyan color
fn format_metric(value: impl Display) -> String {
    format!("{:>10}", value).cyan().to_string()
}

/// Format a total metric with bold green styling
fn format_total(value: impl Display) -> String {
    format!("{:>10}", value).bold().green().to_string()
}

/// Render session information section
pub fn render_session(metrics: &UnifiedMetrics) {
    println!("{}", "Session".bold());
    if let Some(session_id) = &metrics.session_id {
        println!("  ID: {}", session_id.bright_black());
    } else {
        println!("  {}", "No session data found".yellow());
    }
    println!();
}

/// Render token usage section
pub fn render_tokens(metrics: &UnifiedMetrics) {
    println!("{}", "Token Usage".bold());
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
            format!("{:>10}", metrics.token_metrics.total_cache_creation_tokens).bright_black()
        );
        println!(
            "  Cache reads:         {}",
            format!("{:>10}", metrics.token_metrics.total_cache_read_tokens).bright_black()
        );
        println!(
            "  Assistant turns:     {}",
            format!("{:>10}", metrics.token_metrics.assistant_turns).green()
        );

        let total_tokens = metrics.token_metrics.total_input_tokens
            + metrics.token_metrics.total_output_tokens
            + metrics.token_metrics.total_cache_creation_tokens
            + metrics.token_metrics.total_cache_read_tokens;
        println!(
            "  {}            {}",
            "Total:".bold(),
            format_total(total_tokens)
        );
    } else {
        println!("  {}", "No token data found".yellow());
    }
    println!();
}

/// Render activity summary section
pub fn render_activity(metrics: &UnifiedMetrics) {
    println!("{}", "Activity".bold());
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
        println!("{}", "Top Bash Commands".bold());
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
                count.to_string().green(),
                truncated.bright_black()
            );
        }
        println!();
    }
}

/// Render top file modifications section
pub fn render_top_file_modifications(metrics: &UnifiedMetrics) {
    if !metrics.hook_metrics.file_modifications.is_empty() {
        println!("{}", "Top File Modifications".bold());
        let mut freq = metrics.hook_metrics.file_modification_frequency();
        let mut sorted: Vec<_> = freq.drain().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (file, count) in sorted.iter().take(10) {
            println!(
                "  {:>3}x {}",
                count.to_string().green(),
                file.bright_black()
            );
        }
        println!();
    }
}

/// Render state transitions section
pub fn render_state_transitions(metrics: &UnifiedMetrics) {
    if !metrics.state_transitions.is_empty() {
        println!("{}", "Workflow Transitions".bold());
        println!(
            "  Total transitions:   {}",
            format_metric(metrics.state_transitions.len())
        );

        if let Some(first) = metrics.state_transitions.first() {
            println!("  Mode:                {}", first.mode.cyan());
        }

        println!();
        println!("{}", "  Transition History:".bold());
        for transition in &metrics.state_transitions {
            println!(
                "    {} {} {}  ({})",
                transition.from_node.bright_black(),
                "→".bright_black(),
                transition.to_node.cyan(),
                transition.timestamp.bright_black()
            );
        }
        println!();
    } else {
        println!("{}", "Workflow Transitions".bold());
        println!("  {}", "No workflow transitions found".yellow());
        println!();
    }
}

/// Render phase breakdown section
pub fn render_phase_breakdown(phase_metrics: &[PhaseMetrics]) {
    if !phase_metrics.is_empty() {
        println!("{}", "Phase Breakdown".bold());
        for phase in phase_metrics {
            let status = if phase.end_time.is_none() {
                "active".green()
            } else {
                "completed".bright_black()
            };

            // Format duration
            let duration_str = if phase.duration_seconds > 0 {
                let minutes = phase.duration_seconds / 60;
                let seconds = phase.duration_seconds % 60;
                format!("{}m {:02}s", minutes, seconds).cyan().to_string()
            } else {
                "-".bright_black().to_string()
            };

            println!();
            println!(
                "  {} ({})",
                phase.phase_name.to_uppercase().bold().cyan(),
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
                    format!("{:>10}", phase.token_metrics.assistant_turns).green()
                );
            } else {
                println!("    Tokens:            {}", "-".bright_black());
            }

            // Activity
            println!(
                "    Bash commands:     {}",
                if phase.bash_commands.is_empty() {
                    "-".bright_black().to_string()
                } else {
                    format_metric(phase.bash_commands.len())
                }
            );
            println!(
                "    File edits:        {}",
                if phase.file_modifications.is_empty() {
                    "-".bright_black().to_string()
                } else {
                    format_metric(phase.file_modifications.len())
                }
            );
        }
        println!();
    }
}

/// Render workflow graph section
pub fn render_workflow_graph(metrics: &UnifiedMetrics) {
    if !metrics.state_transitions.is_empty() && !metrics.phase_metrics.is_empty() {
        println!("{}", "Workflow Graph".bold());
        println!();

        let graph =
            WorkflowDAG::from_transitions(&metrics.state_transitions, &metrics.phase_metrics);

        // ASCII visualization
        print!("{}", graph.render_ascii());

        // Cycle detection
        let cycles = graph.find_cycles();
        if !cycles.is_empty() {
            println!("{}", "⚠ Cycles Detected:".yellow().bold());
            for cycle in cycles {
                println!("  {}", cycle.join(" → ").yellow());
            }
            println!();
        }

        println!("{}", "Export Options:".bright_black());
        println!(
            "  {} hegel graph --dot > workflow.dot",
            "Run".bright_black()
        );
        println!(
            "  {} dot -Tpng workflow.dot -o workflow.png",
            "Then".bright_black()
        );
        println!();
    }
}
