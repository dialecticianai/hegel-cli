# src/metrics/

Layer 5: Event stream parsing, aggregation, and visualization. Correlates hooks, states, and transcripts by timestamp to provide unified workflow metrics.

## Purpose

Parses three independent event streams (hooks.jsonl, states.jsonl, transcripts) and correlates them via timestamps to produce per-phase metrics, workflow graphs, and aggregate statistics. Enables analysis commands like `hegel analyze` and `hegel top`.

## Structure

```
metrics/
├── mod.rs               Unified metrics orchestrator, parse_unified_metrics entry point
├── aggregation.rs       Phase metrics builder (timestamp correlation, token aggregation per phase)
├── hooks.rs             Parses Claude Code hook events, extracts bash commands and file modifications
├── states.rs            Parses workflow state transition events
├── transcript.rs        Parses Claude Code transcripts for token usage (handles old and new format)
├── git.rs               Git commit tracking and attribution (parse git log, attribute to phases by timestamp)
├── cowboy.rs            Cowboy mode activity attribution (detects inter-workflow gaps, creates synthetic cowboy sessions)
└── graph.rs             Workflow DAG reconstruction (groups workflows, tracks inter-workflow connections, ASCII/DOT rendering)
```

## Event Stream Correlation

Three independent event streams correlate via timestamps:

1. **hooks.jsonl** - Claude Code activity (tool usage, bash commands, file edits)
2. **states.jsonl** - Hegel workflow transitions (phase changes)
3. **Transcripts** - Token usage (input/output/cache metrics)

**Correlation Strategy**:
- Workflow membership: `hook.timestamp >= state.workflow_id`
- Per-phase attribution: `state[X].timestamp <= hook.timestamp < state[X+1].timestamp`
- Git commits: Attributed to phases by commit timestamp matching phase time ranges
