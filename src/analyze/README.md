# src/commands/analyze_impl/

Implementation modules for `hegel analyze` command. Separated from `commands/analyze/` for modularity after refactoring from a large monolithic file.

## Purpose

Provides the implementation for metrics analysis, repair operations, and display rendering. The `commands/analyze/` directory contains the routing logic (~50 lines), while this module contains the actual implementation.

## Structure

```
analyze_impl/
├── mod.rs               Module exports
├── sections.rs          Rendering sections (session, tokens, activity, commands/files, transitions, phases, graph)
├── repair.rs            Archive repair orchestration (backfill, cowboy detection, reporting)
├── backfill.rs          Git metrics backfill (re-parse git history, attribute to phases)
├── gap_detection.rs     Workflow gap detection (identify and create synthetic cowboy archives)
└── totals.rs            Cumulative totals rebuilding (sum archive totals, update state)
```

## Key Features

**Section Rendering**: Formats metrics output (tokens, activity, phase breakdown, workflow graphs)
**Archive Repair**: Detects gaps in workflow coverage and creates synthetic cowboy archives for untracked work
**Git Backfill**: Retrospectively attributes git commits to workflow phases based on timestamps
**Totals Computation**: Aggregates metrics across all archived workflows
