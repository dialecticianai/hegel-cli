use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;

use crate::engine::{get_next_prompt, is_terminal};
use crate::metamodes::evaluate_workflow_completion;
use crate::metrics::cowboy::{build_synthetic_cowboy_archive, CowboyActivityGroup};
use crate::metrics::parse_unified_metrics;
use crate::storage::archive::{read_archives, write_archive, WorkflowArchive};
use crate::storage::log_cleanup::cleanup_logs;
use crate::storage::{FileStorage, State};
use crate::theme::Theme;

use super::context::{display_workflow_prompt, WorkflowContext};

/// Represents the outcome of a transition evaluation
#[derive(Debug, PartialEq)]
pub enum TransitionOutcome {
    /// Stay at current node (no transition matched)
    Stay {
        current_node: String,
        prompt: String,
    },
    /// Transition within the same workflow
    IntraWorkflow {
        from_node: String,
        to_node: String,
        prompt: String,
    },
    /// Transition to a different workflow (meta-mode)
    InterWorkflow {
        from_workflow: String,
        from_node: String,
        to_workflow: String,
        to_node: String,
        prompt: String,
    },
    /// Multiple transition options available (user must choose)
    Ambiguous { options: Vec<TransitionOption> },
}

/// A possible transition option when multiple are available
#[derive(Debug, PartialEq)]
pub struct TransitionOption {
    pub description: String,
    pub target_workflow: String,
    pub target_node: String,
}

/// Evaluate what type of transition should occur based on claims and context
pub fn evaluate_transition(
    context: &WorkflowContext,
    claims: &HashSet<String>,
    storage: &FileStorage,
) -> Result<TransitionOutcome> {
    let current_node = &context.workflow_state.current_node;
    let workflow_mode = &context.workflow_state.mode;

    // First, try intra-workflow transitions (existing behavior)
    let (prompt_text, new_state) = get_next_prompt(
        &context.workflow,
        &context.workflow_state,
        claims,
        storage.state_dir(),
    )?;

    // If we transitioned to a different node within the workflow
    if new_state.current_node != *current_node {
        return Ok(TransitionOutcome::IntraWorkflow {
            from_node: current_node.clone(),
            to_node: new_state.current_node.clone(),
            prompt: prompt_text,
        });
    }

    // If we're at a terminal node with meta-mode, check for workflow transitions
    if is_terminal(&new_state.current_node) {
        if let Some(meta_mode) = &context.workflow_state.meta_mode {
            let meta_mode_transitions = evaluate_workflow_completion(
                &meta_mode.name,
                workflow_mode,
                &new_state.current_node,
            );

            if let Some(transitions) = meta_mode_transitions {
                // Single transition option - auto-transition
                if transitions.len() == 1 {
                    let transition = &transitions[0];
                    let target_node = "spec"; // All workflows start at spec

                    return Ok(TransitionOutcome::InterWorkflow {
                        from_workflow: workflow_mode.clone(),
                        from_node: current_node.clone(),
                        to_workflow: transition.next_workflow.clone(),
                        to_node: target_node.to_string(),
                        prompt: transition.description.clone(),
                    });
                }

                // Multiple options - return ambiguous
                if transitions.len() > 1 {
                    let options = transitions
                        .iter()
                        .map(|t| TransitionOption {
                            description: t.description.clone(),
                            target_workflow: t.next_workflow.clone(),
                            target_node: "spec".to_string(),
                        })
                        .collect();

                    return Ok(TransitionOutcome::Ambiguous { options });
                }
            }
        }
    }

    // No transition matched - stay at current node
    Ok(TransitionOutcome::Stay {
        current_node: current_node.clone(),
        prompt: prompt_text,
    })
}

