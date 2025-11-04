# Log Retention Implementation Plan

Stepwise TDD implementation of workflow archiving with aggregate preservation.

---

## Overview

**Goal:** Implement automatic workflow archiving on completion, with pre-computed aggregates stored and raw logs cleaned up.

**Scope:**
- Archive serialization/deserialization (new module)
- Archive trigger on workflow completion
- Modified metrics parsing to read archives + live logs
- Migration tool for existing logs

**Priorities:**
1. Data safety (never lose metrics)
2. Backward compatibility (existing analysis tools work)
3. Performance (parsing 100 archives < 100ms)

**Methodology:**
- TDD: Write tests first for each step
- Red → Green → Commit
- Integration tests validate end-to-end flow
- Error cases tested explicitly

---

## Step 1: Archive Data Model and Serialization

### Goal
Define archive format and implement serialization/deserialization with validation.

### Step 1.a: Write Tests
**Test file:** `src/storage/archive.rs` (inline tests)

**Key test cases:**
- Serialize workflow archive with all fields
- Deserialize archive from JSON
- Round-trip serialization (serialize → deserialize → verify equal)
- Validation: reject invalid workflow_id (path traversal, invalid ISO 8601)
- Validation: reject missing required fields
- Handle optional fields (session_id can be None)

**Expected behavior:**
- Archive struct matches SPEC.md format
- Validation errors provide clear messages
- Serde serialization produces clean JSON

### Step 1.b: Implement
**Tasks:**
1. Create `src/storage/archive.rs`
2. Define `WorkflowArchive` struct with required/optional fields
3. Define nested structs: `PhaseArchive`, `TransitionArchive`, `TokenTotals`, `BashCommandSummary`, `FileModificationSummary`
4. Implement `serde` derives (Serialize, Deserialize)
5. Add validation methods: `validate_workflow_id()`, `validate_archive_path()`
6. Implement `from_metrics()` constructor (converts UnifiedMetrics → WorkflowArchive)

**Code pattern:**
```rust
#[derive(Serialize, Deserialize)]
pub struct WorkflowArchive {
    pub workflow_id: String,
    pub mode: String,
    pub completed_at: String,
    pub phases: Vec<PhaseArchive>,
    pub transitions: Vec<TransitionArchive>,
    pub totals: WorkflowTotals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl WorkflowArchive {
    pub fn from_metrics(metrics: &UnifiedMetrics, workflow_id: &str) -> Result<Self> {
        validate_workflow_id(workflow_id)?;
        // Build archive from metrics...
    }
}
```

**Error handling:**
- Return `Err` for invalid workflow_id (contains `/`, `..`, or invalid ISO 8601)
- Return `Err` if required fields missing from metrics

### Success Criteria
- [ ] `WorkflowArchive` struct defined with all SPEC.md fields
- [ ] Serde serialization produces valid JSON
- [ ] Round-trip serialization passes
- [ ] Invalid workflow_id validation works
- [ ] `from_metrics()` converts UnifiedMetrics correctly
- [ ] All tests pass

---

## Step 2: Archive Writing with Atomic Operations

### Goal
Implement safe archive writing to `.hegel/archive/` with atomic file operations.

### Step 2.a: Write Tests
**Test file:** `src/storage/archive.rs` (tests module)

**Key test cases:**
- Write archive to temporary directory
- Verify archive file created at correct path
- Verify atomic write (temp file + rename)
- Error: archive already exists (reject duplicate)
- Error: permission denied (I/O error handling)
- Verify archive directory created if missing

**Expected behavior:**
- Archive written to `.hegel/archive/{workflow_id}.json`
- File uses atomic write (no partial files on crash)
- Duplicate archives rejected with clear error

### Step 2.b: Implement
**Tasks:**
1. Add `write_archive()` function to `src/storage/archive.rs`
2. Create `.hegel/archive/` directory if missing
3. Check for existing archive (return `Err(ArchiveExists)`)
4. Use temp file + rename for atomic write (similar to `FileStorage::save()`)
5. Serialize archive to JSON with pretty formatting
6. Add custom error types: `ArchiveError::ArchiveExists`, `ArchiveError::WriteFailed`

