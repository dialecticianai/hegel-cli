//! Workflow graph reconstruction and visualization
//!
//! Builds a directed acyclic graph (DAG) from state transitions,
//! detects cycles (workflow loops), and provides ASCII + DOT rendering.

use crate::metrics::{PhaseMetrics, StateTransitionEvent};
use std::collections::{HashMap, HashSet};

/// Number of workflows to display per row in DOT graph layout
const WORKFLOWS_PER_ROW: usize = 5;

/// A single workflow with its phases in sequence
#[derive(Debug, Clone)]
pub struct WorkflowGroup {
    pub workflow_id: String,
    pub phases: Vec<String>,
    pub phase_data: HashMap<String, PhaseMetrics>,
    pub is_synthetic: bool,
}

#[derive(Debug)]
pub struct WorkflowDAG {
    pub workflows: Vec<WorkflowGroup>,
    pub inter_workflow_edges: Vec<(String, String, String, String)>,
}

impl WorkflowDAG {
    /// Build grouped DAG from state transitions and phase metrics
    pub fn from_transitions(
        transitions: &[StateTransitionEvent],
        phase_metrics: &[PhaseMetrics],
    ) -> Self {
        // Create a map of phase metrics by phase name for quick lookup
        let mut phase_data_map: HashMap<String, Vec<PhaseMetrics>> = HashMap::new();
        for metric in phase_metrics {
            phase_data_map
                .entry(metric.phase_name.clone())
                .or_insert_with(Vec::new)
                .push(metric.clone());
        }

        // Group transitions by workflow_id
        let mut workflow_transitions: HashMap<String, Vec<StateTransitionEvent>> = HashMap::new();
        for transition in transitions {
            if let Some(wid) = &transition.workflow_id {
                workflow_transitions
                    .entry(wid.clone())
                    .or_insert_with(Vec::new)
                    .push(transition.clone());
            }
        }

        // Build workflow groups
        let mut workflows = Vec::new();
        let mut inter_workflow_edges = Vec::new();
        let mut prev_workflow_id: Option<String> = None;
        let mut prev_phase: Option<String> = None;

        for (workflow_id, trans) in &workflow_transitions {
            // Extract ordered phases for this workflow
            let mut phases = Vec::new();
            let mut seen_phases = HashSet::new();

            for t in trans {
                if !seen_phases.contains(&t.phase) {
                    phases.push(t.phase.clone());
                    seen_phases.insert(t.phase.clone());
                }
            }

            // Determine if workflow is synthetic
            let is_synthetic = trans.iter().any(|t| {
                if let Some(metrics) = phase_data_map.get(&t.phase) {
                    metrics.iter().any(|m| m.is_synthetic)
                } else {
                    false
                }
            });

            // Build phase_data map for this workflow
            let mut phase_data = HashMap::new();
            for phase in &phases {
                if let Some(metrics) = phase_data_map.get(phase) {
                    if let Some(metric) = metrics.first() {
                        phase_data.insert(phase.clone(), metric.clone());
                    }
                }
            }

            workflows.push(WorkflowGroup {
                workflow_id: workflow_id.clone(),
                phases,
                phase_data,
                is_synthetic,
            });

            // Check for inter-workflow edge
            if let (Some(prev_wid), Some(prev_p)) = (&prev_workflow_id, &prev_phase) {
                if let Some(current_first_phase) = workflows.last().and_then(|w| w.phases.first()) {
                    if prev_wid != workflow_id {
                        inter_workflow_edges.push((
                            prev_wid.clone(),
                            prev_p.clone(),
                            workflow_id.clone(),
                            current_first_phase.clone(),
                        ));
                    }
                }
            }

            prev_workflow_id = Some(workflow_id.clone());
            prev_phase = workflows.last().and_then(|w| w.phases.last()).cloned();
        }

        Self {
            workflows,
            inter_workflow_edges,
        }
    }

