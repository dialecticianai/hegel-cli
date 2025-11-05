# Refactor: Workflow Graph Grouping

**Date**: 2025-11-05
**Scope**: `src/metrics/graph.rs`

## Problem

The DOT graph export generates a flat global graph that aggregates all transitions across all workflows, losing workflow boundaries and making individual workflow flows invisible.

**Current behavior:**
- `WorkflowDAG::from_transitions()` aggregates all transitions globally
- No concept of workflow grouping by `workflow_id`
- Result: `done → spec (65x), done → code (4x)` shows aggregate counts, not individual flows

**Example output (current):**
```dot
"done" -> "plan" [label="65x"];
"done" -> "code" [label="4x"];
"done" -> "refactor" [label="9x"];
```

**What we need:**
```dot
subgraph cluster_workflow_1 {
  label="Workflow 2025-11-04T22:52";
  spec -> plan -> code -> code_review -> readme -> done;
}
subgraph cluster_workflow_2 {
  label="Workflow 2025-10-24T16:39";
  ride -> done;
}
done -> spec [style=dashed]; // Inter-workflow connection
```

## Files Targeted

- `src/metrics/graph.rs` - Core graph construction and DOT export logic

## Violations Identified

**Architectural issue:**
- `from_transitions()` discards `workflow_id` information (line 67-75)
- No data structure to represent individual workflows
- `export_dot()` has no concept of workflow grouping

## Proposed Solution

### Option 1: New method alongside existing (RECOMMENDED)
Keep existing `from_transitions()` for ASCII rendering (global aggregate view), add new method for workflow-grouped export:

```rust
pub fn export_dot_by_workflow(
    transitions: &[StateTransitionEvent],
    phase_metrics: &[PhaseMetrics],
) -> String {
    // Group transitions by workflow_id
    // Generate DOT with subgraph clusters
    // Connect workflows via DONE nodes
}
```

### Option 2: Refactor from_transitions to preserve workflow_id
Add `workflow_id` to internal structures, modify existing method:

```rust
pub struct DAGNode {
    // ... existing fields
    pub workflow_id: Option<String>, // NEW
}

pub struct DAGEdge {
    // ... existing fields
    pub workflow_id: Option<String>, // NEW
}
```

## Recommendation

**Modified approach** - Replace broken global graph everywhere:
- Both ASCII and DOT rendering use workflow-grouped view
- Remove/replace existing `from_transitions()` global aggregate
- Show workflows as visually distinct groups in both formats
- Inter-workflow connections clearly distinguished

## Implementation Steps

1. Add new data structures to represent workflow groups
2. Add `from_transitions_by_workflow()` method to build workflow-grouped structure
3. Update `render_ascii()` to show workflows grouped with visual separation
4. Add `export_dot_by_workflow()` to generate DOT with subgraph clusters
5. Update `render_workflow_graph()` in `src/analyze/sections.rs` for grouped ASCII
6. Update `render_workflow_graph_dot()` in `src/analyze/sections.rs` for grouped DOT
7. Remove or deprecate old global aggregate methods

## Token Savings

Not applicable - this is a functional fix, not a DRY/token optimization.

## Testing Plan

- Test with real `.hegel/states.jsonl` data
- Verify subgraph clustering works
- Verify inter-workflow connections render correctly
- Ensure synthetic workflows (cowboy mode) are distinguished
