# Log Retention and Archiving Specification

Automatic archiving of completed workflow metrics with aggregate preservation and log cleanup.

---

## Overview

**What it does:** Automatically archives completed workflows to `.hegel/archive/{workflow_id}.json` with pre-computed aggregates, then deletes raw JSONL logs to prevent unbounded growth while preserving queryable metrics history.

**Key principles:**
- Archive on workflow completion (transition to `done` node)
- Preserve all aggregate metrics needed for analysis
- Delete raw logs only after successful archive
- Active workflows remain in live JSONL for real-time TUI updates
- Archives are immutable once written

**Scope:** Workflow-scoped archiving for `hooks.jsonl` and `states.jsonl`. Does not manage external transcript files or `command_log.jsonl`.

**Integration context:**
- Triggered by workflow completion in `src/commands/workflow/transitions.rs`
- Consumed by `src/metrics/mod.rs` (parse_unified_metrics)
- Storage layer in `src/storage/archive.rs`

---

## Data Model

### Archive File Format

**Location:** `.hegel/archive/{workflow_id}.json`

**Example:**
```json
{
  "workflow_id": "2025-10-24T10:00:00Z",
  "mode": "discovery",
  "completed_at": "2025-10-24T12:30:00Z",
  "session_id": "abc-123-def",
  "phases": [
    {
      "phase_name": "spec",
      "start_time": "2025-10-24T10:00:00Z",
      "end_time": "2025-10-24T10:15:00Z",
      "duration_seconds": 900,
      "tokens": {
        "input": 5000,
        "output": 2500,
        "cache_creation": 1000,
        "cache_read": 3000,
        "assistant_turns": 5
      },
      "bash_commands": [
        {
          "command": "cargo build",
          "count": 2,
          "timestamps": ["2025-10-24T10:05:00Z", "2025-10-24T10:12:00Z"]
        }
      ],
      "file_modifications": [
        {
          "file_path": "spec.md",
          "tool": "Write",
          "count": 1,
          "timestamps": ["2025-10-24T10:08:00Z"]
        }
      ]
    }
  ],
  "transitions": [
    {
      "from_node": "START",
      "to_node": "spec",
      "timestamp": "2025-10-24T10:00:00Z"
    },
    {
      "from_node": "spec",
      "to_node": "plan",
      "timestamp": "2025-10-24T10:15:00Z"
    }
  ],
  "totals": {
    "tokens": {
      "input": 25000,
      "output": 12000,
      "cache_creation": 5000,
      "cache_read": 15000,
      "assistant_turns": 30
    },
    "bash_commands": 15,
    "file_modifications": 8,
    "unique_files": 5,
    "unique_commands": 7
  }
}
```

**Required fields:**
- `workflow_id` (string, ISO 8601 timestamp from `hegel start`)
- `mode` (string, e.g., "discovery", "execution")
- `completed_at` (string, ISO 8601 timestamp when workflow reached `done`)
- `phases` (array of phase metrics)
- `transitions` (array of state transitions)
- `totals` (aggregate metrics across entire workflow)

**Optional fields:**
- `session_id` (string, if session metadata available)

---

## Core Operations

### 1. Archive Workflow

**Trigger:** Workflow transition to `done` node (detected in `execute_transition`)

**Behavior:**
1. Parse current unified metrics from `.hegel/` directory
2. Compute per-phase aggregates (already done by `build_phase_metrics`)
3. Compute workflow-level totals
4. Serialize to `.hegel/archive/{workflow_id}.json`
5. Use atomic write (temp file + rename) for crash safety
6. On success: delete `.hegel/hooks.jsonl` and `.hegel/states.jsonl`
7. On failure: leave logs intact, log error, continue workflow

**Example usage:**
```rust
// In src/commands/workflow/transitions.rs
if outcome.to_node == "done" {
    archive_workflow(&storage)?;
}
```

