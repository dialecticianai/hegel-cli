// Embedded resources (workflows, guides) bundled at compile time
// This allows hegel to work from any directory without requiring local workflow files

use std::collections::HashMap;

/// Get embedded workflow content by name
pub fn get_workflow(name: &str) -> Option<&'static str> {
    let workflows: HashMap<&str, &str> = HashMap::from([
        ("discovery", include_str!("../workflows/discovery.yaml")),
        ("execution", include_str!("../workflows/execution.yaml")),
        ("research", include_str!("../workflows/research.yaml")),
        ("minimal", include_str!("../workflows/minimal.yaml")),
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
        // Templates
        (
            "templates/mirror_workflow.md",
            include_str!("../guides/templates/mirror_workflow.md"),
        ),
    ]);
    guides.get(name).copied()
}

/// List available embedded workflows
pub fn list_workflows() -> Vec<&'static str> {
    vec!["discovery", "execution", "research", "minimal"]
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
    ]
}