/// Detect cowboy activity between the most recent archived workflow and the given timestamp
pub(super) fn detect_and_archive_cowboy_activity(
    state_dir: &std::path::Path,
    current_timestamp: &str,
) -> Result<()> {
    use chrono::{DateTime, Utc};

    // Load all existing archives
    let all_archives = read_archives(state_dir)?;

    // Find the most recent real (non-synthetic) archive before the current timestamp
    let previous_archive = all_archives
        .iter()
        .filter(|a| a.workflow_id.as_str() < current_timestamp && !a.is_synthetic)
        .max_by_key(|a| a.workflow_id.as_str());

    // Determine the gap we're checking: from previous workflow's end to now
    let gap_start = if let Some(prev) = previous_archive {
        DateTime::parse_from_rfc3339(&prev.completed_at)
            .context("Failed to parse previous workflow completed_at")?
            .with_timezone(&Utc)
    } else {
        // No previous workflow - gap starts from beginning of time (earliest activity)
        DateTime::<Utc>::MIN_UTC
    };

    let gap_end = DateTime::parse_from_rfc3339(current_timestamp)
        .context("Failed to parse current timestamp")?
        .with_timezone(&Utc);

    // Parse current metrics from hooks/transcripts/git (these haven't been deleted yet)
    let hooks_path = state_dir.join("hooks.jsonl");
    let mut bash_commands = vec![];
    let mut file_modifications = vec![];
    let transcript_events: Vec<crate::metrics::TranscriptEvent> = vec![]; // TODO: parse transcript events if needed

    if hooks_path.exists() {
        use crate::metrics::parse_hooks_file;
        let hook_metrics = parse_hooks_file(&hooks_path)?;
        bash_commands = hook_metrics.bash_commands;
        file_modifications = hook_metrics.file_modifications;
    }

    // Parse git commits
    let mut git_commits = vec![];
    use crate::metrics::git;
    if git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();
        git_commits = git::parse_git_commits(project_root, None).unwrap_or_default();
    }

    // Helper to parse timestamp
    let parse_ts = |ts: &str| -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(ts)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    };

    // Filter activity to only what's in the gap
    let bash_in_gap: Vec<_> = bash_commands
        .into_iter()
        .filter(|cmd| {
            cmd.timestamp
                .as_ref()
                .and_then(|ts| parse_ts(ts))
                .map(|t| t > gap_start && t < gap_end)
                .unwrap_or(false)
        })
        .collect();

    let files_in_gap: Vec<_> = file_modifications
        .into_iter()
        .filter(|fm| {
            fm.timestamp
                .as_ref()
                .and_then(|ts| parse_ts(ts))
                .map(|t| t > gap_start && t < gap_end)
                .unwrap_or(false)
        })
        .collect();

    let commits_in_gap: Vec<_> = git_commits
        .into_iter()
        .filter(|commit| {
            parse_ts(&commit.timestamp)
                .map(|t| t > gap_start && t < gap_end)
                .unwrap_or(false)
        })
        .collect();

    let transcripts_in_gap: Vec<_> = transcript_events
        .into_iter()
        .filter(|event| {
            event
                .timestamp
                .as_ref()
                .and_then(|ts| parse_ts(ts))
                .map(|t| t > gap_start && t < gap_end)
                .unwrap_or(false)
        })
        .collect();

    // Check if there's ANY activity in the gap
    let has_activity = !bash_in_gap.is_empty()
        || !files_in_gap.is_empty()
        || !commits_in_gap.is_empty()
        || !transcripts_in_gap.is_empty();

    if has_activity {
        // Create ONE cowboy workflow for this gap
        let group = CowboyActivityGroup {
            start_time: gap_start,
            end_time: gap_end,
            bash_commands: bash_in_gap,
            file_modifications: files_in_gap,
            git_commits: commits_in_gap,
            transcript_events: transcripts_in_gap,
        };

        let synthetic_archive = build_synthetic_cowboy_archive(&group)?;

        // Check if archive already exists (avoid duplicates)
        let archive_path = state_dir
            .join("archive")
            .join(format!("{}.json", synthetic_archive.workflow_id));

        if !archive_path.exists() {
            write_archive(&synthetic_archive, state_dir)?;
            println!(
                "{} Detected cowboy activity ({})",
                Theme::success("  ✓"),
                synthetic_archive.workflow_id
            );
        }
    }

    // Delete logs after creating cowboy archives to prevent re-detection
    cleanup_logs(state_dir)?;

    Ok(())
}

