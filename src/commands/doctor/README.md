# src/commands/doctor/

Health checks and repairs for workflow state files and DDD artifacts. Detects and fixes common issues automatically.

## Purpose

Validates and repairs four categories of project health:
1. **State migrations**: Updates workflow state.json schema for backward compatibility
2. **DDD artifacts**: Fixes malformed artifact naming (underscores → hyphens)
3. **Phase metrics**: Repairs archived phases/transitions (dedups records duplicated across archives, compacts each archive, prunes empty phases, rebuilds totals)
4. **Cowboy durations**: Trims synthetic cowboy rides to end at their last captured activity instead of the moment detection ran (preserving the anchored start), so overnight gaps don't inflate ride durations

## Structure

```
doctor/
├── mod.rs               Command orchestrator (routes to fix modules, handles --apply flag)
├── tests.rs             Integration tests for doctor command
│
├── fix_state.rs            State file validation and migration (rescue corrupted files, apply schema migrations)
├── fix_ddd.rs              DDD artifact naming repairs (git-based date discovery, rename with git mv/fs)
├── fix_phases.rs           Archive repair (cross-archive dedup of phases+transitions, intra-archive compaction, prune empty phases, rebuild totals)
└── fix_cowboy_durations.rs Trim synthetic cowboy rides to last captured activity (WorkflowArchive::trim_cowboy_to_activity)
```

## Workflow

**Detection mode (default)**: `hegel doctor`
- Shows issues found
- Displays suggested fixes
- No modifications made

**Apply mode**: `hegel doctor --apply`
- Detects issues
- Applies automatic fixes
- Uses `git mv` for tracked files, `fs::rename` for untracked

## Key Features

**State migrations**: Automatically updates state.json schema when Hegel version changes
**DDD naming**: Converts underscore-separated names to hyphen-separated (spec compliance)
**Git integration**: Preserves git history when renaming tracked artifacts
**Date discovery**: Uses git log to determine artifact creation dates for repairs
**Phase repair**: The pre-Nov-3 re-archiving bug wrote nested cumulative snapshots — the same phase/transition landed in many archive files, inflating phase lists, token totals, and the workflow graph. Repair keeps one canonical copy of each `(phase_name, start_time)` phase (highest-token; ties → earliest archive) and each `(from, to, timestamp)` transition (earliest archive), then compacts each archive (intra-archive dedup) and prunes non-terminal phases with no activity; terminal `done`/`aborted` phases are kept. Token/git totals are recomputed from survivors. A defensive read-time sort (`metrics/mod.rs`) keeps phases chronological regardless.
