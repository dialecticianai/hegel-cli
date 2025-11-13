# src/commands/doctor/

Health checks and repairs for workflow state files and DDD artifacts. Detects and fixes common issues automatically.

## Purpose

Validates and repairs two categories of project health:
1. **State migrations**: Updates workflow state.json schema for backward compatibility
2. **DDD artifacts**: Fixes malformed artifact naming (underscores → hyphens)

## Structure

```
doctor/
├── mod.rs               Command orchestrator (routes to fix modules, handles --apply flag)
├── tests.rs             Integration tests for doctor command
│
├── fix_state.rs         State file validation and migration (rescue corrupted files, apply schema migrations)
└── fix_ddd.rs           DDD artifact naming repairs (git-based date discovery, rename with git mv/fs)
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
