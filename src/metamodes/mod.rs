use anyhow::{bail, Result};

/// Meta-mode transition option
#[derive(Debug, Clone)]
pub struct MetaModeTransition {
    pub description: String,
    pub next_workflow: String,
    pub meta_mode_change: Option<String>, // Some("standard") if changing meta-mode
}

/// Meta-mode definitions and transition logic
#[derive(Debug)]
pub struct MetaModeDefinition {
    pub name: String,
    pub description: String,
    pub initial_workflow: String,
}

impl MetaModeDefinition {
    /// Get all available meta-mode definitions
    pub fn all() -> Vec<MetaModeDefinition> {
        vec![
            MetaModeDefinition {
                name: "learning".to_string(),
                description: "Greenfield learning project (Research ↔ Discovery loop)".to_string(),
                initial_workflow: "research".to_string(),
            },
            MetaModeDefinition {
                name: "standard".to_string(),
                description: "Feature development with known patterns (Discovery ↔ Execution)"
                    .to_string(),
                initial_workflow: "discovery".to_string(),
            },
        ]
    }

    /// Get meta-mode definition by name
    pub fn get(name: &str) -> Result<MetaModeDefinition> {
        Self::all()
            .into_iter()
            .find(|m| m.name == name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Invalid meta-mode: {}. Valid options: learning, standard",
                    name
                )
            })
    }
}

/// Evaluate available transitions when a workflow completes
pub fn evaluate_workflow_completion(
    meta_mode_name: &str,
    current_workflow: &str,
    current_node: &str,
) -> Option<Vec<MetaModeTransition>> {
    // Only evaluate at workflow 'done' nodes
    if current_node != "done" {
        return None;
    }

    match meta_mode_name {
        "learning" => match current_workflow {
            "research" => Some(vec![MetaModeTransition {
                description: "Validate questions through toy experiments".to_string(),
                next_workflow: "discovery".to_string(),
                meta_mode_change: None,
            }]),
            "discovery" => Some(vec![
                MetaModeTransition {
                    description: "Loop back to integrate findings into learning docs".to_string(),
                    next_workflow: "research".to_string(),
                    meta_mode_change: None,
                },
                MetaModeTransition {
                    description: "Done learning, begin production delivery".to_string(),
                    next_workflow: "execution".to_string(),
                    meta_mode_change: Some("standard".to_string()),
                },
            ]),
            _ => None,
        },
        "standard" => match current_workflow {
            "discovery" => Some(vec![MetaModeTransition {
                description: "Prototype validated, build production version".to_string(),
                next_workflow: "execution".to_string(),
                meta_mode_change: None,
            }]),
            "execution" => Some(vec![MetaModeTransition {
                description: "Need to validate new approach".to_string(),
                next_workflow: "discovery".to_string(),
                meta_mode_change: None,
            }]),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_meta_modes_returns_two() {
        let modes = MetaModeDefinition::all();
        assert_eq!(modes.len(), 2);
        assert_eq!(modes[0].name, "learning");
        assert_eq!(modes[0].initial_workflow, "research");
        assert_eq!(modes[1].name, "standard");
        assert_eq!(modes[1].initial_workflow, "discovery");
    }

    #[test]
    fn test_get_learning_meta_mode() {
        let mode = MetaModeDefinition::get("learning").unwrap();
        assert_eq!(mode.name, "learning");
        assert_eq!(mode.initial_workflow, "research");
    }

    #[test]
    fn test_get_standard_meta_mode() {
        let mode = MetaModeDefinition::get("standard").unwrap();
        assert_eq!(mode.name, "standard");
        assert_eq!(mode.initial_workflow, "discovery");
    }

    #[test]
    fn test_get_invalid_meta_mode() {
        let result = MetaModeDefinition::get("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid meta-mode"));
    }

    #[test]
    fn test_learning_research_to_discovery() {
        let transitions = evaluate_workflow_completion("learning", "research", "done");
        assert!(transitions.is_some());
        let transitions = transitions.unwrap();
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].next_workflow, "discovery");
        assert!(transitions[0].meta_mode_change.is_none());
    }

    #[test]
    fn test_learning_discovery_has_two_options() {
        let transitions = evaluate_workflow_completion("learning", "discovery", "done");
        assert!(transitions.is_some());
        let transitions = transitions.unwrap();
        assert_eq!(transitions.len(), 2);

        // First option: loop back to research
        assert_eq!(transitions[0].next_workflow, "research");
        assert!(transitions[0].meta_mode_change.is_none());

        // Second option: transition to standard + execution
        assert_eq!(transitions[1].next_workflow, "execution");
        assert_eq!(
            transitions[1].meta_mode_change,
            Some("standard".to_string())
        );
    }

    #[test]
    fn test_standard_discovery_to_execution() {
        let transitions = evaluate_workflow_completion("standard", "discovery", "done");
        assert!(transitions.is_some());
        let transitions = transitions.unwrap();
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].next_workflow, "execution");
        assert!(transitions[0].meta_mode_change.is_none());
    }

    #[test]
    fn test_standard_execution_to_discovery() {
        let transitions = evaluate_workflow_completion("standard", "execution", "done");
        assert!(transitions.is_some());
        let transitions = transitions.unwrap();
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].next_workflow, "discovery");
        assert!(transitions[0].meta_mode_change.is_none());
    }

    #[test]
    fn test_non_done_node_returns_none() {
        let transitions = evaluate_workflow_completion("learning", "research", "plan");
        assert!(transitions.is_none());
    }

    #[test]
    fn test_invalid_workflow_returns_none() {
        let transitions = evaluate_workflow_completion("learning", "unknown", "done");
        assert!(transitions.is_none());
    }

    #[test]
    fn test_invalid_meta_mode_returns_none() {
        let transitions = evaluate_workflow_completion("invalid", "discovery", "done");
        assert!(transitions.is_none());
    }
}
