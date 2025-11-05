# Refactor Analysis: Large Files (>300 impl lines)

**Date**: 2025-11-04
**Trigger**: LOC_REPORT.md flagged 6 files over 200 impl lines, 3 exceed 300 lines
**Goal**: Maximize token efficiency through SoC and DRY principles

---

## Files Targeted

| File | Impl Lines | Status |
|------|------------|--------|
| `commands/fork/mod.rs` | 426 | ✅ Analyzed |
| `commands/analyze/mod.rs` | 405 | ✅ Analyzed |
| `storage/archive.rs` | 322 | ✅ Analyzed |

---

## 1. `commands/fork/mod.rs` (426 lines)

### SoC Violations

**Version Management (Lines 106-164):**
- `expand_tilde()` - Path expansion utility
- `parse_version()` - Version string parsing
- `get_current_node_version()` - Node.js version detection
- `find_nvm_compatible_version()` - nvm directory scanning

**Runtime Compatibility (Lines 166-227):**
- `check_runtime_compatibility()` - Version validation logic
- `RuntimeCompatibility` enum - Compatibility status tracking

**Agent Detection (Lines 229-271):**
- `detect_agents()` - Agent discovery via `which` and fallback paths
- Mixed with display orchestration

**Display Logic (Lines 273-344):**
- `display_agents()` - Terminal formatting and output
- Inline runtime status display

**Agent Execution (Lines 346-393):**
- `execute_agent()` - Command building and execution
- Runtime compatibility enforcement

### Proposed Split (2 files)

```
fork/
├── mod.rs              # Agent detection, display, orchestration (~250 lines)
│   - detect_agents()
│   - display_agents()
│   - handle_fork()
│   - Agent/AgentMetadata types
│   - KNOWN_AGENTS const
│
└── runtime.rs          # Runtime version management (~170 lines)
    - AgentRuntime enum
    - RuntimeCompatibility enum
    - expand_tilde()
    - parse_version()
    - get_current_node_version()
    - find_nvm_compatible_version()
    - check_runtime_compatibility()
    - execute_agent()
```

### Rationale

- **mod.rs**: User-facing agent functionality (detection/display/orchestration)
- **runtime.rs**: Technical runtime version management (Node.js/Python/Native)
- Clean separation: "What agents exist?" vs "Can we run this agent?"

### Token Savings

- Before: 426 lines (read entire file for any concern)
- After: 250 lines (agent detection) OR 170 lines (runtime logic)
- Reduction: 41-60% depending on concern

---

## 2. `commands/analyze/mod.rs` (405 lines)

### SoC Violations

**Mixed Responsibilities:**
- Main analyze orchestrator (lines 10-46)
- Archive repair (lines 49-231) - monolithic function handling:
  - Report formatting
  - Git backfill
  - Cowboy detection
  - Dry-run logic
  - JSON output
- Git backfill implementation (lines 233-281)
- Cowboy detection implementation (lines 283-374)
- Cumulative totals rebuilding (lines 376-404)

### Proposed Split (5 files)

```
analyze/
├── mod.rs              # Main analyze orchestrator (~50 lines)
│   - analyze_metrics() - routing to sections or repair
│
├── sections.rs         # [UNCHANGED] Rendering sections
│
├── repair.rs           # Archive repair orchestrator (~100 lines)
│   - repair_archives() - main repair logic
│   - Report formatting and output
│
├── backfill.rs         # Git metrics backfill (~50 lines)
│   - backfill_git_metrics()
│
├── gap_detection.rs    # Workflow gap detection (~90 lines)
│   - detect_and_create_cowboy_archives()
│
└── totals.rs           # Cumulative totals (~30 lines)
    - rebuild_cumulative_totals()
```

### Rationale

- **mod.rs**: Thin orchestrator, routes to analyze or repair
- **repair.rs**: Repair workflow orchestration (what to repair, reporting)
- **backfill.rs**: Git-specific backfill logic
- **gap_detection.rs**: Workflow gap detection (synthetic cowboy workflows)
- **totals.rs**: State rebuilding utility