**Validation:**
- Workflow must have `workflow_id` in state
- Archive directory must exist (create if missing)
- Must not overwrite existing archive (fail if present)

**Errors:**
- `ArchiveExists`: Archive file already present for this workflow_id
- `ArchiveWriteFailed`: I/O error during archive write
- `MetricsParseError`: Failed to parse metrics for archiving

### 2. Parse Unified Metrics (Modified)

**Behavior:**
1. Read all archives from `.hegel/archive/*.json`
2. Parse live logs from `.hegel/{hooks,states}.jsonl` (if present)
3. Merge archived metrics with live metrics
4. Return unified view across all workflows

**Example usage:**
```rust
// Existing API unchanged
let metrics = parse_unified_metrics(storage.state_dir())?;
// Now returns: archived workflows + current workflow
```

**Merging strategy:**
- Archived workflows: use pre-computed aggregates directly
- Live logs: compute aggregates on-the-fly (existing behavior)
- Session-level view: sum totals across all workflows
- Phase breakdown: show all phases from all workflows

**Performance:**
- Archive reads: O(n) where n = number of completed workflows
- Acceptable for ~100 workflows (<100ms parse time)
- Future optimization: in-memory cache if needed

### 3. Manual Archive Cleanup (Migration Tool)

**Command:** `hegel archive --migrate`

**Behavior:**
1. Parse existing `.hegel/hooks.jsonl`
2. Group events by `workflow_id` (extract from states.jsonl transitions)
3. For each completed workflow (has transition to `done`):
   - Compute aggregates
   - Write archive
   - Remove events from live log
4. Truncate live logs to only current/incomplete workflow

**Validation:**
- Dry-run mode: show what would be archived without modifying files
- Verify all events accounted for (no orphaned events)
- Backup logs before modification

**Errors:**
- `MultipleActiveWorkflows`: Cannot determine current workflow boundary
- `CorruptedLog`: JSONL parse errors
- `BackupFailed`: Could not create backup before migration

---

## Test Scenarios

### Simple

**Scenario 1: Archive single completed workflow**
- Start workflow → transition through phases → reach `done`
- Verify `.hegel/archive/{workflow_id}.json` created
- Verify `hooks.jsonl` and `states.jsonl` deleted
- Verify archive contains all phase metrics
- Verify `hegel analyze` shows archived data

**Scenario 2: Start new workflow after archiving**
- Complete workflow (triggers archive + cleanup)
- Start new workflow
- Verify new `hooks.jsonl` created from scratch
- Verify `hegel analyze` shows both archived + new workflow

### Complex

**Scenario 3: Multiple workflows over time**
- Complete 3 workflows sequentially
- Verify 3 archive files exist
- Verify each archive has correct workflow_id and metrics
- Verify `hegel analyze` aggregates across all 3 archives

**Scenario 4: Archive with transcript tokens**
- Complete workflow with transcript file
- Verify archive includes per-phase token metrics
- Verify totals match sum of phase tokens
- Verify cache metrics preserved

**Scenario 5: TUI during workflow**
- Start workflow, make some progress
- Run `hegel top` (TUI watching live logs)
- Complete workflow (triggers archive)
- Verify TUI doesn't crash when logs deleted
- Start new workflow
- Verify TUI shows new workflow data

### Error

**Scenario 6: Archive write failure**
- Simulate disk full or permission error
- Attempt to archive workflow
- Verify logs NOT deleted
- Verify workflow can still be analyzed from live logs
- Verify error logged but doesn't crash

**Scenario 7: Duplicate archive**
- Manually create archive file
- Complete workflow with same workflow_id
- Verify archiving fails with `ArchiveExists` error
- Verify logs NOT deleted

**Scenario 8: Corrupted live logs during migration**
- Create `.hegel/hooks.jsonl` with invalid JSON line
- Run `hegel archive --migrate`
- Verify error reported with line number
- Verify no files modified (atomic migration)

