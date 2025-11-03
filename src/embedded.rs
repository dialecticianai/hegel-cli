// Embedded resources (workflows, guides) bundled at compile time
// This allows hegel to work from any directory without requiring local workflow files

use std::collections::HashMap;

/// Get embedded workflow content by name
pub fn get_workflow(name: &str) -> Option<&'static str> {
    let workflows: HashMap<&str, &str> = HashMap::from([
        ("discovery", include_str!("../workflows/discovery.yaml")),
        ("execution", include_str!("../workflows/execution.yaml")),
        ("research", include_str!("../workflows/research.yaml")),
        ("refactor", include_str!("../workflows/refactor.yaml")),
        ("minimal", include_str!("../workflows/minimal.yaml")),
        (
            "init-greenfield",
            include_str!("../workflows/init-greenfield.yaml"),
        ),
        (
            "init-retrofit",
            include_str!("../workflows/init-retrofit.yaml"),
        ),
    ]);
    workflows.get(name).copied()
}

/// Get embedded guide content by name
pub fn get_guide(name: &str) -> Option<&'static str> {
    let guides: HashMap<&str, &str> = HashMap::from([
        ("SPEC_WRITING.md", include_str!("../guides/SPEC_WRITING.md")),
        ("PLAN_WRITING.md", include_str!("../guides/PLAN_WRITING.md")),
        (
            "LEARNINGS_WRITING.md",
            include_str!("../guides/LEARNINGS_WRITING.md"),
        ),
        (
            "README_WRITING.md",
            include_str!("../guides/README_WRITING.md"),
        ),
        (
            "CODE_MAP_WRITING.md",
            include_str!("../guides/CODE_MAP_WRITING.md"),
        ),
        (
            "KICKOFF_WRITING.md",
            include_str!("../guides/KICKOFF_WRITING.md"),
        ),
        (
            "HANDOFF_WRITING.md",
            include_str!("../guides/HANDOFF_WRITING.md"),
        ),
        (
            "STUDY_PLANNING.md",
            include_str!("../guides/STUDY_PLANNING.md"),
        ),
        (
            "KNOWLEDGE_CAPTURE.md",
            include_str!("../guides/KNOWLEDGE_CAPTURE.md"),
        ),
        (
            "QUESTION_TRACKING.md",
            include_str!("../guides/QUESTION_TRACKING.md"),
        ),
        (
            "CLAUDE_CUSTOMIZATION.md",
            include_str!("../guides/CLAUDE_CUSTOMIZATION.md"),
        ),
        (
            "VISION_WRITING.md",
            include_str!("../guides/VISION_WRITING.md"),
        ),
        (
            "ARCHITECTURE_WRITING.md",
            include_str!("../guides/ARCHITECTURE_WRITING.md"),
        ),
        // Templates
        (
            "templates/mirror_workflow.md",
            include_str!("../guides/templates/mirror_workflow.md"),
        ),
        (
            "templates/code_map_monolithic.md",
            include_str!("../guides/templates/code_map_monolithic.md"),
        ),
        (
            "templates/code_map_hierarchical.md",
            include_str!("../guides/templates/code_map_hierarchical.md"),
        ),
    ]);
    guides.get(name).copied()
}

/// List available embedded workflows
pub fn list_workflows() -> Vec<&'static str> {
    vec![
        "discovery",
        "execution",
        "research",
        "refactor",
        "minimal",
        "init-greenfield",
        "init-retrofit",
    ]
}

/// List available embedded guides
pub fn list_guides() -> Vec<&'static str> {
    vec![
        "SPEC_WRITING.md",
        "PLAN_WRITING.md",
        "LEARNINGS_WRITING.md",
        "README_WRITING.md",
        "CODE_MAP_WRITING.md",
        "KICKOFF_WRITING.md",
        "HANDOFF_WRITING.md",
        "STUDY_PLANNING.md",
        "KNOWLEDGE_CAPTURE.md",
        "QUESTION_TRACKING.md",
        "CLAUDE_CUSTOMIZATION.md",
        "VISION_WRITING.md",
        "ARCHITECTURE_WRITING.md",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_embedded_workflows_are_valid() {
        // Test that all embedded workflows can be parsed and validated
        for workflow_name in list_workflows() {
            let yaml_content = get_workflow(workflow_name)
                .unwrap_or_else(|| panic!("Workflow '{}' not found", workflow_name));

            let workflow: crate::engine::Workflow = serde_yaml::from_str(yaml_content)
                .unwrap_or_else(|e| panic!("Failed to parse workflow '{}': {}", workflow_name, e));

            workflow.validate().unwrap_or_else(|e| {
                panic!("Workflow '{}' failed validation: {}", workflow_name, e)
            });
        }
    }
}