### Token Savings

- Before: 405 lines (read entire file for repair/analyze/backfill)
- After: 30-100 lines per concern
- Reduction: 75-92% depending on concern

---

## 3. `storage/archive.rs` (322 lines)

### SoC Violations

**Archive Creation (Lines 86-193):**
- `WorkflowArchive::from_metrics()` - 107 lines
- Validation, aggregation, conversion all mixed

**Aggregation Logic (Lines 114-147):**
- Bash command aggregation (HashMap freq counting)
- File modification aggregation (identical pattern)

**Totals Computation (Lines 215-254):**
- Token summation
- Unique counting
- Git commit counting

### DRY Violations

**Lines 114-129 vs 131-147:**
Nearly identical aggregation pattern:
```rust
// Bash commands
let mut bash_freq: HashMap<String, Vec<String>> = HashMap::new();
for cmd in &phase.bash_commands {
    bash_freq.entry(cmd.command.clone())
        .or_insert_with(Vec::new)
        .push(cmd.timestamp.clone().unwrap_or_default());
}

// File modifications (same pattern!)
let mut file_freq: HashMap<(String, String), Vec<String>> = HashMap::new();
for file_mod in &phase.file_modifications {
    file_freq.entry((file_mod.file_path.clone(), file_mod.tool.clone()))
        .or_insert_with(Vec::new)
        .push(file_mod.timestamp.clone().unwrap_or_default());
}
```

### Proposed Split (4 files)

```
archive/
├── mod.rs              # Public API + core types (~100 lines)
│   - WorkflowArchive struct
│   - PhaseArchive, TransitionArchive, TokenTotals structs
│   - read_archives()
│   - write_archive()
│
├── builder.rs          # Archive creation (~80 lines)
│   - WorkflowArchive::from_metrics()
│   - Uses aggregation.rs helpers
│
├── aggregation.rs      # Aggregation helpers (~60 lines)
│   - aggregate_by_key<T, F>() - generic aggregation
│   - aggregate_bash_commands()
│   - aggregate_file_modifications()
│   - compute_totals()
│
└── validation.rs       # Validation (~20 lines)
    - validate_workflow_id()
```

### DRY Fix

Extract common aggregation pattern:
```rust
fn aggregate_by_key<T, F, K>(
    items: &[T],
    key_fn: F,
) -> HashMap<K, Vec<String>>
where
    F: Fn(&T) -> (K, Option<String>),
    K: Eq + Hash,
{
    let mut freq: HashMap<K, Vec<String>> = HashMap::new();
    for item in items {
        let (key, timestamp) = key_fn(item);
        freq.entry(key)
            .or_insert_with(Vec::new)
            .push(timestamp.unwrap_or_default());
    }
    freq
}
```

### Rationale

- **mod.rs**: Core types and I/O operations
- **builder.rs**: Archive construction logic
- **aggregation.rs**: Reusable aggregation utilities (DRY)
- **validation.rs**: Input validation utilities

### Token Savings

- Before: 322 lines
- After: 20-100 lines per concern
- Reduction: 69-94% depending on concern
- DRY: ~30 lines of duplicated logic eliminated

---

## Implementation Order

1. ✅ **`storage/archive.rs`** - Smallest, clearest SoC boundaries, demonstrates DRY benefits
2. **`commands/fork/mod.rs`** - Simple 2-file split, good next step
3. **`commands/analyze/mod.rs`** - Most complex, benefits from seeing patterns from #1 and #2

---

## Success Criteria

- All existing tests pass without modification
- No functionality changes
- Backwards compatibility maintained (re-exports in mod.rs)
- LOC_REPORT.md shows no files >200 impl lines in refactored modules
- Token overhead reduced by 60-90% when reading specific concerns

---

## Notes

- All splits maintain backwards compatibility through re-exports
- Tests remain in original locations (co-located with functionality)
- No changes to public APIs
- Focus on token-efficient legibility, not architectural changes
