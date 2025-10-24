use super::super::*;
use super::*;
use crate::test_helpers::*;

// ========== Node Flow Extraction Tests ==========

#[test]
fn test_extract_node_flow_linear() {
    use crate::engine::load_workflow_from_str;

    let yaml = r#"
mode: test
start_node: a
nodes:
  a:
    prompt: "Node A"
    transitions:
      - when: "next"
        to: b
  b:
    prompt: "Node B"
    transitions:
      - when: "next"
        to: c
  c:
    prompt: "Node C"
    transitions: []
"#;

    let workflow = load_workflow_from_str(yaml).unwrap();
    let flow = super::extract_node_flow(&workflow);
    assert_eq!(flow, "a → b → c");
}

#[test]
fn test_extract_node_flow_with_cycle_prevention() {
    use crate::engine::load_workflow_from_str;

    let yaml = r#"
mode: test
start_node: a
nodes:
  a:
    prompt: "Node A"
    transitions:
      - when: "next"
        to: b
  b:
    prompt: "Node B"
    transitions:
      - when: "next"
        to: a
"#;

    let workflow = load_workflow_from_str(yaml).unwrap();
    let flow = super::extract_node_flow(&workflow);
    // Should stop at b to prevent infinite loop
    assert_eq!(flow, "a → b");
}

#[test]
fn test_extract_node_flow_self_loop_skipped() {
    use crate::engine::load_workflow_from_str;

    let yaml = r#"
mode: test
start_node: a
nodes:
  a:
    prompt: "Node A"
    transitions:
      - when: "stay"
        to: a
      - when: "next"
        to: b
  b:
    prompt: "Node B"
    transitions: []
"#;

    let workflow = load_workflow_from_str(yaml).unwrap();
    let flow = super::extract_node_flow(&workflow);
    // Takes first transition, filters self-loops, so stops at 'a'
    assert_eq!(flow, "a");
}

#[test]
fn test_extract_node_flow_single_node() {
    use crate::engine::load_workflow_from_str;

    let yaml = r#"
mode: test
start_node: only
nodes:
  only:
    prompt: "Only node"
    transitions: []
"#;

    let workflow = load_workflow_from_str(yaml).unwrap();
    let flow = super::extract_node_flow(&workflow);
    assert_eq!(flow, "only");
}

#[test]
fn test_extract_node_flow_actual_workflow() {
    use crate::engine::load_workflow_from_str;

    // Test with a realistic workflow similar to discovery/execution
    let yaml = r#"
mode: test
start_node: spec
nodes:
  spec:
    prompt: "Spec"
    transitions:
      - when: "spec_complete"
        to: plan
  plan:
    prompt: "Plan"
    transitions:
      - when: "plan_complete"
        to: code
  code:
    prompt: "Code"
    transitions:
      - when: "code_complete"
        to: learn
  learn:
    prompt: "Learn"
    transitions:
      - when: "learn_complete"
        to: done
  done:
    prompt: "Done"
    transitions: []
"#;

    let workflow = load_workflow_from_str(yaml).unwrap();
    let flow = super::extract_node_flow(&workflow);
    assert_eq!(flow, "spec → plan → code → learn → done");
}
