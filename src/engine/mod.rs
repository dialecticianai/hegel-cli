mod handlebars;
mod template;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::storage::WorkflowState;
pub use handlebars::render_template_hbs;
pub use template::render_template;

/// Render template using appropriate engine based on is_handlebars flag
///
/// Routes to either the Markdown template engine (template::render_template)
/// or the Handlebars engine (handlebars::render_template_hbs) based on the
/// is_handlebars parameter.
///
/// # Arguments
/// * `template_content` - The template string to render
/// * `is_handlebars` - If true, use Handlebars engine; if false, use Markdown engine
/// * `guides_dir` - Directory containing guide files and partials
/// * `context` - Context variables for template rendering
///
/// # Returns
/// Rendered template string or error
pub fn render_prompt(
    template_content: &str,
    is_handlebars: bool,
    guides_dir: &Path,
    context: &HashMap<String, String>,
) -> Result<String> {
    if is_handlebars {
        // Wrap context for Handlebars (enables future extensibility: config, etc.)
        let hbs_context = handlebars::HandlebarsContext {
            context: context.clone(),
        };
        render_template_hbs(template_content, guides_dir, &hbs_context)
    } else {
        render_template(template_content, guides_dir, context)
    }
}

/// Check if a node is a terminal node (workflow completion state)
///
/// Terminal nodes are "done" (successful completion) or "aborted" (workflow termination).
/// These nodes have no outgoing transitions and represent the end of a workflow.
///
/// # Arguments
/// * `node` - The node name to check
///
/// # Returns
/// `true` if the node is terminal, `false` otherwise
pub fn is_terminal(node: &str) -> bool {
    node == "done" || node == "aborted"
}

/// Workflow transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub when: String,
    pub to: String,
}

/// Workflow node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub prompt_hbs: String,
    #[serde(default)]
    pub summary: String,
    pub transitions: Vec<Transition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<crate::rules::RuleConfig>,
}

/// Complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub mode: String,
    pub start_node: String,
    pub nodes: HashMap<String, Node>,
}

impl Workflow {
    /// Validate all rules in all nodes
    pub fn validate(&self) -> Result<()> {
        for (node_name, node) in &self.nodes {
            // Validate that nodes don't have both prompt and prompt_hbs
            if !node.prompt.is_empty() && !node.prompt_hbs.is_empty() {
                anyhow::bail!(
                    "Workflow validation failed: Node '{}' cannot have both 'prompt' and 'prompt_hbs' fields",
                    node_name
                );
            }

            // Validate that 'done' nodes don't have prompts
            if node_name == "done" && (!node.prompt.is_empty() || !node.prompt_hbs.is_empty()) {
                anyhow::bail!(
                    "Workflow validation failed: 'done' node must not have a prompt or prompt_hbs field"
                );
            }

            for rule in &node.rules {
                rule.validate()
                    .with_context(|| format!("Invalid rule in node '{}'", node_name))?;
            }
        }
        Ok(())
    }

    /// Check if a node is terminal (has no outgoing transitions)
    pub fn is_terminal_node(&self, node_name: &str) -> bool {
        self.nodes
            .get(node_name)
            .map(|node| node.transitions.is_empty())
            .unwrap_or(false)
    }
}

/// Load workflow definition from YAML string
pub fn load_workflow_from_str(content: &str) -> Result<Workflow> {
    let mut workflow: Workflow =
        serde_yaml::from_str(content).with_context(|| "Failed to parse workflow YAML")?;

    // Reject workflows with explicit "done" nodes - these are now implicit
    if workflow.nodes.contains_key("done") {
        anyhow::bail!(
            "Workflow validation failed: 'done' nodes are implicit and should not be defined in YAML. \
             Remove the 'done' node from your workflow definition - it will be auto-injected."
        );
    }

    // Auto-inject implicit "done" terminal node
    workflow.nodes.insert(
        "done".to_string(),
        Node {
            prompt: String::new(),
            prompt_hbs: String::new(),
            summary: String::new(),
            transitions: vec![],
            rules: vec![],
        },
    );

    workflow.validate()?;
    Ok(workflow)
}

/// Load workflow definition from YAML file (tries filesystem first, then embedded fallback)
/// This allows users to override embedded workflows with local versions
pub fn load_workflow<P: AsRef<Path>>(yaml_path: P) -> Result<Workflow> {
    let path = yaml_path.as_ref();

    // Extract workflow name from path (e.g., "workflows/discovery.yaml" -> "discovery")
    let workflow_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    // Try filesystem first (allows user overrides)
    if let Ok(content) = fs::read_to_string(path) {
        return load_workflow_from_str(&content);
    }

    // Fall back to embedded workflow
    if let Some(embedded_content) = crate::embedded::get_workflow(workflow_name) {
        return load_workflow_from_str(embedded_content);
    }

    anyhow::bail!("Workflow not found: {}", workflow_name)
}

