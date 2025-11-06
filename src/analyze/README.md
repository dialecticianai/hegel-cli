# src/analyze/

Implementation modules for `hegel analyze` command. Separated from `commands/analyze/` for modularity after refactoring from a large monolithic file.

## Purpose

Provides the implementation for metrics analysis, repair operations, and display rendering. The `commands/analyze/` directory contains the routing logic (~50 lines), while this module contains the actual implementation.

## Structure

```
analyze/
├── mod.rs               Module exports
├── sections.rs          Rendering sections (brief summary, activity, tokens, transitions, phases, graph)
├── repair.rs            Archive repair orchestration (cleanup trait integration, reporting)
├── gap_detection.rs     Workflow gap detection (identify and create synthetic cowboy archives)
├── totals.rs            Cumulative totals rebuilding (sum archive totals, update state)
│
├── cleanup/             Trait-based archive cleanup system (See cleanup/README.md)
└── tests/               Test suites for analyze module (gap_detection coverage)
```

## Key Features

**Section Rendering**: Formats metrics output with progressive disclosure (brief summary default, detailed sections via flags)
**Archive Repair**: Detects gaps in workflow coverage and repairs archives via cleanup trait implementations
**Cleanup System**: Pluggable repair operations (git backfill, aborted node addition) via ArchiveCleanup trait
**Totals Computation**: Aggregates metrics across all archived workflows
