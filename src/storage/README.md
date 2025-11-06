# src/storage/

Layer 4: Atomic persistence and event logging. File-based state management with atomic writes, parent directory discovery, and JSONL event streaming.

## Purpose

Provides reliable local-first persistence for workflow state. Manages `.hegel/state.json` (current state), `.hegel/states.jsonl` (transition log), `.hegel/hooks.jsonl` (agent activity), and `.hegel/archives/` (completed workflow snapshots). Implements atomic writes, file locking, and git-style parent directory discovery.

## Structure

```
storage/
├── mod.rs               FileStorage (load/save/clear state.json, log_state_transition, parent dir discovery, file locking)
│
└── archive/             Workflow archive storage with pre-computed aggregates
    ├── mod.rs           Core types (WorkflowArchive, PhaseArchive, TokenTotals) + I/O (read/write_archive)
    ├── builder.rs       Archive construction (WorkflowArchive::from_metrics)
    ├── aggregation.rs   Aggregation utilities with DRY helpers (aggregate_bash_commands, aggregate_file_modifications, compute_totals)
    └── validation.rs    Input validation (validate_workflow_id for path safety)
```

## Key Features

**Atomic Writes**: State updates use temp file + rename to prevent corruption
**File Locking**: Exclusive locks on JSONL appends prevent concurrent write corruption (fs2 crate)
**Parent Discovery**: Finds `.hegel/` by walking up directory tree (like git), works from any subdirectory
**Event Streaming**: Append-only JSONL logs for hooks and state transitions
**Archive Management**: Completed workflows archived with pre-computed metrics for fast analysis

## State Files

- `state.json` - Current workflow state (workflow, workflow_state with is_handlebars flag, session_metadata)
- `states.jsonl` - State transition event log (timestamped from→to transitions)
- `hooks.jsonl` - Agent activity log (tool usage, bash commands, file edits)
- `archives/` - Completed workflow snapshots with aggregated metrics

## WorkflowState Fields

- `current_node` - Current phase in workflow
- `mode` - Workflow mode (discovery/execution)
- `history` - Ordered list of visited nodes
- `workflow_id` - Optional unique identifier for workflow run
- `meta_mode` - Optional meta-mode wrapper (fork/mirror)
- `phase_start_time` - RFC3339 timestamp for time-based rules
- `is_handlebars` - Template engine flag (true = Handlebars, false = Markdown)
