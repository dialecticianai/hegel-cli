# Refactor: Multi-Session Token Attribution

**Date**: 2025-11-06
**Status**: Planning
**Scope**: `src/metrics/` (aggregation, transcript scanning)

## Problem Statement

**Bug**: Workflows spanning multiple Claude Code sessions have incomplete token attribution.

**Impact**:
- Gap-detected cowboy workflows show 0 tokens despite actual activity
- Multi-session explicit workflows lose tokens from earlier sessions
- Metrics analysis underreports actual token usage

**Root cause**: Both normal archiving and gap detection only scan ONE transcript file (current session), missing historical sessions.

## Evidence

### Perl Validation Script

Created `scripts/scan-gap-transcripts.pl` to validate multi-file scanning:

```bash
./scripts/scan-gap-transcripts.pl "2025-11-03T03:13:34Z" "2025-11-04T22:11:56Z"
```

**Results** (Nov 3-4 gap, ~43 hours):
- Scanned: 88 transcript files
- Matched: 1625 assistant events across multiple sessions
- **Tokens found**: 75K input, 132K output, 5.2M cache creation, 160M cache read
- **Current implementation**: 0 tokens

**Proof**: Historical transcript data exists and is accessible, but current code doesn't discover it.

## Current Architecture

### Single-File Scanning Flow

```
parse_unified_metrics()
├─ Get transcript_path from state.session_metadata (ONE path)
├─ OR scan hooks.jsonl for last SessionStart (ONE path)
└─ build_phase_metrics(..., transcript_path: Option<&str>, ...)
   └─ For each phase:
      └─ aggregate_tokens_for_phase(transcript_path, start, end)  // ONE file
```

### Key Functions

**src/metrics/mod.rs:183-220** - Transcript discovery
- Reads state.session_metadata.transcript_path
- Fallback: scans hooks.jsonl for last SessionStart event
- Returns single path

**src/metrics/aggregation.rs:133** - Token aggregation
```rust
fn aggregate_tokens_for_phase(
    transcript_path: &str,           // Single file!
    start_time: &str,
    end_time: Option<&str>,
    phase_name: &str,
    debug_config: Option<&DebugConfig>,
) -> Result<(TokenMetrics, usize, usize)>
```

**src/metrics/aggregation.rs:8** - Phase metrics builder
```rust
pub fn build_phase_metrics(
    transitions: &[StateTransitionEvent],
    hook_metrics: &HookMetrics,
    transcript_path: Option<&str>,   // Single file!
    debug_config: Option<&crate::metrics::DebugConfig>,
) -> Result<Vec<PhaseMetrics>>
```

## SoC/DRY Violations

### 1. DRY: Duplicated Transcript Discovery

**Location**: Gap detection duplicates transcript discovery logic

**src/analyze/gap_detection.rs:308-314**:
```rust
let storage = FileStorage::new(state_dir)?;
let state = storage.load()?;
let transcript_path = state
    .session_metadata
    .as_ref()
    .map(|s| s.transcript_path.as_str());
```

**src/metrics/mod.rs:180-220**: Same logic, slightly different

**Violation**: Transcript discovery logic duplicated instead of extracted to helper.

### 2. SoC: Transcript Discovery Mixed with Metrics Parsing

**Location**: `parse_unified_metrics()` does both discovery AND parsing

**Responsibilities mixed**:
- Session metadata extraction
- Transcript path discovery (agent-specific concern)
- Hooks parsing
- States parsing
- Token metrics parsing (agent-agnostic concern)
- Phase metrics building

**Should be**: Separate transcript discovery into adapter layer, keep aggregation in metrics layer.

### 3. DRY: Single-File Limitation Baked Into Signatures

**Problem**: Function signatures assume single file:
- `transcript_path: Option<&str>`
- Must change multiple function signatures simultaneously

**Better**: Encapsulate discovery/scanning behind clean interface.

## Proposed Refactoring

