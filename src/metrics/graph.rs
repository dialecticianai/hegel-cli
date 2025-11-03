//! Workflow graph reconstruction and visualization
//!
//! Builds a directed acyclic graph (DAG) from state transitions,
//! detects cycles (workflow loops), and provides ASCII + DOT rendering.

use crate::metrics::{PhaseMetrics, StateTransitionEvent};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DAGNode {
    pub phase_name: String,
    pub visits: usize,
    pub total_tokens: u64,
    pub total_duration_secs: u64,
    pub file_modifications: usize,
    pub bash_commands: usize,
}

#[derive(Debug, Clone)]
pub struct DAGEdge {
    pub from: String,
    pub to: String,
    pub count: usize,
}

#[derive(Debug)]
pub struct WorkflowDAG {
    pub nodes: HashMap<String, DAGNode>,
    pub edges: Vec<DAGEdge>,
}

impl WorkflowDAG {
    /// Build DAG from state transitions and phase metrics
    pub fn from_transitions(
        transitions: &[StateTransitionEvent],
        phase_metrics: &[PhaseMetrics],
    ) -> Self {
        let mut nodes = HashMap::new();
        let mut edges_map: HashMap<(String, String), usize> = HashMap::new();

        // Initialize nodes from phase metrics
        for phase in phase_metrics {
            let entry = nodes
                .entry(phase.phase_name.clone())
                .or_insert_with(|| DAGNode {
                    phase_name: phase.phase_name.clone(),
                    visits: 0,
                    total_tokens: 0,
                    total_duration_secs: 0,
                    file_modifications: 0,
                    bash_commands: 0,
                });

            entry.visits += 1;
            entry.total_tokens +=
                phase.token_metrics.total_input_tokens + phase.token_metrics.total_output_tokens;
            entry.total_duration_secs += phase.duration_seconds;
            entry.file_modifications += phase.file_modifications.len();
            entry.bash_commands += phase.bash_commands.len();
        }

        // Build edges from transitions
        for i in 0..transitions.len().saturating_sub(1) {
            let from = &transitions[i].phase;
            let to = &transitions[i + 1].phase;

            // Skip self-loops (same phase transition)
            if from != to {
                *edges_map.entry((from.clone(), to.clone())).or_insert(0) += 1;
            }
        }

        let edges: Vec<DAGEdge> = edges_map
            .into_iter()
            .map(|((from, to), count)| DAGEdge { from, to, count })
            .collect();

        Self { nodes, edges }
    }

