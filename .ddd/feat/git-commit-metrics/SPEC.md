# Git Commit Metrics Specification

Automatic git commit tracking and attribution within Hegel workflow phase metrics.

---

## Overview

### What it does
Extends the `hegel analyze` command to automatically detect git repositories and correlate commit activity with workflow phases, providing per-phase commit statistics (count, files changed, insertions, deletions) when `.git/` exists alongside `.hegel/`.

### Key principles
- **Zero configuration**: Automatic detection and activation when `.git/` directory present
- **Phase attribution**: Commits mapped to workflow phases via timestamp correlation
- **Session-scoped**: Track all commits from session start to analysis time
- **Graceful degradation**: Non-git projects continue working without errors
- **Stats-rich**: Capture meaningful commit metadata beyond simple counts

### Scope
Production feature integrating git metadata into existing metrics pipeline. Extends `UnifiedMetrics` and `PhaseMetrics` data structures, adds git log parser, updates analyze command output.

### Integration context
- Integrates with: `src/metrics/mod.rs` (UnifiedMetrics), `src/commands/analyze/sections.rs` (Phase Breakdown rendering)
- Reads from: Git repository history via `git2` crate (libgit2 Rust bindings)
- Outputs to: Terminal via enhanced Phase Breakdown section in `hegel analyze`
- Dependencies: Add `git2` crate to Cargo.toml

---

## Data Model

### GitCommit
```rust
pub struct GitCommit {
    /// Commit SHA (abbreviated, 7 chars)
    pub hash: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Commit message (first line only)
    pub message: String,
    /// Author name
    pub author: String,
    /// Number of files changed
    pub files_changed: usize,
    /// Lines inserted
    pub insertions: usize,
    /// Lines deleted
    pub deletions: usize,
}
```

**Example**:
```json
{
  "hash": "a310c04",
  "timestamp": "2025-11-02T17:50:00Z",
  "message": "fix(lib): include test_helpers module for test compilation",
  "author": "Emily Madum",
  "files_changed": 4,
  "insertions": 21,
  "deletions": 15
}
```

### PhaseMetrics Extension
```rust
pub struct PhaseMetrics {
    // ... existing fields ...

    /// Git commits attributed to this phase
    pub git_commits: Vec<GitCommit>,
}
```

### UnifiedMetrics Extension
```rust
pub struct UnifiedMetrics {
    // ... existing fields ...

    /// All git commits in session scope (not phase-specific)
    pub git_commits: Vec<GitCommit>,
}
```

---

## Core Operations

### Git Repository Detection

**Function**: `has_git_repository(state_dir: &Path) -> bool`

**Implementation**:
```rust
use git2::Repository;

fn has_git_repository(state_dir: &Path) -> bool {
    let project_root = match state_dir.parent() {
        Some(p) => p,
        None => return false,
    };

    Repository::open(project_root).is_ok()
}
```

**Behavior**:
- Attempts to open git repository at project root (parent of `.hegel/`)
- Returns `true` if `git2::Repository::open()` succeeds
- Returns `false` for any error (no .git, corrupt repo, permissions, etc.)
- Never panics or propagates errors

**Examples**:
```rust
// Project structure: /project/.hegel/ and /project/.git/
has_git_repository(Path::new("/project/.hegel")) // -> true

// No git repo
has_git_repository(Path::new("/project/.hegel")) // -> false
```

---

### Git Commit Parsing

**Function**: `parse_git_commits(project_root: &Path, since: Option<i64>) -> Result<Vec<GitCommit>>`

**Parameters**:
- `project_root`: Path to project root (where `.git/` exists)
- `since`: Optional Unix timestamp to filter commits (e.g., session start time)