/// Archive completed workflow and delete raw logs
pub(crate) fn archive_and_cleanup(storage: &FileStorage) -> Result<()> {
    let state_dir = storage.state_dir();

    // Parse current metrics WITHOUT archives AND git to prevent duplication bug
    let mut metrics = parse_unified_metrics(state_dir, false, None)?;

    // Get workflow_id from state
    let state = storage.load()?;
    let workflow_id = state
        .workflow_state
        .as_ref()
        .and_then(|ws| ws.workflow_id.clone())
        .context("No workflow_id for archiving")?;

    // Get completed_at timestamp (current time)
    let completed_at = chrono::Utc::now().to_rfc3339();

    // Parse git commits and attribute to phases ONLY during archiving
    use crate::metrics::git;
    if git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();

        // Use first state transition timestamp as session start (if available)
        let since_timestamp = metrics
            .state_transitions
            .first()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.timestamp).ok())
            .map(|dt| dt.timestamp());

        let git_commits =
            git::parse_git_commits(project_root, since_timestamp).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse git commits: {}", e);
                Vec::new()
            });

        // Attribute commits to phases
        git::attribute_commits_to_phases(git_commits.clone(), &mut metrics.phase_metrics);
        metrics.git_commits = git_commits;
    }

    // Create archive from metrics (explicit workflow, not synthetic)
    let mut archive = WorkflowArchive::from_metrics(&metrics, &workflow_id, false)?;
    archive.completed_at = completed_at;

    // Write archive
    write_archive(&archive, state_dir)?;

    // Update cumulative totals in state
    let mut updated_state = state.clone();
    let cumulative = updated_state
        .cumulative_totals
        .get_or_insert_with(Default::default);

    // Accumulate tokens
    cumulative.tokens.input += archive.totals.tokens.input;
    cumulative.tokens.output += archive.totals.tokens.output;
    cumulative.tokens.cache_creation += archive.totals.tokens.cache_creation;
    cumulative.tokens.cache_read += archive.totals.tokens.cache_read;
    cumulative.tokens.assistant_turns += archive.totals.tokens.assistant_turns;

    // Accumulate activity
    cumulative.bash_commands += archive.totals.bash_commands;
    cumulative.file_modifications += archive.totals.file_modifications;
    cumulative.unique_files += archive.totals.unique_files;
    cumulative.unique_commands += archive.totals.unique_commands;
    cumulative.git_commits += archive.totals.git_commits;

    // Save updated state
    storage.save(&updated_state)?;

    // Delete logs on success
    cleanup_logs(state_dir)?;

    // Display success with archive totals
    println!(
        "{}",
        Theme::success("✓ Workflow archived and logs cleaned up")
    );
    println!(
        "  Phases: {}  |  Tokens: {} in, {} out ({} cache hits)",
        archive.phases.len(),
        format_token_count(archive.totals.tokens.input),
        format_token_count(archive.totals.tokens.output),
        format_token_count(archive.totals.tokens.cache_read)
    );
    println!(
        "  Activity: {} bash, {} files, {} commits",
        archive.totals.bash_commands, archive.totals.file_modifications, archive.totals.git_commits
    );

    Ok(())
}

/// Format token count with K suffix for thousands
fn format_token_count(count: u64) -> String {
    if count >= 1000 {
        format!("{:.1}K", count as f64 / 1000.0)
    } else {
        count.to_string()
    }
}

