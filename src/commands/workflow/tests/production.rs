use super::*;

// ========== Production Workflow Tests ==========

#[test]
#[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
fn test_discovery_workflow_loads_with_rules() {
    use crate::engine::load_workflow;

    let temp_dir = setup_production_workflows();
    let workflow_path = temp_dir.path().join("workflows/discovery.yaml");
    let workflow = load_workflow(&workflow_path).unwrap();
    assert_eq!(workflow.mode, "discovery");

    // Verify code node has rules
    let code_node = &workflow.nodes["code"];
    assert_eq!(code_node.rules.len(), 3);
}

#[test]
#[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
fn test_execution_workflow_loads_with_rules() {
    use crate::engine::load_workflow;

    let temp_dir = setup_production_workflows();
    let workflow_path = temp_dir.path().join("workflows/execution.yaml");
    let workflow = load_workflow(&workflow_path).unwrap();
    assert_eq!(workflow.mode, "execution");

    // Verify code node has rules (4 rules including phase_timeout)
    let code_node = &workflow.nodes["code"];
    assert_eq!(code_node.rules.len(), 4);
}

#[test]
#[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
fn test_discovery_workflow_rules_are_valid() {
    use crate::engine::load_workflow;

    let temp_dir = setup_production_workflows();
    let workflow_path = temp_dir.path().join("workflows/discovery.yaml");
    let workflow = load_workflow(&workflow_path).unwrap();
    let code_node = &workflow.nodes["code"];

    // Validate all rules (errors would panic)
    for rule in &code_node.rules {
        rule.validate().unwrap();
    }
}

#[test]
#[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
fn test_execution_workflow_rules_are_valid() {
    use crate::engine::load_workflow;

    let temp_dir = setup_production_workflows();
    let workflow_path = temp_dir.path().join("workflows/execution.yaml");
    let workflow = load_workflow(&workflow_path).unwrap();
    let code_node = &workflow.nodes["code"];

    // Validate all rules (errors would panic)
    for rule in &code_node.rules {
        rule.validate().unwrap();
    }
}
