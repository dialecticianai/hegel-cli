# src/analyze/cleanup/

Trait-based archive cleanup system for detecting and repairing issues in workflow archives.

## Purpose

Provides pluggable repair strategies for archived workflows. Each cleanup implementation can detect specific issues (missing git metrics, incomplete terminal nodes) and repair them via the `ArchiveCleanup` trait.

## Structure

```
cleanup/
├── mod.rs               ArchiveCleanup trait definition and cleanup registry
├── git.rs               GitBackfillCleanup - backfills missing git commit metrics
└── aborted.rs           AbortedNodeCleanup - adds aborted terminal nodes to incomplete workflows
```

## Key Concepts

**ArchiveCleanup Trait**: Defines `needs_repair()` detection and `repair()` mutation interface
**Cleanup Registry**: `all_cleanups()` returns all available cleanup strategies for batch processing
**Dry-Run Support**: All cleanups support dry-run mode for issue detection without mutation
