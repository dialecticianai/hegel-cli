# State & Metrics Subsystem Onboarding

Quick-start guide for working on `hegel [analyze|archive|doctor]` and the state/metrics pipeline.

---

## Mental Model

**Three-phase lifecycle:**
1. **Live capture** → JSONL event logs (`hooks.jsonl`, `states.jsonl`) + transcript files
2. **Archive** → Terminal node reached → compress to `.hegel/archive/{workflow_id}.json` + delete logs
3. **Analysis** → Parse archives + live logs → aggregate metrics → repair/visualize

**Key insight:** Archives are pre-computed aggregates. Live metrics parse JSONL on-demand. Never mix them (duplication bug).

---

## Data Flow

```
Session activity
    ↓
hooks.jsonl (bash, file mods) + states.jsonl (transitions) + transcript.jsonl (tokens)
    ↓
[hegel next → done node]
    ↓
archive_and_cleanup() → parse_unified_metrics(include_archives=false) → WorkflowArchive
    ↓
.hegel/archive/{workflow_id}.json + DELETE logs
    ↓
[hegel analyze]
    ↓
parse_unified_metrics(include_archives=true) → UnifiedMetrics → render sections
```

**Critical:** `parse_unified_metrics(include_archives=false)` when creating archives to prevent counting twice.

---

## Key Files

**Commands:**
- `src/commands/workflow/transitions.rs:470-476` - Auto-archive on terminal node transition
- `src/commands/archive.rs` - Manual log migration (`hegel archive --migrate`)
- `src/commands/analyze/mod.rs` - Analysis entry point (delegates to `src/analyze/`)

**Analysis/Repair:**
- `src/analyze/repair.rs` - Orchestrates archive repairs via cleanup traits
- `src/analyze/gap_detection.rs` - Creates synthetic cowboy workflows for inter-workflow gaps
- `src/analyze/cleanup/` - Trait-based repair strategies (git backfill, aborted nodes, duplicates)

**Metrics Parsing:**
- `src/metrics/mod.rs:155` - `parse_unified_metrics()` - Central parser (controls archive inclusion)
- `src/metrics/aggregation.rs` - `build_phase_metrics()` - Phase boundary detection + token attribution
- `src/metrics/hooks.rs` - Parse `hooks.jsonl` → bash commands, file modifications
- `src/metrics/states.rs` - Parse `states.jsonl` → state transitions (phase boundaries)
- `src/metrics/transcript.rs` - Parse transcript files → token metrics
- `src/metrics/git.rs` - Parse git commits → attribute to phases by timestamp

**Storage:**
- `src/storage/archive/mod.rs` - Archive I/O (`read_archives`, `write_archive`)
- `src/storage/archive/builder.rs` - `WorkflowArchive::from_metrics()`
- `src/storage/archive/aggregation.rs` - DRY helpers for totals computation
- `src/storage/log_cleanup.rs` - Delete JSONL logs after archiving

**Doctor:**
- `src/commands/doctor/mod.rs` - State health check + rescue corrupted `state.json`
- `src/doctor/migrations/` - Schema migrations for `state.json` evolution

---

## Critical Paths

### Auto-Archive on Terminal Node

**Trigger:** `hegel next` transitions to `done`/`aborted`/`cancelled` node

**Path:** `transitions.rs:execute_transition()` → `IntraWorkflow` match arm → `is_terminal(to_node)` → `archive_and_cleanup()`

**archive_and_cleanup() steps:**
1. Parse metrics WITHOUT archives: `parse_unified_metrics(state_dir, false, None)`
2. Get `workflow_id` from `state.json`
3. Parse git commits and attribute to phases (only during archiving)
4. Build archive: `WorkflowArchive::from_metrics(&metrics, &workflow_id, false)`
5. Write: `write_archive(&archive, state_dir)`
6. Update cumulative totals in `state.json`
7. **Delete logs:** `cleanup_logs(state_dir)` removes JSONL files

### Repair Archives (`hegel analyze --fix-archives`)

**Path:** `analyze/repair.rs:repair_archives()`

**Steps:**
1. Read all archives: `read_archives(state_dir)`
2. For each archive, check cleanups via `ArchiveCleanup` trait:
   - `GitBackfillCleanup` - adds missing git commits
   - `AbortedNodeCleanup` - adds terminal node to incomplete workflows
   - `DuplicateCowboyCleanup` - removes duplicate synthetic cowboys
3. Apply repairs: `cleanup.repair(archive, state_dir, dry_run)`
4. Post-process: batch operations (e.g., duplicate removal)
5. **Gap detection:** `ensure_cowboy_coverage()` - creates synthetic cowboys for gaps with git activity
6. Rebuild cumulative totals: `rebuild_cumulative_totals(storage, &archives)`

### Parse Unified Metrics

**Signature:** `parse_unified_metrics(state_dir, include_archives: bool, debug_config)`

**When `include_archives=false` (default):**
- Parse live JSONL logs only (`hooks.jsonl`, `states.jsonl`)
- Discover transcript files via `list_transcript_files()` (Claude Code project dir)
- Build phase metrics: `build_phase_metrics()` aggregates by phase boundaries

**When `include_archives=true` (analysis only):**
- Also load `.hegel/archive/*.json`
- Merge archived phases with live phases
- WARNING: Never use during archiving (duplication bug)

### Token Attribution

**How tokens get attributed to phases:**

1. **Phase boundaries** from `states.jsonl`:
   ```rust
   Phase "spec": 2025-11-10T10:00:00Z → 2025-11-10T10:15:00Z
   Phase "plan": 2025-11-10T10:15:00Z → 2025-11-10T10:30:00Z
   ```

