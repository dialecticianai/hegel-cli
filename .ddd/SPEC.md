# Gap Detection Test Coverage Specification

Comprehensive test suite for `src/analyze/gap_detection.rs`

---

## Overview

**What it does:** Implements test coverage for `ensure_cowboy_coverage()` which fills gaps between workflows with synthetic cowboy archives when git activity is detected.

**Key principles:**
- Use `ArchiveBuilder` and `test_git_commit()` helpers for clean test data
- Inject test data via `git_commits: Option<&[GitCommit]>` parameter
- Verify timestamp correctness (cowboys must span gaps)
- Cover all 7 scenarios from HANDOFF plus private helper functions

**Scope:** Test implementation only - no changes to `gap_detection.rs` logic

**Integration context:** Uses existing test helpers from `src/test_helpers/archive.rs` and storage utilities

---

## Test Organization

**New files:**
- `src/analyze/tests/mod.rs` - test module declaration
- `src/analyze/tests/gap_detection.rs` - comprehensive test suite

**Modified:**
- `src/analyze/mod.rs` - add `#[cfg(test)] mod tests;`

---

## Test Scenarios

### Core Function: `ensure_cowboy_coverage()`

**Scenario 1: Gap with git activity**
- Two workflows with time gap
- Git commits in gap timeframe
- Verify: `(created=1, removed=0)`, cowboy timestamps span gap

**Scenario 2: Gap without git activity**
- Two workflows with time gap
- No git commits in gap
- Verify: `(created=0, removed=0)`, no cowboy created

**Scenario 3: Correct cowboy already exists**
- Gap with git activity
- Cowboy with exact gap timestamps already present
- Verify: `(created=0, removed=0)`, existing cowboy preserved

**Scenario 4: Wrong cowboy timestamps**
- Gap with git activity
- Cowboy exists but spans different time range
- Verify: `(created=1, removed=1)`, old replaced with correct

**Scenario 5: Multiple cowboys in gap**
- Gap with git activity
- Multiple cowboys overlap gap, one correct
- Verify: `(created=0, removed=N-1)`, only correct cowboy remains

**Scenario 6: Cowboy without activity**
- Gap with no git commits
- Cowboy exists in this gap
- Verify: `(created=0, removed=1)`, spurious cowboy removed

**Scenario 7: Multiple gaps**
- Three workflows creating two gaps
- First gap has activity, second doesn't
- Verify: Creates cowboy for first gap only, handles independently

### Helper Functions

**Scenario 8: `parse_timestamp()` validation**
- Valid RFC3339 timestamps parse correctly
- Private function, test via coverage of main function or expose for testing

**Scenario 9: `create_cowboy_for_gap()` behavior**
- Private function called when creating cowboys
- Verify through integration (scenario 1) or test directly if needed for coverage

---

## Success Criteria

- All tests pass: `cargo test gap_detection`
- Coverage for `gap_detection.rs` reaches ≥80%
- Tests use existing infrastructure (`ArchiveBuilder`, `test_archive`, `test_git_commit`)
- Timestamp verification confirms cowboys span exact gap boundaries
- No code changes to `gap_detection.rs` implementation
- Tests follow patterns from `commands/workflow/tests/transitions.rs`

---

## Out of Scope

- Edge cases (malformed timestamps, filesystem errors) - defer until hot paths covered
- Testing `dry_run=true` mode exhaustively - verify in one scenario only
- Performance testing with large archive sets
- Integration with real git repositories
- Archive content verification beyond timestamps and counts
