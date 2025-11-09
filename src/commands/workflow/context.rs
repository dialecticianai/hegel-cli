use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::HegelConfig;
use crate::engine::{render_prompt, Workflow};
use crate::storage::{FileStorage, SessionMetadata, WorkflowState};
use crate::theme::Theme;

/// Context bundle for workflow operations
pub struct WorkflowContext {
    pub workflow: Workflow,
    pub workflow_state: WorkflowState,
    pub session_metadata: Option<SessionMetadata>,
}

/// Load complete workflow context from storage
pub fn load_workflow_context(storage: &FileStorage) -> Result<WorkflowContext> {
    let state = storage.load()?;

    let workflow_state = state
        .workflow
        .as_ref()
        .context("No workflow loaded. Run 'hegel start <workflow>' first.")?
        .clone();

    // Load workflow from YAML file based on mode
    let workflow_path =
        PathBuf::from(storage.workflows_dir()).join(format!("{}.yaml", workflow_state.mode));
    let workflow = crate::engine::load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_state.mode))?;

    Ok(WorkflowContext {
        workflow,
        workflow_state,
        session_metadata: state.session_metadata,
    })
}

/// Render a workflow node's prompt with guide templates
pub fn render_node_prompt(
    prompt: &str,
    is_handlebars: bool,
    storage: &FileStorage,
) -> Result<String> {
    let guides_dir_str = storage.guides_dir();
    let guides_dir = Path::new(&guides_dir_str);

    // Load config and inject values as template context
    let config = HegelConfig::load(storage.state_dir())?;
    let mut context = HashMap::new();

    // Inject all config values as context variables
    for (key, value) in config.list() {
        context.insert(key, value);
    }

    render_prompt(prompt, is_handlebars, guides_dir, &context)
        .with_context(|| "Failed to render prompt template")
}

/// Display workflow prompt with consistent formatting
pub fn display_workflow_prompt(
    current_node: &str,
    mode: &str,
    prompt: &str,
    is_handlebars: bool,
    storage: &FileStorage,
) -> Result<()> {
    let rendered_prompt = render_node_prompt(prompt, is_handlebars, storage)?;

    println!("{}: {}", Theme::label("Mode"), mode);
    println!("{}: {}", Theme::label("Current node"), current_node);
    println!();
    println!("{}", Theme::header("Prompt:"));
    println!("{}", rendered_prompt);

    Ok(())
}