2. **Transcript files** discovered via `list_transcript_files()`:
   - Looks for `~/.claude/projects/-{normalized-repo-path}/*.jsonl`
   - Multi-session support: aggregates tokens across all session transcripts

3. **Token aggregation** in `aggregate_tokens_for_range()`:
   - Filters transcript events by timestamp: `event.timestamp ∈ [phase_start, phase_end)`
   - Sums input/output/cache tokens per phase
   - Returns `(TokenMetrics, examined_count, matched_count)`

**Git commits** attributed separately via `git::attribute_commits_to_phases()` (timestamp-based bucketing).

### Cowboy Workflow Synthesis

**Purpose:** Capture work done outside explicit workflows (inter-workflow gaps with git activity)

**Detection:** `gap_detection.rs:detect_and_archive_cowboy_activity()`

**Algorithm:**
1. Find most recent real (non-synthetic) archive
2. Define gap: `[prev_workflow.completed_at, current_timestamp)`
3. Check for git commits OR uncommitted changes in gap
4. If activity found: create synthetic cowboy archive with `is_synthetic=true`
5. Delete logs to prevent re-detection

**Repair:** `gap_detection.rs:ensure_cowboy_coverage()`
- Ensures exactly one cowboy per gap between real workflows
- Only creates cowboys for gaps with git activity
- Removes duplicate/incorrect cowboys

---

## Debugging Checklist

### "Tokens not captured in archive"

1. Check transcript discovery: `list_transcript_files(repo_path)` returning files?
2. Verify timestamps align: phase boundaries vs transcript events
3. Debug token attribution: `hegel analyze --debug START..END --verbose`
4. Check if archives included during archiving: `parse_unified_metrics(_, false, _)` must be false

### "Git commits missing from archive"

1. Git attribution happens ONLY during archiving (not live metrics)
2. Check `archive_and_cleanup():333-353` - git parsing block
3. Repair: `hegel analyze --fix-archives` runs `GitBackfillCleanup`

### "Duplicate activity in archives"

1. Archive included during archiving? Check `parse_unified_metrics` call
2. Multiple cowboys for same gap? Run `hegel analyze --fix-archives`
3. Check cleanup_logs() deleted JSONL after archiving

### "Archive command not working"

1. `hegel archive` is for **migration** of old multi-workflow logs (legacy)
2. Normal path: auto-archive via `hegel next` to terminal node
3. Manual repair: `hegel analyze --fix-archives`

### "State file corrupted"

1. Run `hegel doctor` - attempts rescue via backup/reconstruction
2. Check `.hegel/state.json.backup` for previous good state
3. Migrations applied via `doctor/migrations/` on `hegel doctor`

---

## File Structure Reference

```
.hegel/
├── state.json              # Current workflow state + session metadata + cumulative totals
├── hooks.jsonl            # Live event log (deleted after archive)
├── states.jsonl           # Live state transitions (deleted after archive)
├── archive/               # Compressed workflow archives
│   ├── 2025-11-10T10:00:00Z.json  # Real workflow archive
│   └── 2025-11-10T12:30:00Z.json  # Synthetic cowboy archive (is_synthetic=true)
└── reviews.json           # Reflect GUI review feedback (unrelated to metrics)

~/.claude/projects/-{repo-path}/
├── {session-id}.jsonl     # Transcript with token usage
└── ...                    # Multiple sessions supported
```

---

## Common Patterns

**Reading archives:**
```rust
use crate::storage::archive::read_archives;
let archives = read_archives(state_dir)?;
// Always sorted by workflow_id (chronological)
```

**Parsing metrics (analysis):**
```rust
use crate::metrics::parse_unified_metrics;
let metrics = parse_unified_metrics(state_dir, true, None)?; // include_archives=true
```

**Parsing metrics (archiving):**
```rust
let metrics = parse_unified_metrics(state_dir, false, None)?; // include_archives=false !!!
```

**Creating archive:**
```rust
use crate::storage::archive::{write_archive, WorkflowArchive};
let archive = WorkflowArchive::from_metrics(&metrics, &workflow_id, false)?; // is_synthetic=false
write_archive(&archive, state_dir)?;
```

**Cleanup trait implementation:**
```rust
impl ArchiveCleanup for MyCleanup {
    fn name(&self) -> &str { "my_cleanup" }
    fn needs_repair(&self, archive: &WorkflowArchive) -> bool { /* detect issue */ }
    fn repair(&self, archive: &mut WorkflowArchive, state_dir: &Path, dry_run: bool) -> Result<bool> {
        if dry_run { return Ok(false); }
        // mutate archive in-place
        Ok(true) // return true if modified
    }
}
```

---

## Quick Commands

```bash
# Analyze current metrics (live + archived)
hegel analyze --full

# Repair archives (backfill git, create cowboys, remove duplicates)
hegel analyze --fix-archives

# Dry-run repair (see what would change)
hegel analyze --fix-archives --dry-run

# Debug token attribution for specific time range
hegel analyze --debug 2025-11-10T10:00:00Z..2025-11-10T11:00:00Z --verbose

# Check state health
hegel doctor

# Migrate old logs to archives (legacy, rarely needed)
hegel archive --migrate
```

---

## Testing Patterns

See `src/test_helpers/` for utilities:
- `create_hooks_file()` - Generate test `hooks.jsonl`
- `create_states_file()` - Generate test `states.jsonl`
- `create_transcript_file()` - Generate test transcript
- `test_storage_with_files()` - Full test setup with logs

Example:
```rust
use crate::test_helpers::*;

let hooks = vec![/* JSON strings */];
let states = vec![/* JSON strings */];
let (_temp, storage) = test_storage_with_files(Some(&hooks), Some(&states));

let metrics = parse_unified_metrics(storage.state_dir(), false, None)?;
assert_eq!(metrics.phase_metrics.len(), 2);
```