**Implementation approach** (using `git2` crate):
```rust
use git2::{Repository, Commit, DiffOptions};
use chrono::{DateTime, Utc};

fn parse_git_commits(project_root: &Path, since: Option<i64>) -> Result<Vec<GitCommit>> {
    let repo = Repository::open(project_root)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut commits = Vec::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Filter by timestamp if provided
        let commit_time = commit.time().seconds();
        if let Some(since_time) = since {
            if commit_time < since_time {
                continue;
            }
        }

        // Get diff stats
        let stats = get_commit_stats(&repo, &commit)?;

        // Convert to GitCommit
        let git_commit = GitCommit {
            hash: format!("{:.7}", oid),
            timestamp: timestamp_to_iso8601(commit_time),
            message: commit.message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        };

        commits.push(git_commit);
    }

    Ok(commits)
}

fn get_commit_stats(repo: &Repository, commit: &Commit) -> Result<git2::DiffStats> {
    let old_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(
        old_tree.as_ref(),
        Some(&commit.tree()?),
        None
    )?;

    Ok(diff.stats()?)
}
```

**Error Handling**:
- Repository open failure: Return `Err` (caller handles gracefully)
- Corrupt commits: Skip and continue iteration
- Missing metadata (author, message): Use empty string defaults
- Root commits (no parent): Diff against empty tree

**Examples**:
```rust
// Get all commits since session start (Unix timestamp)
let since = 1730563200; // 2025-11-02T10:00:00Z
let commits = parse_git_commits(
    Path::new("/project"),
    Some(since)
)?;

// Get all commits (no filter)
let commits = parse_git_commits(Path::new("/project"), None)?;
```

---

### Phase Attribution

**Function**: `attribute_commits_to_phases(commits: Vec<GitCommit>, phases: &mut [PhaseMetrics])`

**Behavior**:
- For each commit, find the phase whose time range contains the commit timestamp
- Phase time range: `[start_time, end_time]` (or `[start_time, now]` for active phases)
- Commits outside all phase ranges are ignored (per user requirement)
- Mutates `phase.git_commits` in place

**Algorithm**:
```rust
for commit in commits {
    for phase in phases {
        if commit.timestamp >= phase.start_time
           && (phase.end_time.is_none() || commit.timestamp <= phase.end_time) {
            phase.git_commits.push(commit.clone());
            break; // Commit attributed to earliest matching phase
        }
    }
}
```