    /// Detect cycles in the graph (indicates workflow loops)
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut vec![], &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        // Find outgoing edges
        for edge in &self.edges {
            if edge.from == node {
                if rec_stack.contains(&edge.to) {
                    // Found cycle - extract the cycle portion
                    if let Some(cycle_start) = path.iter().position(|n| n == &edge.to) {
                        let mut cycle = path[cycle_start..].to_vec();
                        cycle.push(edge.to.clone()); // Close the cycle
                        cycles.push(cycle);
                    }
                } else if !visited.contains(&edge.to) {
                    self.dfs_cycle(&edge.to, visited, rec_stack, path, cycles);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Render as ASCII art using box-drawing characters
    pub fn render_ascii(&self) -> String {
        let mut output = String::new();

        // Sort nodes alphabetically for consistent output
        let mut sorted_nodes: Vec<_> = self.nodes.keys().collect();
        sorted_nodes.sort();

        for node_name in &sorted_nodes {
            let node = &self.nodes[*node_name];

            // Node header
            output.push_str(&format!("┌─ {} ", node.phase_name.to_uppercase()));
            output.push_str(&"─".repeat(50));
            output.push_str("┐\n");

            // Node stats
            output.push_str(&format!(
                "│ Visits: {:>3}  Tokens: {:>8}  Duration: {:>5}s",
                node.visits, node.total_tokens, node.total_duration_secs
            ));
            output.push_str(&" ".repeat(12));
            output.push_str("│\n");

            output.push_str(&format!(
                "│ Bash: {:>3}  Files: {:>3}",
                node.bash_commands, node.file_modifications
            ));
            output.push_str(&" ".repeat(42));
            output.push_str("│\n");

            output.push_str("└");
            output.push_str(&"─".repeat(62));
            output.push_str("┘\n");

            // Outgoing edges
            let outgoing: Vec<_> = self
                .edges
                .iter()
                .filter(|e| e.from == **node_name)
                .collect();

            for (i, edge) in outgoing.iter().enumerate() {
                let connector = if i == outgoing.len() - 1 {
                    "└─"
                } else {
                    "├─"
                };
                output.push_str(&format!(
                    "  {}→ {}  ({}x)\n",
                    connector, edge.to, edge.count
                ));
            }

            output.push('\n');
        }

        output
    }

    /// Export as DOT format for Graphviz
    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph workflow {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Nodes
        for (name, node) in &self.nodes {
            let label = format!(
                "{}\\n{} tokens\\n{}s\\n{} visits",
                name, node.total_tokens, node.total_duration_secs, node.visits
            );
            dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", name, label));
        }

        dot.push('\n');

        // Edges
        for edge in &self.edges {
            let label = if edge.count > 1 {
                format!(" [label=\"{}x\"]", edge.count)
            } else {
                String::new()
            };
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\"{};\n",
                edge.from, edge.to, label
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
        vec![
            PhaseMetrics {
                phase_name: "spec".to_string(),
                start_time: "2025-01-01T10:00:00Z".to_string(),
                end_time: Some("2025-01-01T10:15:00Z".to_string()),
                duration_seconds: 900,
                token_metrics: TokenMetrics {
                    total_input_tokens: 1000,
                    total_output_tokens: 500,
                    total_cache_creation_tokens: 0,
                    total_cache_read_tokens: 0,
                    assistant_turns: 5,
                },
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            },
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
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            },
        ]
    }

    #[test]
    fn test_build_workflow_dag() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);

        assert_eq!(dag.nodes.len(), 2); // spec and plan (code has no metrics)
        assert!(dag.nodes.contains_key("spec"));
        assert!(dag.nodes.contains_key("plan"));

        // Check edges
        assert_eq!(dag.edges.len(), 2); // spec->plan, plan->code
    }

    #[test]
    fn test_dag_annotation() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);

        let spec_node = &dag.nodes["spec"];
        assert_eq!(spec_node.total_tokens, 1500); // 1000 + 500
        assert_eq!(spec_node.total_duration_secs, 900);
        assert_eq!(spec_node.visits, 1);

        let plan_node = &dag.nodes["plan"];
        assert_eq!(plan_node.total_tokens, 3000); // 2000 + 1000
    }

    #[test]
    fn test_detect_cycles() {
        let mut transitions = test_transitions();

        // Add a cycle: code -> spec
        transitions.push(StateTransitionEvent {
            timestamp: "2025-01-01T10:45:00Z".to_string(),
            workflow_id: Some("test".to_string()),
            from_node: "code".to_string(),
            to_node: "spec".to_string(),
            phase: "spec".to_string(),
            mode: "discovery".to_string(),
        });

        let dag = WorkflowDAG::from_transitions(&transitions, &test_phase_metrics());
        let cycles = dag.find_cycles();

        assert!(!cycles.is_empty());
        assert!(cycles.iter().any(|c| c.contains(&"spec".to_string())
            && c.contains(&"plan".to_string())
            && c.contains(&"code".to_string())));
    }

    #[test]
    fn test_render_ascii_dag() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);
        let ascii = dag.render_ascii();

        assert!(ascii.contains("SPEC"));
        assert!(ascii.contains("PLAN"));
        assert!(ascii.contains("→"));
        assert!(ascii.contains("├─") || ascii.contains("└─"));
    }

    #[test]
    fn test_export_dot() {
        let transitions = test_transitions();
        let phase_metrics = test_phase_metrics();
        let dag = WorkflowDAG::from_transitions(&transitions, &phase_metrics);
        let dot = dag.export_dot();

        assert!(dot.contains("digraph workflow"));
        assert!(dot.contains("spec"));
        assert!(dot.contains("plan"));
        assert!(dot.contains("->"));
        assert!(dot.contains("1500 tokens")); // spec total
    }
}
