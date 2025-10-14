mod commands;
mod engine;
mod guardrails;
mod metrics;
mod rules;
mod storage;
mod tui;

#[cfg(test)]
mod test_helpers;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use storage::FileStorage;

#[derive(Parser)]
#[command(name = "hegel")]
#[command(about = "Dialectic-Driven Development CLI", long_about = None)]
struct Cli {
    /// Override state directory (default: .hegel)
    #[arg(long, global = true, value_name = "PATH")]
    state_dir: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new workflow
    Start {
        /// Workflow name (e.g., discovery, execution)
        workflow: String,
    },
    /// Get next prompt based on claims
    Next {
        /// Claims as JSON string (e.g., '{"spec_complete": true}')
        claims: String,
    },
    /// Continue workflow after interrupt (bypasses rules)
    Continue,
    /// Show current workflow status
    Status,
    /// Reset workflow state
    Reset,
    /// Handle Claude Code hook events
    Hook {
        /// Hook event name (e.g., PostToolUse, PreToolUse)
        event_name: String,
    },
    /// Analyze captured metrics (hooks, transcripts, states)
    Analyze,
    /// Interactive TUI dashboard (real-time metrics)
    Top,
    /// AST-based code search and transformation (wraps ast-grep)
    Astq {
        /// Arguments to pass to ast-grep
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch ephemeral Markdown review UI (wraps mirror)
    Reflect {
        /// Markdown files to review
        files: Vec<std::path::PathBuf>,
        /// Output directory for review files
        #[arg(long)]
        out_dir: Option<std::path::PathBuf>,
        /// Emit JSON with review file paths on exit
        #[arg(long)]
        json: bool,
        /// Headless mode (no-op, for testing)
        #[arg(long)]
        headless: bool,
    },
    /// Run git with guardrails and audit logging
    Git {
        /// Arguments to pass to git
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run docker with guardrails and audit logging
    Docker {
        /// Arguments to pass to docker
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If no command provided, show the coming soon message
    if cli.command.is_none() {
        println!(
            "{}",
            "Hegel - Dialectic-Driven Development CLI".bold().cyan()
        );
        println!();
        println!(
            "{}",
            "Thesis. Antithesis. Synthesis.".italic().bright_white()
        );
        println!();
        println!("{}", "Coming soon...".yellow());
        println!();
        println!(
            "{} {}",
            "Learn more at:".white(),
            "https://dialectician.ai".bright_blue().underline()
        );
        return Ok(());
    }

    // Initialize storage with resolved state directory
    let state_dir = FileStorage::resolve_state_dir(cli.state_dir)?;
    let storage = FileStorage::new(state_dir)?;

    // Execute command
    match cli.command.unwrap() {
        Commands::Start { workflow } => {
            commands::start_workflow(&workflow, &storage)?;
        }
        Commands::Next { claims } => {
            commands::next_prompt(&claims, &storage)?;
        }
        Commands::Continue => {
            commands::continue_prompt(&storage)?;
        }
        Commands::Status => {
            commands::show_status(&storage)?;
        }
        Commands::Reset => {
            commands::reset_workflow(&storage)?;
        }
        Commands::Hook { event_name } => {
            commands::handle_hook(&event_name, &storage)?;
        }
        Commands::Analyze => {
            commands::analyze_metrics(&storage)?;
        }
        Commands::Top => {
            tui::run_tui(storage.state_dir())?;
        }
        Commands::Astq { args } => {
            commands::run_astq(&args)?;
        }
        Commands::Reflect {
            files,
            out_dir,
            json,
            headless,
        } => {
            commands::run_reflect(&files, out_dir.as_deref(), json, headless)?;
        }
        Commands::Git { args } => {
            commands::run_wrapped_command("git", &args, &storage)?;
        }
        Commands::Docker { args } => {
            commands::run_wrapped_command("docker", &args, &storage)?;
        }
    }

    Ok(())
}