/// Initialize workflow state from workflow definition
pub fn init_state(workflow: &Workflow) -> WorkflowState {
    let start = workflow.start_node.clone();

    // Determine if start node uses Handlebars
    let is_handlebars = workflow
        .nodes
        .get(&start)
        .map(|node| !node.prompt_hbs.is_empty())
        .unwrap_or(false);

    WorkflowState {
        current_node: start.clone(),
        mode: workflow.mode.clone(),
        history: vec![start],
        workflow_id: None,
        meta_mode: None, // Will be set by caller if needed
        phase_start_time: Some(chrono::Utc::now().to_rfc3339()),
        is_handlebars,
    }
}

/// Get next prompt based on current state and claims
pub fn get_next_prompt(
    workflow: &Workflow,
    state: &WorkflowState,
    claims: &HashSet<String>,
    state_dir: &Path,
    force_bypass: Option<&Option<String>>,
) -> Result<(String, WorkflowState)> {
    let current = &state.current_node;
    let node = workflow
        .nodes
        .get(current)
        .with_context(|| format!("Node not found in workflow: {}", current))?;

    // Special handling for restart_cycle - always returns to start_node
    let next_node = if claims.contains("restart_cycle") {
        workflow.start_node.clone()
    } else {
        // Evaluate transitions - find first matching claim
        let mut next = current.clone();
        for transition in &node.transitions {
            if claims.contains(&transition.when) {
                next = transition.to.clone();
                break;
            }
        }
        next
    };

    // Build new state
    let mut new_state = state.clone();
    if next_node != *current {
        new_state.current_node = next_node.clone();
        new_state.history.push(next_node.clone());
    }

    // Get prompt for resulting node
    let next_node_obj = workflow
        .nodes
        .get(&new_state.current_node)
        .with_context(|| {
            format!(
                "Next node not found in workflow: {}",
                new_state.current_node
            )
        })?;

    // Set is_handlebars flag based on which prompt field is present
    new_state.is_handlebars = !next_node_obj.prompt_hbs.is_empty();

    // Evaluate rules for resulting node (if any)
    let prompt = if !next_node_obj.rules.is_empty() {
        use crate::config::HegelConfig;
        use crate::metrics::parse_unified_metrics;
        use crate::rules::{
            evaluate_rules, generate_interrupt_prompt, RuleConfig, RuleEvaluationContext,
        };
        use crate::storage::FileStorage;

        let metrics = parse_unified_metrics(state_dir, false, None)?;

        // Load config for rule behavior
        let config = HegelConfig::load(state_dir)?;

        // Load state to get git_info
        let storage = FileStorage::new(state_dir)?;
        let full_state = storage.load()?;
        let git_info = full_state.git_info.as_ref();

        // Filter rules based on force_bypass
        let rules_to_evaluate: Vec<RuleConfig> = match force_bypass {
            Some(None) => {
                // --force with no argument: skip all rules
                vec![]
            }
            Some(Some(rule_type)) => {
                // --force <type>: filter out matching rule type
                next_node_obj
                    .rules
                    .iter()
                    .filter(|rule| {
                        let type_name = match rule {
                            RuleConfig::RepeatedCommand { .. } => "repeated_command",
                            RuleConfig::RepeatedFileEdit { .. } => "repeated_file_edit",
                            RuleConfig::PhaseTimeout { .. } => "phase_timeout",
                            RuleConfig::TokenBudget { .. } => "token_budget",
                            RuleConfig::RequireCommits { .. } => "require_commits",
                        };
                        type_name != rule_type.as_str()
                    })
                    .cloned()
                    .collect()
            }
            None => {
                // No force: evaluate all rules
                next_node_obj.rules.clone()
            }
        };

        let context = RuleEvaluationContext {
            current_phase: &new_state.current_node,
            phase_start_time: new_state.phase_start_time.as_ref(),
            all_phase_metrics: &metrics.phase_metrics,
            hook_metrics: &metrics.hook_metrics,
            config: &config,
            git_info,
        };

        if let Some(violation) = evaluate_rules(&rules_to_evaluate, &context)? {
            // Interrupt REPLACES normal prompt
            generate_interrupt_prompt(&violation)
        } else {
            // Select prompt based on which field is present
            if !next_node_obj.prompt_hbs.is_empty() {
                next_node_obj.prompt_hbs.clone()
            } else {
                next_node_obj.prompt.clone()
            }
        }
    } else {
        // Select prompt based on which field is present
        if !next_node_obj.prompt_hbs.is_empty() {
            next_node_obj.prompt_hbs.clone()
        } else {
            next_node_obj.prompt.clone()
        }
    };

    Ok((prompt, new_state))
}

#[cfg(test)]
mod tests {
    mod handlebars;
    mod integration;
    mod navigation;
    mod rules;
    mod template;
    mod workflow;
}
