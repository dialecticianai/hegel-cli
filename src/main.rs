mod adapters;
mod commands;
mod config;
mod embedded;
mod engine;
mod guardrails;
mod metamodes;
mod metrics;
mod rules;
mod storage;
mod theme;
mod tui;

#[cfg(test)]
mod test_helpers;

use anyhow::Result;
use clap::{Parser, Subcommand};
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
    /// Initialize project (auto-detects greenfield or retrofit)
    Init,
    /// Start a new workflow
    Start {
        /// Workflow name (e.g., discovery, execution)
        workflow: String,
    },
    /// List available workflows
    Workflows,
    /// List available guides
    Guides,
    /// Advance to next phase (implicit: current_complete=true, or provide custom claims)
    Next {
        /// Optional claims as JSON string (e.g., '{"spec_complete": true}')
        /// If omitted, uses happy-path claim: {"{current}_complete": true}
        claims: Option<String>,
    },
    /// Repeat current phase (claim: current_complete=false)
    Repeat,
    /// Restart workflow cycle (claim: restart_cycle=true)
    Restart,
    /// Show current workflow status
    Status,
    /// Reset workflow state
    Reset,
    /// Abort current workflow (clears state)
    Abort,
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
    ///
    /// Pattern syntax (tree-sitter):
    ///   $VAR  - Single AST node (identifier, expression, etc.)
    ///   $$$   - Variadic (zero or more nodes)
    ///
    /// Examples:
    ///   hegel astq -l rust -p 'pub fn $FUNC' src/
    ///   hegel astq -l rust -p 'println!($X)' -r 'log::info!($X)' src/
    ///   hegel astq -l rust -p 'fn $NAME($$$) { $$$ }' --apply src/
    ///
    /// Common flags:
    ///   -l <lang>    Language (rust, go, js, py, etc.)
    ///   -p <pattern> Search pattern
    ///   -r <repl>    Replacement pattern
    ///   --apply      Apply changes (default: preview only)
    Astq {
        /// Arguments to pass to ast-grep
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch ephemeral Markdown review UI (wraps mirror)
    ///
    /// Workflow:
    ///   1. Mirror GUI launches
    ///   2. User selects text → adds comment → submits
    ///   3. Review saved to .ddd/<filename>.review.N (JSONL format)
    ///   4. Mirror exits (ephemeral, no persistent state)
    ///
    /// Read reviews with: cat .ddd/SPEC.review.1 | jq -r '.comment'
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
    ///
    /// Guardrails configuration: .hegel/guardrails.yaml
    ///   - Blocks commands matching regex patterns
    ///   - Exits with code 1 and prints reason when blocked
    ///   - If no guardrails.yaml exists, commands pass through
    ///
    /// All invocations logged to: .hegel/command_log.jsonl
    ///
    /// Example guardrails.yaml:
    ///   git:
    ///     blocked:
    ///       - pattern: "reset --hard"
    ///         reason: "Destructive: permanently discards uncommitted changes"
    ///       - pattern: "push.*--force"
    ///         reason: "Force push can overwrite remote history"
    Git {
        /// Arguments to pass to git
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run docker with guardrails and audit logging
    ///
    /// Guardrails configuration: .hegel/guardrails.yaml
    ///   - Blocks commands matching regex patterns
    ///   - Exits with code 1 and prints reason when blocked
    ///   - If no guardrails.yaml exists, commands pass through
    ///
    /// All invocations logged to: .hegel/command_log.jsonl
    ///
    /// Example guardrails.yaml:
    ///   docker:
    ///     blocked:
    ///       - pattern: "rm -f"
    ///         reason: "Force remove containers blocked"
    ///       - pattern: "system prune -a"
    ///         reason: "Destructive: removes all unused containers, networks, images"
    Docker {
        /// Arguments to pass to docker
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Declare or view current meta-mode
    Meta {
        /// Meta-mode name. Omit to view current meta-mode
        name: Option<String>,
        /// List available meta-modes
        #[arg(long)]
        list: bool,
    },
    /// Get, set, or list configuration values
    Config {
        /// Action: get, set, or list (default: list)
        action: Option<String>,
        /// Config key (for get/set)
        key: Option<String>,
        /// Config value (for set)
        value: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If no command provided, output HEGEL_CLAUDE.md for LLM onboarding
    if cli.command.is_none() {
        // Embed HEGEL_CLAUDE.md at compile time
        const HEGEL_GUIDE: &str = include_str!("../HEGEL_CLAUDE.md");
        println!("{}", HEGEL_GUIDE);
        return Ok(());
    }

    // Initialize storage with resolved state directory
    let state_dir = FileStorage::resolve_state_dir(cli.state_dir)?;
    let storage = FileStorage::new(state_dir)?;

    // Auto-install Claude Code hooks if needed (silently fails if not in Claude Code)
    let _ = commands::auto_install_hooks();

    // Execute command
    match cli.command.unwrap() {
        Commands::Init => {
            commands::init_project(&storage)?;
        }
        Commands::Start { workflow } => {
            commands::start_workflow(&workflow, &storage)?;
        }
        Commands::Workflows => {
            commands::list_workflows(&storage)?;
        }
        Commands::Guides => {
            commands::list_guides(&storage)?;
        }
        Commands::Next { claims } => {
            commands::next_prompt(claims.as_deref(), &storage)?;
        }
        Commands::Repeat => {
            commands::repeat_prompt(&storage)?;
        }
        Commands::Restart => {
            commands::restart_workflow(&storage)?;
        }
        Commands::Status => {
            commands::show_status(&storage)?;
        }
        Commands::Reset => {
            commands::reset_workflow(&storage)?;
        }
        Commands::Abort => {
            commands::abort_workflow(&storage)?;
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
        Commands::Meta { name, list } => {
            commands::meta_mode(name.as_deref(), list, &storage)?;
        }
        Commands::Config { action, key, value } => {
            commands::handle_config(
                action.as_deref(),
                key.as_deref(),
                value.as_deref(),
                &storage,
            )?;
        }
    }

    Ok(())
}