**Scenario 9: Abandoned workflow**
- Start workflow, do not complete (never reach `done`)
- Start new workflow
- Verify incomplete workflow NOT archived
- Verify logs from incomplete workflow preserved (no data loss)

### Integration

**Scenario 10: Archive compatibility with existing analysis**
- Create archive with various metrics
- Run `hegel analyze`
- Verify all sections render correctly (tokens, bash commands, files, phases, graph)
- Verify DAG reconstruction works from archived transitions

**Scenario 11: TUI with mixed archived + live data**
- Complete 1 workflow (archived)
- Start new workflow (live)
- Run `hegel top`
- Verify Overview tab shows totals across both
- Verify Phases tab shows phases from both workflows
- Verify Events tab shows only live workflow events

---

## Success Criteria

### Core Functionality
- [ ] Workflows archive automatically on completion (transition to `done`)
- [ ] Archive files are valid JSON with all required fields
- [ ] Live logs deleted only after successful archive write
- [ ] Failed archive attempts leave logs intact

### Data Preservation
- [ ] All per-phase metrics preserved in archives (tokens, bash commands, file mods, duration)
- [ ] Workflow-level totals match sum of phase metrics
- [ ] State transitions preserved for DAG reconstruction
- [ ] Session metadata preserved when available

### Query Compatibility
- [ ] `hegel analyze` shows archived + live workflows
- [ ] Per-phase breakdown includes all workflows
- [ ] Token totals aggregate correctly across archives
- [ ] Top commands/files include data from archives

### Error Handling
- [ ] Archive write failures logged with context
- [ ] Duplicate archives detected and rejected
- [ ] Corrupted archives skipped with warning (don't crash analysis)
- [ ] Partial migration failures leave system in consistent state

### Performance
- [ ] Parsing 100 archives completes in <100ms
- [ ] TUI remains responsive with 10+ archives
- [ ] Archive file sizes reasonable (~10-50KB per workflow)

### Migration Tool
- [ ] `hegel archive --migrate` successfully splits existing logs
- [ ] Dry-run mode shows preview without modifying files
- [ ] Backup created before migration
- [ ] All events accounted for (no orphaned events)

### Edge Cases
- [ ] Abandoned workflows (never reach `done`) are not archived
- [ ] Multiple sessions in one workflow handled correctly
- [ ] Workflows with no transcript still archive successfully
- [ ] Empty phases (no hooks) don't break archiving

---

## Security and Validation

**Archive integrity:**
- Archive files are append-only (never modified after creation)
- Use atomic writes (temp file + rename) to prevent corruption
- Workflow_id must match filename (e.g., `2025-10-24T10:00:00Z.json`)

**Input validation:**
- Reject workflow_id with path traversal characters (`/`, `..`)
- Validate ISO 8601 timestamp format for workflow_id
- Archive directory must be within `.hegel/` (no symlink escape)

**Error recovery:**
- Failed archive writes leave original logs intact
- Partial archives are never left on disk (atomic write)
- Corrupted archives are skipped during analysis (fail gracefully)

---

## Performance and Scaling

**Current scale:** 33MB hooks.jsonl → ~1000 workflow archives at 30KB each

**Read performance:**
- Archive parsing: ~1ms per archive (JSON deserialize)
- 100 archives: ~100ms total (acceptable for CLI)
- 1000 archives: ~1s (may need optimization)

**Future optimization (if needed):**
- In-memory cache of recent archives
- Lazy loading (only parse archives when requested)
- Archive index file (metadata-only for fast listing)

**Disk usage:**
- Archives compress well (JSON text)
- Expect 70-80% reduction vs raw JSONL
- 33MB hooks.jsonl → ~7-10MB total archives

---

## Conclusion

This spec defines a simple, robust archiving system that preserves all metrics while preventing unbounded log growth. Archives are immutable, queryable, and backward-compatible with existing analysis tools. Implementation is straightforward: serialize aggregates on completion, delete raw logs, merge on read.