    /// Render grouped workflows as ASCII
    pub fn render_ascii(&self) -> String {
        let mut output = String::new();

        for (idx, workflow) in self.workflows.iter().enumerate() {
            let synthetic_label = if workflow.is_synthetic {
                " (synthetic)"
            } else {
                ""
            };

            output.push_str(&format!(
                "\n┌─ Workflow {}: {}{} ",
                idx + 1,
                workflow.workflow_id,
                synthetic_label
            ));
            output.push_str(&"─".repeat(40));
            output.push_str("┐\n");

            // Show phases in sequence
            for (phase_idx, phase_name) in workflow.phases.iter().enumerate() {
                let arrow = if phase_idx == 0 { "│ " } else { "│ → " };

                if let Some(metrics) = workflow.phase_data.get(phase_name) {
                    let total_tokens = metrics.token_metrics.total_input_tokens
                        + metrics.token_metrics.total_output_tokens;
                    let duration = metrics.duration_seconds;

                    output.push_str(&format!(
                        "{}{} ({} tokens, {}s)\n",
                        arrow,
                        phase_name.to_uppercase(),
                        total_tokens,
                        duration
                    ));
                } else {
                    output.push_str(&format!("{}{}\n", arrow, phase_name.to_uppercase()));
                }
            }

            output.push_str("└");
            output.push_str(&"─".repeat(64));
            output.push_str("┘\n");
        }

        // Show inter-workflow connections
        if !self.inter_workflow_edges.is_empty() {
            output.push_str("\nInter-workflow Connections:\n");
            for (from_wid, from_phase, to_wid, to_phase) in &self.inter_workflow_edges {
                output.push_str(&format!(
                    "  {} ({}) → {} ({})\n",
                    from_wid, from_phase, to_wid, to_phase
                ));
            }
        }

        output
    }

    /// Export as DOT format with subgraph clusters
    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph workflow {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=rounded];\n");
        dot.push_str("  compound=true;\n");
        dot.push_str("  newrank=true;\n");
        dot.push_str("  ranksep=1.5;\n\n");

        // Generate subgraph for each workflow
        for (idx, workflow) in self.workflows.iter().enumerate() {
            let synthetic_label = if workflow.is_synthetic {
                " (synthetic)"
            } else {
                ""
            };

            dot.push_str(&format!("  subgraph cluster_{} {{\n", idx));
            dot.push_str(&format!(
                "    label=\"Workflow {}{}\";\n",
                workflow.workflow_id, synthetic_label
            ));

            if workflow.is_synthetic {
                dot.push_str("    style=dashed;\n");
            } else {
                dot.push_str("    style=solid;\n");
            }

            dot.push_str("    color=blue;\n\n");

            // Add nodes for each phase
            for phase_name in &workflow.phases {
                let node_id = format!("{}_{}", workflow.workflow_id, phase_name);

                if let Some(metrics) = workflow.phase_data.get(phase_name) {
                    let total_tokens = metrics.token_metrics.total_input_tokens
                        + metrics.token_metrics.total_output_tokens;
                    let duration = metrics.duration_seconds;

                    dot.push_str(&format!(
                        "    \"{}\" [label=\"{}\\n{} tokens\\n{}s\"];\n",
                        node_id, phase_name, total_tokens, duration
                    ));
                } else {
                    dot.push_str(&format!(
                        "    \"{}\" [label=\"{}\"];\n",
                        node_id, phase_name
                    ));
                }
            }

            // Add edges between phases within workflow
            for i in 0..workflow.phases.len().saturating_sub(1) {
                let from = format!("{}_{}", workflow.workflow_id, workflow.phases[i]);
                let to = format!("{}_{}", workflow.workflow_id, workflow.phases[i + 1]);
                dot.push_str(&format!("    \"{}\" -> \"{}\";\n", from, to));
            }

            dot.push_str("  }\n\n");
        }

        // Group workflows into rows using rank=same
        let mut row_start = 0;
        while row_start < self.workflows.len() {
            let row_end = (row_start + WORKFLOWS_PER_ROW).min(self.workflows.len());
            if row_end - row_start > 1 {
                dot.push_str("  { rank=same; ");
                for i in row_start..row_end {
                    let first_node = format!(
                        "\"{}_{}\";",
                        self.workflows[i].workflow_id, self.workflows[i].phases[0]
                    );
                    dot.push_str(&first_node);
                    dot.push(' ');
                }
                dot.push_str("}\n");
            }
            row_start = row_end;
        }
        dot.push('\n');