/// Execute a transition outcome, performing all necessary side effects
pub fn execute_transition(
    outcome: TransitionOutcome,
    context: &mut WorkflowContext,
    storage: &FileStorage,
) -> Result<()> {
    use crate::engine::{init_state, load_workflow};
    use chrono::Utc;

    match outcome {
        TransitionOutcome::Stay {
            current_node,
            prompt,
        } => {
            // No state change, just display
            println!("{}", Theme::warning("Stayed at current node"));
            println!();
            display_workflow_prompt(
                &current_node,
                &context.workflow_state.mode,
                &prompt,
                context.workflow_state.is_handlebars,
                storage,
            )?;
        }

        TransitionOutcome::IntraWorkflow {
            from_node,
            to_node,
            prompt,
        } => {
            // Update workflow state
            context.workflow_state.current_node = to_node.clone();
            context.workflow_state.history.push(to_node.clone());
            context.workflow_state.phase_start_time = Some(chrono::Utc::now().to_rfc3339());

            // Persist state
            let state = State {
                workflow: Some(serde_yaml::to_value(&context.workflow)?),
                workflow_state: Some(context.workflow_state.clone()),
                session_metadata: context.session_metadata.clone(),
                cumulative_totals: storage.load().ok().and_then(|s| s.cumulative_totals),
                git_info: storage.load().ok().and_then(|s| s.git_info),
            };
            storage.save(&state)?;

            // Log transition
            storage.log_state_transition(
                &from_node,
                &to_node,
                &context.workflow_state.mode,
                context.workflow_state.workflow_id.as_deref(),
            )?;

            // Archive workflow if transitioning to a terminal node
            if is_terminal(&to_node) {
                if let Err(e) = archive_and_cleanup(storage) {
                    eprintln!("{} {}", Theme::error("Warning: archiving failed:"), e);
                    eprintln!("Workflow logs preserved for manual inspection.");
                }
            }

            // Display transition
            println!(
                "{} {} {} {}",
                Theme::success("Transitioned:").bold(),
                Theme::secondary(&from_node),
                Theme::secondary("→"),
                Theme::highlight(&to_node)
            );
            println!();
            display_workflow_prompt(
                &to_node,
                &context.workflow_state.mode,
                &prompt,
                context.workflow_state.is_handlebars,
                storage,
            )?;
        }

        TransitionOutcome::InterWorkflow {
            from_workflow,
            from_node,
            to_workflow,
            to_node,
            prompt: _,
        } => {
            // Log completion of old workflow
            println!(
                "{} {} workflow completed",
                Theme::success("✓").bold(),
                Theme::highlight(&from_workflow)
            );
            println!();

            // Load new workflow
            let workflows_dir = storage.workflows_dir();
            let workflow_path = format!("{}/{}.yaml", workflows_dir, to_workflow);
            let new_workflow = load_workflow(&workflow_path)
                .with_context(|| format!("Failed to load workflow: {}", to_workflow))?;

            // Initialize new workflow state
            let mut new_state = init_state(&new_workflow);
            new_state.workflow_id = Some(Utc::now().to_rfc3339());
            new_state.meta_mode = context.workflow_state.meta_mode.clone(); // Preserve meta-mode

            // Persist new state
            let state = State {
                workflow: Some(serde_yaml::to_value(&new_workflow)?),
                workflow_state: Some(new_state.clone()),
                session_metadata: context.session_metadata.clone(),
                cumulative_totals: storage.load().ok().and_then(|s| s.cumulative_totals),
                git_info: storage.load().ok().and_then(|s| s.git_info),
            };
            storage.save(&state)?;

            // Log new workflow start
            storage.log_state_transition(
                &from_node,
                &to_node,
                &to_workflow,
                new_state.workflow_id.as_deref(),
            )?;

            // Update context
            context.workflow = new_workflow;
            context.workflow_state = new_state;

            // Display transition
            println!(
                "{} {} {} {}",
                Theme::success("Transitioned:").bold(),
                Theme::secondary(&format!("{}:{}", from_workflow, from_node)),
                Theme::secondary("→"),
                Theme::highlight(&format!("{}:{}", to_workflow, to_node))
            );
            println!();

            // Get and display the actual node prompt from the new workflow
            let node = context
                .workflow
                .nodes
                .get(&to_node)
                .with_context(|| format!("Node not found: {}", to_node))?;

            // Select prompt based on which field is present
            let prompt_text = if !node.prompt_hbs.is_empty() {
                &node.prompt_hbs
            } else {
                &node.prompt
            };

            display_workflow_prompt(
                &to_node,
                &to_workflow,
                prompt_text,
                context.workflow_state.is_handlebars,
                storage,
            )?;
        }

        TransitionOutcome::Ambiguous { options } => {
            // Display options to user
            println!(
                "{}",
                Theme::warning("Multiple transition options available:")
            );
            println!();

            for (i, option) in options.iter().enumerate() {
                println!(
                    "  {}. {} → {}",
                    i + 1,
                    Theme::highlight(&option.target_workflow),
                    option.description
                );
            }

            println!();
            println!(
                "Use {} to select:",
                Theme::highlight("hegel start <workflow>")
            );

            for option in &options {
                println!("  - hegel start {}", option.target_workflow);
            }
        }
    }

    Ok(())
}
