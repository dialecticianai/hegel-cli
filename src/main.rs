mod commands;
mod engine;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use storage::FileStorage;

#[derive(Parser)]
#[command(name = "hegel")]
#[command(about = "Dialectic-Driven Development CLI", long_about = None)]
struct Cli {
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
    /// Show current workflow status
    Status,
    /// Reset workflow state
    Reset,
    /// Handle Claude Code hook events
    Hook {
        /// Hook event name (e.g., PostToolUse, PreToolUse)
        event_name: String,
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

    // Initialize storage
    let state_dir = FileStorage::default_state_dir()?;
    let storage = FileStorage::new(state_dir)?;

    // Execute command
    match cli.command.unwrap() {
        Commands::Start { workflow } => {
            commands::start_workflow(&workflow, &storage)?;
        }
        Commands::Next { claims } => {
            commands::next_prompt(&claims, &storage)?;
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
    }

    Ok(())
}