### Design Principles

1. **Adapter separation**: Agent-specific discovery logic in `src/adapters/<agent>/`
2. **Metrics agnostic**: Token aggregation works with any transcript file list
3. **DRY**: Single source of truth for multi-file scanning
4. **Backward compat**: Gracefully handle missing transcript directories
5. **Performance**: Stream files, don't load into memory
6. **Testability**: Pure functions, easy to mock

### New Functions in src/adapters/claude_code.rs

**Purpose**: Claude Code-specific transcript discovery

**API**:
```rust
/// Discover Claude Code project directory for given repo path
pub fn find_transcript_dir(repo_path: &Path) -> Result<PathBuf> {
    // Convert /Users/foo/Code/bar
    //      → ~/.claude/projects/-Users-foo-Code-bar
}

/// List all transcript files for this Claude Code project
pub fn list_transcript_files(repo_path: &Path) -> Result<Vec<PathBuf>> {
    // 1. Find project directory
    // 2. Glob *.jsonl files
    // 3. Return sorted by mtime
}
```

### New Function in src/metrics/aggregation.rs

**Purpose**: Agent-agnostic multi-file token aggregation

**API**:
```rust
/// Aggregate tokens from multiple transcript files for a time range
pub fn aggregate_tokens_for_range(
    transcript_files: &[PathBuf],
    start_time: &str,
    end_time: Option<&str>,
    debug_config: Option<&DebugConfig>,
) -> Result<TokenMetrics> {
    // For each file:
    //   1. Stream parse line-by-line
    //   2. Filter assistant events by timestamp
    //   3. Accumulate tokens
    // Return aggregated TokenMetrics
}
```

### Updated aggregation.rs

**Change**: `build_phase_metrics()` signature

**Before**:
```rust
pub fn build_phase_metrics(
    transitions: &[StateTransitionEvent],
    hook_metrics: &HookMetrics,
    transcript_path: Option<&str>,  // OLD: single file
    debug_config: Option<&crate::metrics::DebugConfig>,
) -> Result<Vec<PhaseMetrics>>
```

**After**:
```rust
pub fn build_phase_metrics(
    transitions: &[StateTransitionEvent],
    hook_metrics: &HookMetrics,
    transcript_files: &[PathBuf],   // NEW: multiple files
    debug_config: Option<&crate::metrics::DebugConfig>,
) -> Result<Vec<PhaseMetrics>>
```

**Implementation**:
```rust
// For each phase:
let token_metrics = aggregate_tokens_for_range(
    transcript_files,
    &start_time,
    end_time.as_deref(),
    debug_config,
)?;
```

### Updated gap_detection.rs

**Change**: Use adapter for transcript discovery

**Before**:
```rust
let transcript_path = state
    .session_metadata
    .as_ref()
    .map(|s| s.transcript_path.as_str());

build_phase_metrics(&state_transitions, &hook_metrics, transcript_path, None)?;
```

**After**:
```rust
use crate::adapters::claude_code;

let project_root = state_dir.parent().unwrap();
let transcript_files = claude_code::list_transcript_files(project_root).unwrap_or_default();

build_phase_metrics(&state_transitions, &hook_metrics, &transcript_files, None)?;
```

### Updated mod.rs (parse_unified_metrics)

**Change**: Use adapter instead of single-file discovery

**Before**:
```rust
let transcript_path = if let Some(session) = state.session_metadata {
    Some(session.transcript_path)
} else {
    // Scan hooks.jsonl for last SessionStart...
};

let live_phase_metrics = build_phase_metrics(
    &unified.state_transitions,
    &unified.hook_metrics,
    transcript_path.as_deref(),
    debug_config,
)?;
```

**After**:
```rust
use crate::adapters::claude_code;

let project_root = state_dir.parent().unwrap();
let transcript_files = claude_code::list_transcript_files(project_root).unwrap_or_default();

let live_phase_metrics = build_phase_metrics(
    &unified.state_transitions,
    &unified.hook_metrics,
    &transcript_files,
    debug_config,
)?;
```