        // Add inter-workflow edges
        for (from_wid, from_phase, to_wid, to_phase) in &self.inter_workflow_edges {
            let from_node = format!("{}_{}", from_wid, from_phase);
            let to_node = format!("{}_{}", to_wid, to_phase);
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [style=dashed, color=red];\n",
                from_node, to_node
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::TokenMetrics;

    fn test_transitions() -> Vec<StateTransitionEvent> {
        vec![
            StateTransitionEvent {
                timestamp: "2025-01-01T10:00:00Z".to_string(),
                workflow_id: Some("test".to_string()),
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                phase: "spec".to_string(),
                mode: "discovery".to_string(),
            },
            StateTransitionEvent {
                timestamp: "2025-01-01T10:15:00Z".to_string(),
                workflow_id: Some("test".to_string()),
                from_node: "spec".to_string(),
                to_node: "plan".to_string(),
                phase: "plan".to_string(),
                mode: "discovery".to_string(),
            },
            StateTransitionEvent {
                timestamp: "2025-01-01T10:30:00Z".to_string(),
                workflow_id: Some("test".to_string()),
                from_node: "plan".to_string(),
                to_node: "code".to_string(),
                phase: "code".to_string(),
                mode: "discovery".to_string(),
            },
        ]
    }

    fn test_phase_metrics() -> Vec<PhaseMetrics> {
        use crate::test_helpers::test_phase_metrics;

        vec![
            test_phase_metrics(),
            PhaseMetrics {
                phase_name: "plan".to_string(),
                start_time: "2025-01-01T10:15:00Z".to_string(),
                end_time: Some("2025-01-01T10:30:00Z".to_string()),
                duration_seconds: 900,
                token_metrics: TokenMetrics {
                    total_input_tokens: 2000,
                    total_output_tokens: 1000,
                    total_cache_creation_tokens: 0,
                    total_cache_read_tokens: 0,
                    assistant_turns: 8,
                },
                ..test_phase_metrics()
            },
        ]
    }

    #[test]
    fn test_build_workflow_dag() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);

        // All transitions have same workflow_id, should be 1 workflow
        assert_eq!(dag.workflows.len(), 1);

        let workflow = &dag.workflows[0];
        assert_eq!(workflow.workflow_id, "test");
        assert_eq!(workflow.phases.len(), 3); // spec, plan, code
        assert!(workflow.phases.contains(&"spec".to_string()));
        assert!(workflow.phases.contains(&"plan".to_string()));
        assert!(workflow.phases.contains(&"code".to_string()));
    }

    #[test]
    fn test_dag_annotation() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);

        let workflow = &dag.workflows[0];

        // Check spec metrics
        let spec_metrics = &workflow.phase_data["spec"];
        assert_eq!(
            spec_metrics.token_metrics.total_input_tokens
                + spec_metrics.token_metrics.total_output_tokens,
            1500
        );
        assert_eq!(spec_metrics.duration_seconds, 900);

        // Check plan metrics
        let plan_metrics = &workflow.phase_data["plan"];
        assert_eq!(
            plan_metrics.token_metrics.total_input_tokens
                + plan_metrics.token_metrics.total_output_tokens,
            3000
        );
    }

    #[test]
    fn test_multi_workflow() {
        let mut transitions = test_transitions();

        // Add another workflow
        transitions.push(StateTransitionEvent {
            timestamp: "2025-01-01T11:00:00Z".to_string(),
            workflow_id: Some("test2".to_string()),
            from_node: "START".to_string(),
            to_node: "spec".to_string(),
            phase: "spec".to_string(),
            mode: "discovery".to_string(),
        });

        let dag = WorkflowDAG::from_transitions(&transitions, &test_phase_metrics());

        assert_eq!(dag.workflows.len(), 2);
        let workflow_ids: Vec<_> = dag
            .workflows
            .iter()
            .map(|w| w.workflow_id.as_str())
            .collect();
        assert!(workflow_ids.contains(&"test"));
        assert!(workflow_ids.contains(&"test2"));
    }

    #[test]
    fn test_render_ascii_dag() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);
        let ascii = dag.render_ascii();

        assert!(ascii.contains("Workflow"));
        assert!(ascii.contains("SPEC"));
        assert!(ascii.contains("PLAN"));
        assert!(ascii.contains("→"));
    }

    #[test]
    fn test_export_dot() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);
        let dot = dag.export_dot();

        assert!(dot.contains("digraph workflow"));
        assert!(dot.contains("subgraph cluster"));
        assert!(dot.contains("spec"));
        assert!(dot.contains("plan"));
        assert!(dot.contains("->"));
        assert!(dot.contains("1500 tokens")); // spec total
    }
}