**Edge Cases**:
- Commit exactly at phase boundary: Attributed to earlier phase
- Multiple phases overlap (shouldn't happen): First match wins
- No phases defined: All commits ignored

---

## Integration Points

### Metrics Module (`src/metrics/mod.rs`)

**Add to `parse_unified_metrics`**:
```rust
use chrono::DateTime;

pub fn parse_unified_metrics(state_dir: &Path) -> Result<UnifiedMetrics> {
    // ... existing parsing ...

    // Git commit parsing (with git2 crate)
    let git_commits = if has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();

        // Convert session start time from ISO 8601 to Unix timestamp
        let since_timestamp = session_start_time
            .as_ref()
            .and_then(|iso| DateTime::parse_from_rfc3339(iso).ok())
            .map(|dt| dt.timestamp());

        parse_git_commits(project_root, since_timestamp)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse git commits: {}", e);
                Vec::new()
            })
    } else {
        Vec::new()
    };

    // Attribute commits to phases
    attribute_commits_to_phases(git_commits.clone(), &mut phase_metrics);

    Ok(UnifiedMetrics {
        // ... existing fields ...
        git_commits,
    })
}
```

---

### Analyze Command Rendering (`src/commands/analyze/sections.rs`)

**Update `render_phase_breakdown`**:
```rust
pub fn render_phase_breakdown(phase_metrics: &[PhaseMetrics]) {
    // ... existing rendering ...

    // Git commits (new section)
    if !phase.git_commits.is_empty() {
        let total_insertions: usize = phase.git_commits.iter()
            .map(|c| c.insertions).sum();
        let total_deletions: usize = phase.git_commits.iter()
            .map(|c| c.deletions).sum();
        let total_files: usize = phase.git_commits.iter()
            .map(|c| c.files_changed).sum();

        println!(
            "    Commits:           {} ({} files, +{} -{} lines)",
            format_metric(phase.git_commits.len()),
            total_files,
            total_insertions,
            total_deletions
        );
    } else {
        println!("    Commits:           {}", Theme::secondary("-"));
    }
}
```

**Example Output**:
```
SPEC (completed)
  Duration:          15m 23s
  Tokens:            12,450 (10,200 in, 2,250 out)
  Assistant turns:          18
  Bash commands:            5
  File edits:               3
  Commits:                  2 (4 files, +87 -23 lines)
```

---

## Test Scenarios

### Simple: Single commit in single phase
```rust
#[test]
fn test_git_single_commit_single_phase() {
    let commits = vec![GitCommit {
        hash: "abc1234".into(),
        timestamp: "2025-01-01T10:05:00Z".into(),
        message: "test commit".into(),
        author: "Test".into(),
        files_changed: 2,
        insertions: 10,
        deletions: 5,
    }];

    let mut phases = vec![PhaseMetrics {
        phase_name: "spec".into(),
        start_time: "2025-01-01T10:00:00Z".into(),
        end_time: Some("2025-01-01T10:15:00Z".into()),
        git_commits: Vec::new(),
        // ... other fields ...
    }];

    attribute_commits_to_phases(commits, &mut phases);

    assert_eq!(phases[0].git_commits.len(), 1);
    assert_eq!(phases[0].git_commits[0].hash, "abc1234");
}
```

### Complex: Multiple commits across multiple phases
```rust
#[test]
fn test_git_multiple_commits_multiple_phases() {
    // 3 commits: 1 in spec, 2 in plan, 0 in impl
    let commits = vec![
        commit_at("2025-01-01T10:05:00Z"), // spec phase
        commit_at("2025-01-01T10:20:00Z"), // plan phase
        commit_at("2025-01-01T10:25:00Z"), // plan phase
    ];

    let mut phases = vec![
        phase("spec", "10:00", Some("10:15")),
        phase("plan", "10:15", Some("10:30")),
        phase("impl", "10:30", None), // active
    ];

    attribute_commits_to_phases(commits, &mut phases);

    assert_eq!(phases[0].git_commits.len(), 1);
    assert_eq!(phases[1].git_commits.len(), 2);
    assert_eq!(phases[2].git_commits.len(), 0);
}
```

### Error: No git repository
```rust
#[test]
fn test_no_git_repository() {
    let (_temp_dir, storage) = test_storage_with_files(None, None);
    // No .git directory created

    let metrics = parse_unified_metrics(storage.state_dir()).unwrap();

    assert!(metrics.git_commits.is_empty());
    // Should not error, just empty vec
}
```

### Error: Malformed git log output
```rust
#[test]
fn test_malformed_git_log() {
    // Simulate corrupt git output
    let corrupt_output = "COMMIT\ngarbage\nmore garbage";

    let commits = parse_git_log_output(corrupt_output);

    // Should return empty vec, not panic
    assert!(commits.is_empty());
}
```

### Integration: Full analyze command with git data
```rust
#[test]
fn test_analyze_with_git_commits() {
    let (_temp_dir, storage) = test_storage_with_git_repo();

    // Create mock git commits
    create_test_commit(&temp_dir, "2025-01-01T10:05:00Z", "feat: add feature");

    // Create workflow states
    let states = vec![
        state_transition("START", "spec", "10:00"),
    ];
    let (_states_temp, _) = test_storage_with_files(None, Some(&states));

    let result = analyze_metrics(&storage, false);

    assert!(result.is_ok());
    // Verify git commits appear in output
}
```

---

## Validation Rules

### Git Detection
- **Rule**: Only parse git commits if `.git/` directory exists and is readable
- **Error**: If `.git/` exists but `git` command fails, log warning and continue with empty commits

### Timestamp Parsing
- **Rule**: All timestamps must be valid ISO 8601 format
- **Error**: Skip commits with invalid timestamps, log warning

### Phase Attribution
- **Rule**: Commits must fall within `[phase.start_time, phase.end_time]` range
- **Error**: Commits outside all phase ranges are silently ignored (expected behavior)

### Stats Parsing
- **Rule**: `files_changed`, `insertions`, `deletions` must be non-negative integers
- **Error**: If numstat line unparseable, use 0 for that metric

---

## Performance Considerations

### Git Repository Access
- **Optimization**: Use `since` timestamp filter in revwalk to limit commit traversal
- **Best case**: Session-scoped filtering skips historical commits entirely
- **Worst case**: Large repos with 1000+ commits in session → ~200ms overhead (libgit2 is fast)
- **Mitigation**: Timestamp filtering happens during iteration, minimal memory overhead

### Memory Usage
- **Typical**: 10-50 commits per session → ~5KB memory
- **Worst case**: 1000 commits × 200 bytes/commit → ~200KB (acceptable)
- **git2 overhead**: Repository handle and object cache ~1-2MB (reused across calls)

### Diff Computation
- **Impact**: One diff computation per commit (tree comparison)
- **Latency**: ~5-10ms per commit on average
- **Acceptable**: Sequential processing during metrics aggregation, no user-facing delay

---

## Security Considerations

### Command Injection
- **Risk**: None (no subprocess calls, using native libgit2 library)
- **Benefit**: git2 provides safe Rust API, no shell injection vectors

### Path Traversal
- **Risk**: Low (`.git/` path constructed from validated state directory)
- **Mitigation**: git2::Repository::open() validates paths internally
- **Additional**: Use canonical paths when constructing project_root

### Git Repository Trust
- **Risk**: Minimal (read-only operations via libgit2)
- **Benefit**: No git hooks execute during libgit2 read operations
- **Scope**: Only reads commit history, trees, and diffs - no network, no writes

---

## Success Criteria

- [ ] `has_git_repository()` correctly detects git repository via git2::Repository::open()
- [ ] `parse_git_commits()` uses git2 crate to read commit history
- [ ] Git commit structs contain hash, timestamp, author, message, files_changed, insertions, deletions from git2::DiffStats
- [ ] `attribute_commits_to_phases()` correctly maps commits to phases by timestamp
- [ ] Commits outside phase ranges are ignored (not displayed)
- [ ] Phase Breakdown section displays commit stats: count, files, +insertions, -deletions
- [ ] Non-git projects continue working without errors (empty git_commits vec)
- [ ] git2::Repository errors gracefully degrade (empty vec, warning logged)
- [ ] All tests pass with coverage ≥80%
- [ ] Session-scoped filtering via timestamp reduces commit traversal overhead
- [ ] git2 dependency added to Cargo.toml
- [ ] No performance regression in `hegel analyze` for non-git projects
- [ ] Git commits attributed to active phases (end_time = None) work correctly

---

## Edge Cases

### Multiple Commits at Same Timestamp
- **Behavior**: All attributed to same phase, order preserved from git log

### Commit Exactly at Phase Boundary
- **Behavior**: Attributed to earlier phase (inclusive start, exclusive end)

### Active Phase (end_time = None)
- **Behavior**: All commits from phase start to "now" are included

### Git Repository But No Commits in Session
- **Behavior**: All phases show "Commits: -" (zero commits)

### Corrupted .git Directory
- **Behavior**: Log warning, proceed with empty commits vec

### Submodules
- **Behavior**: Only parse commits from main repository (ignore submodules for v1)

---

## Future Enhancements (Out of Scope)

- Submodule commit tracking
- Branch-aware analysis
- Commit message categorization (feat/fix/chore)
- Per-file change visualization
- Integration with `--export-dot` for commit nodes in workflow graph