**Code pattern:**
```rust
pub fn write_archive(archive: &WorkflowArchive, state_dir: &Path) -> Result<()> {
    let archive_dir = state_dir.join("archive");
    fs::create_dir_all(&archive_dir)?;

    let archive_path = archive_dir.join(format!("{}.json", archive.workflow_id));

    if archive_path.exists() {
        bail!("Archive already exists: {}", archive.workflow_id);
    }

    // Atomic write: temp file + rename
    let temp_path = archive_path.with_extension("tmp");
    let json = serde_json::to_string_pretty(archive)?;
    fs::write(&temp_path, json)?;
    fs::rename(&temp_path, &archive_path)?;

    Ok(())
}
```

**Error handling:**
- Check for existing archive before writing
- Clean up temp file on error
- Propagate I/O errors with context

### Success Criteria
- [ ] Archive written to correct path
- [ ] Atomic write prevents partial files
- [ ] Duplicate archives rejected
- [ ] Archive directory auto-created
- [ ] I/O errors propagated with context
- [ ] All tests pass

---

## Step 3: Archive Reading and Merging

### Goal
Modify `parse_unified_metrics` to read archives and merge with live logs.

### Step 3.a: Write Tests
**Test file:** `src/metrics/mod.rs` (tests module)

**Key test cases:**
- Parse metrics with no archives (existing behavior)
- Parse metrics with 1 archived workflow + no live logs
- Parse metrics with 1 archived workflow + 1 live workflow
- Parse metrics with 3 archived workflows
- Aggregate totals across archived + live workflows
- Skip corrupted archive with warning (don't crash)
- Handle empty archive directory

**Expected behavior:**
- Archived workflows loaded from `.hegel/archive/*.json`
- Phase metrics include all workflows
- Totals aggregate correctly
- Corrupted archives skipped gracefully

### Step 3.b: Implement
**Tasks:**
1. Add `read_archives()` function to `src/storage/archive.rs`
2. Modify `parse_unified_metrics()` in `src/metrics/mod.rs`
3. Read all `.json` files from `.hegel/archive/`
4. Deserialize each archive (skip invalid with warning)
5. Merge archived phase metrics with live phase metrics
6. Aggregate totals across all workflows
7. Log warnings for corrupted archives (don't fail)

**Code pattern:**
```rust
pub fn read_archives(state_dir: &Path) -> Result<Vec<WorkflowArchive>> {
    let archive_dir = state_dir.join("archive");
    if !archive_dir.exists() {
        return Ok(Vec::new());
    }

    let mut archives = Vec::new();
    for entry in fs::read_dir(&archive_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "json") {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(archive) => archives.push(archive),
                    Err(e) => eprintln!("Warning: skipping corrupted archive {:?}: {}", path, e),
                },
                Err(e) => eprintln!("Warning: failed to read archive {:?}: {}", path, e),
            }
        }
    }

    Ok(archives)
}
```

**Merging strategy:**
- Archived workflows contribute to totals
- Phase metrics list includes archived + live phases
- Session-level view sums across all workflows

### Success Criteria
- [ ] Archives read from `.hegel/archive/`
- [ ] Corrupted archives skipped with warning
- [ ] Phase metrics include archived workflows
- [ ] Totals aggregate correctly
- [ ] Backward compatible (no archives = existing behavior)
- [ ] All tests pass

---

## Step 4: Trigger Archiving on Workflow Completion

### Goal
Automatically archive workflow when transitioning to `done` node and delete raw logs.

### Step 4.a: Write Tests
**Test file:** `src/commands/workflow/tests/transitions.rs`

**Key test cases:**
- Transition to `done` triggers archiving
- Archive file created with correct workflow_id
- `hooks.jsonl` deleted after successful archive
- `states.jsonl` deleted after successful archive
- Archive failure leaves logs intact (no deletion)
- Non-done transitions don't trigger archiving
- Abandoned workflow (no `done` transition) not archived

**Expected behavior:**
- Archiving happens synchronously during transition
- Logs deleted only after successful archive write
- Failed archives logged but don't block workflow

### Step 4.b: Implement
**Tasks:**
1. Add `archive_and_cleanup()` function to `src/commands/workflow/transitions.rs`
2. Call from `execute_transition()` when `to_node == "done"`
3. Parse current metrics, convert to archive
4. Write archive using `write_archive()`
5. On success: delete `.hegel/hooks.jsonl` and `.hegel/states.jsonl`
6. On failure: log error, leave logs intact, continue workflow
7. Expose `archive_workflow()` in `src/commands/mod.rs`

**Code pattern:**
```rust
fn archive_and_cleanup(storage: &FileStorage) -> Result<()> {
    let state_dir = storage.state_dir();

    // Parse metrics
    let metrics = parse_unified_metrics(state_dir)?;

    // Get workflow_id from state
    let state = storage.load()?;
    let workflow_id = state.workflow_state
        .and_then(|ws| ws.workflow_id)
        .context("No workflow_id for archiving")?;

    // Create archive
    let archive = WorkflowArchive::from_metrics(&metrics, &workflow_id)?;

    // Write archive
    write_archive(&archive, state_dir)?;

    // Delete logs on success
    let hooks_path = state_dir.join("hooks.jsonl");
    let states_path = state_dir.join("states.jsonl");
    if hooks_path.exists() { fs::remove_file(hooks_path)?; }
    if states_path.exists() { fs::remove_file(states_path)?; }

    Ok(())
}

// In execute_transition()
if to_node == "done" {
    if let Err(e) = archive_and_cleanup(storage) {
        eprintln!("Warning: archiving failed: {}", e);
        // Continue workflow despite archive failure
    }
}
```

**Error handling:**
- Log archive failures but don't block workflow completion
- Never delete logs if archive write failed
- Provide clear error context

### Success Criteria
- [ ] Transition to `done` triggers archiving
- [ ] Archive file created successfully
- [ ] Logs deleted after successful archive
- [ ] Failed archives don't delete logs
- [ ] Non-done transitions skip archiving
- [ ] All tests pass

---

## Step 5: Integration Testing

### Goal
End-to-end validation of archiving workflow across multiple cycles.

### Step 5.a: Write Tests
**Test file:** `src/commands/workflow/tests/integration.rs`

**Key test cases:**
- Complete 3 workflows sequentially, verify 3 archives
- Start workflow after archiving, verify fresh logs
- Run `hegel analyze` on archived workflows
- Run `hegel analyze` on mixed archived + live
- TUI compatibility (manual verification)
- Archive → restart → archive (verify isolated workflows)

**Expected behavior:**
- Multiple workflows archive independently
- Analysis works across archived + live data
- Fresh workflow starts with empty logs

### Step 5.b: Implement
**Tasks:**
1. Add integration tests to workflow test suite
2. Test helper: `complete_workflow()` (start → transition to done)
3. Verify archive count and contents
4. Verify `parse_unified_metrics()` aggregates correctly
5. Manual TUI test: complete workflow while `hegel top` running

**Integration test pattern:**
```rust
#[test]
fn test_multi_workflow_archiving() {
    let (temp_dir, storage) = setup_workflow_env();

    // Workflow 1
    start_workflow("discovery", &storage).unwrap();
    transition_to_done(&storage);
    assert_eq!(count_archives(&temp_dir), 1);
    assert!(!hooks_file_exists(&temp_dir));

    // Workflow 2
    start_workflow("execution", &storage).unwrap();
    transition_to_done(&storage);
    assert_eq!(count_archives(&temp_dir), 2);

    // Verify aggregation
    let metrics = parse_unified_metrics(storage.state_dir()).unwrap();
    assert_eq!(metrics.phase_metrics.len(), 10); // 5 phases × 2 workflows
}
```

### Success Criteria
- [ ] Multiple workflows archive correctly
- [ ] Fresh workflows start with clean logs
- [ ] Analysis aggregates across archives
- [ ] TUI doesn't crash during archiving
- [ ] All integration tests pass

---

## Step 6: Migration Tool (Optional Manual Command)

### Goal
Provide manual tool to migrate existing large logs to archives.

### Step 6.a: Write Tests
**Test file:** `src/commands/archive.rs` (new file)

**Key test cases:**
- Migrate multi-workflow hooks.jsonl
- Verify archives created for completed workflows
- Verify incomplete workflow remains in live log
- Dry-run shows preview without modification
- Backup created before migration
- Corrupted log handled gracefully

**Expected behavior:**
- Parse logs, group by workflow_id, archive completed
- Dry-run mode for safety
- Backup created automatically

### Step 6.b: Implement
**Tasks:**
1. Create `src/commands/archive.rs`
2. Add `hegel archive --migrate` subcommand
3. Add `--dry-run` flag for preview
4. Parse hooks.jsonl and states.jsonl
5. Group events by workflow_id (from states transitions)
6. For completed workflows: create archive, remove events
7. Create backup before modification
8. Wire up in main.rs CLI

**Code pattern:**
```rust
pub fn migrate_logs(storage: &FileStorage, dry_run: bool) -> Result<()> {
    let state_dir = storage.state_dir();

    // Backup
    if !dry_run {
        backup_logs(state_dir)?;
    }

    // Parse and group
    let workflows = group_by_workflow(state_dir)?;

    for (workflow_id, events) in workflows {
        if is_completed(&events) {
            if dry_run {
                println!("Would archive: {}", workflow_id);
            } else {
                archive_workflow_events(&events, state_dir)?;
            }
        }
    }

    Ok(())
}
```

### Success Criteria
- [ ] Multi-workflow logs migrated correctly
- [ ] Incomplete workflows preserved
- [ ] Dry-run mode works
- [ ] Backup created automatically
- [ ] All tests pass

---

## Performance Validation

**Step 7: Benchmark Archive Parsing**

### Goal
Validate that parsing 100 archives meets <100ms requirement.

### Tasks
1. Create benchmark test with 100 mock archives
2. Measure `parse_unified_metrics()` execution time
3. Verify < 100ms on average hardware
4. Profile if slow, optimize if needed

**Code pattern:**
```rust
#[test]
fn bench_parse_100_archives() {
    let temp_dir = create_100_mock_archives();

    let start = Instant::now();
    let _metrics = parse_unified_metrics(temp_dir.path()).unwrap();
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100, "Parse took {:?}", duration);
}
```

### Success Criteria
- [ ] 100 archives parse in <100ms
- [ ] Performance acceptable for expected scale

---

## Security Validation

**Step 8: Path Traversal and Validation**

### Goal
Ensure archive paths cannot escape `.hegel/archive/`.

### Tasks
1. Test path traversal in workflow_id (`../../../etc/passwd`)
2. Test symlink escape attempts
3. Verify validation rejects malicious inputs
4. Test archive_dir is always within state_dir

### Success Criteria
- [ ] Path traversal rejected
- [ ] Symlink escapes prevented
- [ ] All validation tests pass

---

## Commit Discipline

After each numbered step completes:
```bash
git add .
git commit -m "feat(archive): complete Step N - <description>

<details of what was implemented>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

Example:
- Step 1: `feat(archive): complete Step 1 - archive data model and serialization`
- Step 2: `feat(archive): complete Step 2 - atomic archive writing`
- Step 3: `feat(archive): complete Step 3 - archive reading and merging`

---

## Conclusion

This plan implements workflow archiving through incremental TDD steps. Each step builds on the previous, with comprehensive test coverage and clear success criteria. The implementation prioritizes data safety, backward compatibility, and performance while maintaining clean architecture.

Implementation order ensures:
1. Data model stable before I/O
2. Reading/writing work before automation
3. Integration validated before optimization
4. Migration tool last (optional for users)

Estimated completion: 1-2 sessions with rigorous testing.
