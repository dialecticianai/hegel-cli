use anyhow::Result;
use colored::Colorize;

use crate::metrics::parse_unified_metrics;
use crate::storage::FileStorage;

pub fn analyze_metrics(storage: &FileStorage) -> Result<()> {
    println!("{}", "=== Hegel Metrics Analysis ===".bold().cyan());
    println!();

    let metrics = parse_unified_metrics(storage.state_dir())?;

    // Session Info
    println!("{}", "Session".bold());
    if let Some(session_id) = &metrics.session_id {
        println!("  ID: {}", session_id.bright_black());
    } else {
        println!("  {}", "No session data found".yellow());
    }
    println!();

    // Token Metrics
    println!("{}", "Token Usage".bold());
    if metrics.token_metrics.assistant_turns > 0 {
        println!(
            "  Input tokens:        {}",
            format!("{:>10}", metrics.token_metrics.total_input_tokens).cyan()
        );
        println!(
            "  Output tokens:       {}",
            format!("{:>10}", metrics.token_metrics.total_output_tokens).cyan()
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
            format!("{:>10}", total_tokens).bold().green()
        );
    } else {
        println!("  {}", "No token data found".yellow());
    }
    println!();

    // Hook Metrics
    println!("{}", "Activity".bold());
    println!(
        "  Total events:        {}",
        format!("{:>10}", metrics.hook_metrics.total_events).cyan()
    );
    println!(
        "  Bash commands:       {}",
        format!("{:>10}", metrics.hook_metrics.bash_commands.len()).cyan()
    );
    println!(
        "  File modifications:  {}",
        format!("{:>10}", metrics.hook_metrics.file_modifications.len()).cyan()
    );
    println!();

    // Top Bash Commands
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

    // Top File Modifications
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

    // State Transitions
    if !metrics.state_transitions.is_empty() {
        println!("{}", "Workflow Transitions".bold());
        println!(
            "  Total transitions:   {}",
            format!("{:>10}", metrics.state_transitions.len()).cyan()
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

    // Phase Breakdown
    if !metrics.phase_metrics.is_empty() {
        println!("{}", "Phase Breakdown".bold());
        for phase in &metrics.phase_metrics {
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
                    format!("{:>10}", total_tokens).cyan(),
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
                    format!("{:>10}", phase.bash_commands.len())
                        .cyan()
                        .to_string()
                }
            );
            println!(
                "    File edits:        {}",
                if phase.file_modifications.is_empty() {
                    "-".bright_black().to_string()
                } else {
                    format!("{:>10}", phase.file_modifications.len())
                        .cyan()
                        .to_string()
                }
            );
        }
        println!();
    }

    Ok(())
}