## Implementation Steps

### Phase 1: Add Claude Code adapter functions

1. ✅ Create `scripts/scan-gap-transcripts.pl` (validation)
2. Port Perl logic to `src/adapters/claude_code.rs`:
   - `find_transcript_dir()` - Path conversion logic
   - `list_transcript_files()` - Find dir, glob *.jsonl
3. Export functions via `src/adapters/mod.rs`
4. Unit tests for path conversion logic

### Phase 2: Add multi-file aggregation to metrics

1. Add `aggregate_tokens_for_range()` to `src/metrics/aggregation.rs`
2. Implementation: Stream each file, filter by timestamp, accumulate
3. Remove old single-file `aggregate_tokens_for_phase()` function
4. Update tests

### Phase 3: Update build_phase_metrics signature

1. Change `build_phase_metrics()` signature: `transcript_path` → `transcript_files`
2. Update implementation to call new `aggregate_tokens_for_range()`
3. Update tests

### Phase 4: Update callers

1. `src/metrics/mod.rs::parse_unified_metrics()` - Call adapter, pass files list
2. `src/analyze/gap_detection.rs::create_cowboy_for_gap()` - Call adapter, pass files list
3. Remove duplicated transcript discovery logic

### Phase 5: Verification

1. Run `./scripts/build.sh --skip-bump` - Ensure builds
2. Run `cargo test` - Ensure tests pass
3. Run `hegel analyze --fix-archives` - Regenerate archives
4. Run debug analysis on Nov 3-4 gap:
   ```bash
   hegel analyze --debug 2025-11-03T00:00:00Z..2025-11-05T00:00:00Z --json \
     | ./scripts/analyze-token-attribution.py
   ```
5. Verify zero-token rate drops significantly

## Expected Outcomes

### Token Attribution Accuracy

**Before**: 99.7% zero-token rate for archived phases
**After**: <5% zero-token rate (legitimate gaps with no Claude activity)

### Code Quality

- **DRY**: Single source of truth for transcript scanning
- **SoC**: Clear separation between discovery, parsing, aggregation
- **Maintainability**: Changes to transcript scanning logic in ONE place
- **Testability**: Pure functions, easy to unit test

### Performance

- **Multi-file scanning**: Streaming parse (constant memory)
- **Parallelization opportunity**: Independent file scans (future optimization)
- **Caching opportunity**: Transcript timestamp ranges (future optimization)

## Risks & Mitigation

### Risk: Claude Code Directory Not Found

**Scenario**: User has never opened repo in Claude Code
**Mitigation**: Graceful fallback to 0 tokens (same as current behavior)

### Risk: Large Transcript Files

**Scenario**: Transcript files >100MB, slow to scan
**Mitigation**:
- Streaming parse (constant memory)
- Early termination when timestamp exceeds range
- Future: Cache file timestamp ranges

### Risk: Breaking Changes

**Scenario**: Signature changes break downstream code
**Mitigation**:
- Limited call sites (2 locations)
- Comprehensive test coverage before merge
- Internal functions (not public API)

## Success Criteria

1. ✅ Perl validation script proves concept
2. ⏳ Nov 3-4 gap shows non-zero tokens after refactor
3. ⏳ All existing tests pass
4. ⏳ Zero-token rate <5% in production data
5. ⏳ No regressions in normal archiving flow

## Token Savings Estimate

**Before refactor**:
- Functions reading state.json, hooks.jsonl repeatedly
- Duplicated discovery logic in gap_detection.rs
- ~150 lines across multiple files

**After refactor**:
- Single module with clear responsibilities
- Reused by all callers
- ~200 lines in ONE place (net +50 lines for multi-file support)

**Token efficiency**: More lines but MUCH clearer structure. Worth it for correctness.
