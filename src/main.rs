mod adapters;
mod analyze;
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
#[command(version)]
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
    Init {
        /// Optional project type override (greenfield or retrofit)
        #[arg(value_parser = ["greenfield", "retrofit"])]
        project_type: Option<String>,
    },
    /// Start a new workflow
    Start {
        /// Workflow name (e.g., discovery, execution)
        workflow: String,
        /// Optional starting node (defaults to workflow's start_node)
        start_node: Option<String>,
    },
    /// List available workflows
    Workflows,
    /// List available guides
    Guides {
        /// Show embedded guide content
        #[arg(short = 's', long)]
        show_embedded: Option<String>,
    },
    /// Advance to next phase (implicit: current_complete, or provide custom claim)
    Next {
        /// Optional claim name (e.g., 'spec_complete', 'needs_refactor')
        /// If omitted, uses happy-path claim: {current}_complete
        claim: Option<String>,
    },
    /// Go back to previous phase
    #[command(visible_alias = "previous")]
    Prev,
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
    Analyze {
        /// Export workflow graph as DOT format (for Graphviz visualization)
        #[arg(long)]
        export_dot: bool,
        /// Repair archives: backfill git metrics and create synthetic cowboy workflows for inter-workflow activity
        #[arg(long)]
        fix_archives: bool,
        /// Show what would be fixed without making changes (requires --fix-archives)
        #[arg(long, requires = "fix_archives")]
        dry_run: bool,
        /// Output repair results as JSON (implies --dry-run, requires --fix-archives)
        #[arg(long, requires = "fix_archives")]
        json: bool,
        /// Display brief cross-section summary
        #[arg(long)]
        brief: bool,
        /// Display activity section (session, tokens, activity)
        #[arg(long)]
        activity: bool,
        /// Display workflow transitions section
        #[arg(long)]
        workflow_transitions: bool,
        /// Display phase breakdown section
        #[arg(long)]
        phase_breakdown: bool,
        /// Display workflow graph section
        #[arg(long)]
        workflow_graph: bool,
    },
    /// Archive workflow logs and metrics
    Archive(commands::archive::ArchiveArgs),
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
    /// Launch project manager dashboard
    ///
    /// Provides web UI for managing multiple Hegel projects:
    ///   - Auto-discover projects with .hegel/ directories
    ///   - View workflow state and metrics across projects
    ///   - Real-time updates and statistics
    ///
    /// Launch modes:
    ///   hegel pm            Start dashboard (auto-opens browser)
    ///   hegel pm discover   Run discovery and print projects
    ///
    /// All arguments are passed through to hegel-pm binary.
    Pm {
        /// Arguments to pass to hegel-pm
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
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
    /// Delegate task to external agent CLI
    ///
    /// Without arguments, lists available agents and their compatibility status.
    ///
    /// With --agent flag, delegates the task to the specified agent CLI.
    ///
    /// Examples:
    ///   hegel fork                                    # List available agents
    ///   hegel fork --agent=codex "Write hello world"
    ///   hegel fork --agent=gemini -- -o json "Explain this code"
    ///   hegel fork --agent=codex -- --full-auto "Implement feature X"
    Fork {
        /// Agent to use (e.g., codex, gemini, aider)
        #[arg(long)]
        agent: Option<String>,

        /// Prompt for the agent (if not using positional args)
        prompt: Option<String>,

        /// Additional arguments to pass to the agent (use -- separator)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
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
    // Special case: for `init` command, use cwd/.hegel if no .hegel found
    let state_dir = if matches!(cli.command, Some(Commands::Init { .. })) {
        match cli.state_dir {
            Some(path) => path,
            None => std::env::current_dir()?.join(".hegel"),
        }
    } else {
        FileStorage::resolve_state_dir(cli.state_dir)?
    };
    let storage = FileStorage::new(state_dir)?;

    // Auto-install Claude Code hooks if needed (silently fails if not in Claude Code)
    let _ = commands::auto_install_hooks();

    // Execute command
    match cli.command.unwrap() {
        Commands::Init { project_type } => {
            commands::init_project(&storage, project_type.as_deref())?;
        }
        Commands::Start {
            workflow,
            start_node,
        } => {
            commands::start_workflow(&workflow, start_node.as_deref(), &storage)?;
        }
        Commands::Workflows => {
            commands::list_workflows(&storage)?;
        }
        Commands::Guides { show_embedded } => {
            commands::list_guides(show_embedded.as_deref(), &storage)?;
        }
        Commands::Next { claim } => {
            commands::next_prompt(claim.as_deref(), &storage)?;
        }
        Commands::Prev => {
            commands::prev_prompt(&storage)?;
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
        Commands::Analyze {
            export_dot,
            fix_archives,
            dry_run,
            json,
            brief,
            activity,
            workflow_transitions,
            phase_breakdown,
            workflow_graph,
        } => {
            let options = commands::AnalyzeOptions {
                export_dot,
                fix_archives,
                dry_run,
                json,
                brief,
                activity,
                workflow_transitions,
                phase_breakdown,
                workflow_graph,
            };
            commands::analyze_metrics(&storage, options)?;
        }
        Commands::Archive(args) => {
            commands::archive(args, &storage)?;
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
        Commands::Pm { args } => {
            commands::run_pm(&args)?;
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
        Commands::Fork {
            agent,
            prompt,
            args,
        } => {
            commands::handle_fork(agent.as_deref(), prompt.as_deref(), &args)?;
        }
    }

    Ok(())
}
