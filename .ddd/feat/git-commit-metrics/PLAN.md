# Git Commit Metrics Implementation Plan

Implementation plan for integrating git commit tracking into Hegel workflow phase metrics using git2 crate.

---

## Overview

**Goal**: Automatically detect and parse git repository commits, attribute them to workflow phases by timestamp, and display commit statistics (count, files changed, insertions, deletions) in the analyze command output.

**Scope**:
- Add git2 dependency to Cargo.toml
- Create git module in metrics for commit parsing
- Extend UnifiedMetrics and PhaseMetrics data structures
- Update analyze command rendering
- Comprehensive test coverage for all git operations

**Priorities**:
1. Zero-impact on non-git projects (graceful degradation)
2. Accurate timestamp-based phase attribution
3. Robust error handling for corrupt repos
4. Test coverage ≥80%

---

## Methodology

**TDD Principles**:
- Write failing tests before implementation for all public functions
- Test both happy path and error cases (no git repo, corrupt repo, empty history)
- Integration tests verify end-to-end analyze command output
- Commit after each numbered step completion

**What to Test**:
- Git repository detection logic
- Commit parsing and stats extraction
- Phase attribution by timestamp correlation
- Graceful degradation when git2 operations fail
- Analyze command output formatting

**What NOT to Test** (covered by git2 crate):
- libgit2 internal diff algorithms
- Git object storage format
- Repository corruption detection internals

---

## Step 1: Add git2 Dependency

### Goal
Enable git repository access via libgit2 Rust bindings.

### Step 1.a: Write Tests
Not applicable - dependency addition only.

### Step 1.b: Implement
- Add git2 crate to Cargo.toml dependencies
- Verify compilation with cargo check
- No code changes yet, just dependency setup

### Success Criteria
- [ ] git2 crate added to Cargo.toml
- [ ] cargo check passes without errors
- [ ] Commit with message: feat(metrics): add git2 dependency for commit tracking

---

## Step 2: Create Git Commit Data Structures

### Goal
Define the GitCommit struct and extend existing metrics structures to hold git data.

### Step 2.a: Write Tests
Write tests that verify:
- GitCommit struct can be constructed with all required fields
- PhaseMetrics can hold a vector of GitCommit
- UnifiedMetrics can hold git commits
- Serialization/deserialization works correctly (if needed for future features)

Tests should check:
- Default empty git_commits vectors
- Adding commits to phase metrics
- Hash formatting (7 character truncation)
- Timestamp ISO 8601 format validation

### Step 2.b: Implement
- Create src/metrics/git.rs module
- Define GitCommit struct with all fields from SPEC
- Add git_commits field to PhaseMetrics in src/metrics/mod.rs
- Add git_commits field to UnifiedMetrics in src/metrics/mod.rs
- Add module declaration in src/metrics/mod.rs
- Ensure all new fields initialize to empty vectors by default

### Success Criteria
- [ ] GitCommit struct defined with all required fields
- [ ] PhaseMetrics extended with git_commits vector
- [ ] UnifiedMetrics extended with git_commits vector
- [ ] All tests pass
- [ ] Commit with message: feat(metrics): add GitCommit data structures

---

## Step 3: Implement Git Repository Detection

### Goal
Safely detect presence of git repository without errors.

### Step 3.a: Write Tests
Write tests that verify:
- Returns true when valid git repository exists
- Returns false when no git directory exists
- Returns false when git directory is corrupt
- Returns false when passed invalid paths
- Never panics regardless of filesystem state

Test cases:
- Create temporary directory with initialized git repo
- Create temporary directory without git repo
- Pass non-existent path
- Pass file path instead of directory path

### Step 3.b: Implement
- Add has_git_repository function to src/metrics/git.rs
- Use git2::Repository::open on parent directory of state_dir
- Return boolean based on Result (Ok = true, Err = false)
- Handle all edge cases gracefully with early returns
- No error logging yet, just boolean detection

### Success Criteria
- [ ] has_git_repository function implemented
- [ ] All detection tests pass (happy path and errors)
- [ ] Function never panics
- [ ] Commit with message: feat(metrics): implement git repository detection

---

## Step 4: Implement Commit Parsing with Stats

### Goal
Parse git commits using git2 crate and extract metadata plus diff statistics.

### Step 4.a: Write Tests
Write tests that verify:
- Successfully parses commits from test repository
- Extracts correct hash (7 chars), author, message, timestamp
- Computes diff stats (files changed, insertions, deletions)
- Handles commits with no parent (root commits)
- Filters commits by since timestamp correctly
- Returns empty vector when repository has no commits
- Returns error when repository path is invalid
- Skips commits outside timestamp filter

Test cases:
- Create test git repo with multiple commits
- Verify stats match expected values for known commits
- Test with since parameter filtering half the commits
- Test root commit (no parent) returns zero diff stats
- Test empty repository returns empty vector
- Test invalid path returns error

### Step 4.b: Implement
- Add parse_git_commits function to src/metrics/git.rs
- Open repository with git2::Repository::open
- Create revwalk and push HEAD
- Iterate commits, filter by timestamp if since provided
- For each commit, get diff stats via helper function
- Extract metadata (hash format with 7 chars, author name, first line of message)
- Convert Unix timestamp to ISO 8601 string
- Return vector of GitCommit structs
- Add helper function get_commit_stats that diffs against parent tree
- Handle root commits by diffing against empty tree

### Success Criteria
- [ ] parse_git_commits function implemented
- [ ] get_commit_stats helper function implemented
- [ ] Timestamp conversion utility function implemented
- [ ] All parsing tests pass
- [ ] Handles root commits correctly
- [ ] Filters by timestamp correctly
- [ ] Commit with message: feat(metrics): implement git commit parsing with stats

---

## Step 5: Implement Phase Attribution Logic

### Goal
Correlate parsed commits to workflow phases based on timestamp ranges.

### Step 5.a: Write Tests
Write tests that verify:
- Single commit correctly attributed to matching phase
- Multiple commits split across multiple phases correctly
- Commit at exact phase boundary attributed to earlier phase
- Commits outside all phase ranges are ignored
- Active phases (end_time = None) include commits up to now
- Empty commits vector handled gracefully
- Empty phases vector handled gracefully

Test cases:
- Create mock phases with known time ranges
- Create mock commits with timestamps in different phases
- Verify each phase gets correct commits
- Test boundary conditions (exact start/end times)
- Test active phase captures recent commits

### Step 5.b: Implement
- Add attribute_commits_to_phases function to src/metrics/git.rs
- Takes commits vector and mutable reference to phases vector
- For each commit, iterate phases to find matching time range
- Check if commit timestamp is between phase start and end
- Handle active phases (end_time is None) by comparing against now
- Clone commit and push to phase's git_commits vector
- Break after first match (earlier phase wins on boundary)
- Commits with no match are silently skipped

### Success Criteria
- [ ] attribute_commits_to_phases function implemented
- [ ] All attribution tests pass
- [ ] Boundary cases handled correctly
- [ ] Active phases work correctly
- [ ] Commit with message: feat(metrics): implement commit-to-phase attribution

---

## Step 6: Integrate Git Parsing into parse_unified_metrics

### Goal
Wire up git commit parsing into the main metrics aggregation pipeline.

### Step 6.a: Write Tests
Write tests that verify:
- parse_unified_metrics includes git commits when repo exists
- parse_unified_metrics returns empty commits when no repo
- Git parsing errors don't fail overall metrics parsing
- Session start time correctly converted to Unix timestamp
- Commits are attributed to phases during parsing
- Integration test with real hegel state directory and git repo

Test cases:
- Create test storage with git repo and commits
- Create test storage without git repo
- Verify UnifiedMetrics contains expected commits
- Verify phases contain attributed commits
- Test error handling when git parsing fails

### Step 6.b: Implement
- Import git functions in src/metrics/mod.rs
- In parse_unified_metrics, call has_git_repository
- If true, get project root from state_dir parent
- Convert session_start_time from ISO 8601 to Unix timestamp using chrono
- Call parse_git_commits with project root and timestamp
- Handle errors by logging warning and using empty vector
- Call attribute_commits_to_phases with commits and phase_metrics
- Store commits in UnifiedMetrics git_commits field
- Ensure non-git projects skip all git logic without errors

### Success Criteria
- [ ] Git parsing integrated into parse_unified_metrics
- [ ] All integration tests pass
- [ ] Non-git projects unaffected
- [ ] Errors logged but don't fail metrics parsing
- [ ] Commit with message: feat(metrics): integrate git parsing into unified metrics

---

## Step 7: Update Phase Breakdown Rendering

### Goal
Display commit statistics in the Phase Breakdown section of analyze output.

### Step 7.a: Write Tests
Write tests that verify:
- Phases with commits show correct count and stats
- Phases without commits show dash placeholder
- Stats correctly sum files changed, insertions, deletions
- Output formatting matches theme style
- Integration test that full analyze command includes git stats

Test cases:
- Create UnifiedMetrics with phases containing git commits
- Verify output includes commit line with correct format
- Test phase with zero commits shows dash
- Test phase with multiple commits aggregates stats correctly

### Step 7.b: Implement
- Update render_phase_breakdown in src/commands/analyze/sections.rs
- After file edits section, add commits section
- Check if phase.git_commits is empty
- If empty, display dash using Theme::secondary
- If not empty, sum up total files changed, insertions, deletions across all commits
- Format output: count, files count, plus insertions, minus deletions
- Use format_metric for count
- Match existing section formatting style

### Success Criteria
- [ ] Commit stats displayed in phase breakdown
- [ ] Empty phases show dash placeholder
- [ ] Stats correctly aggregated across multiple commits
- [ ] Formatting consistent with existing sections
- [ ] All rendering tests pass
- [ ] Commit with message: feat(analyze): display git commit stats in phase breakdown

---

## Step 8: Integration Testing and Documentation

### Goal
Verify end-to-end functionality and document the feature.

### Step 8.a: Write Tests
Write comprehensive integration tests that verify:
- Full workflow: create git repo, run workflow, make commits, run analyze
- Output includes commit statistics
- Multiple phases with different commit counts
- Non-git project continues working
- Error scenarios degrade gracefully

Test cases:
- Create temporary git repo with hegel state
- Initialize hegel workflow, make commits in different phases
- Run analyze and verify output contains commit stats
- Test without git repo shows no errors
- Test with corrupt git repo shows warning

### Step 8.b: Implement
- Add comprehensive test in src/commands/analyze/mod.rs
- Create helper in test_helpers for setting up test git repos
- Test full analyze_metrics function with git data
- Verify no regressions in existing analyze tests
- Add inline documentation to git.rs module
- Document public functions with examples
- Ensure all error paths are covered by tests

### Success Criteria
- [ ] End-to-end integration tests pass
- [ ] Test coverage ≥80% for git module
- [ ] All existing tests still pass
- [ ] Public functions documented with examples
- [ ] No performance regression for non-git projects
- [ ] Commit with message: test(metrics): add comprehensive git integration tests

---

## Step 9: Manual Testing and Polish

### Goal
Verify feature works correctly in real usage scenarios.

### Step 9.a: Manual Testing Steps
- Build and install hegel locally
- Run hegel start/next workflow in this repository
- Make test commits during workflow phases
- Run hegel analyze and verify commit stats appear
- Test in non-git project directory
- Test with very old commits (outside session)
- Verify performance is acceptable on large repos

### Step 9.b: Implementation Polish
- Review all error messages for clarity
- Ensure warnings are actionable
- Verify theme consistency in output
- Check for any unwanted debug output
- Final cargo clippy and cargo fmt pass
- Update COVERAGE_REPORT.md if needed

### Success Criteria
- [ ] Manual testing completed successfully
- [ ] Feature works in real hegel workflow
- [ ] No console spam or debug output
- [ ] Error messages are clear and helpful
- [ ] All clippy warnings resolved
- [ ] Code formatted with rustfmt
- [ ] Commit with message: polish(metrics): finalize git commit metrics feature

---

## Commit Discipline

Each step above must be committed individually after tests pass:
- Step 1: feat(metrics): add git2 dependency for commit tracking
- Step 2: feat(metrics): add GitCommit data structures
- Step 3: feat(metrics): implement git repository detection
- Step 4: feat(metrics): implement git commit parsing with stats
- Step 5: feat(metrics): implement commit-to-phase attribution
- Step 6: feat(metrics): integrate git parsing into unified metrics
- Step 7: feat(analyze): display git commit stats in phase breakdown
- Step 8: test(metrics): add comprehensive git integration tests
- Step 9: polish(metrics): finalize git commit metrics feature

All commits must include Claude Code footer.

---

## Risk Mitigation

**Performance**: Timestamp filtering and session scoping minimize commit traversal overhead. Test with large repos (1000+ commits) to verify acceptable latency.

**Security**: git2 crate provides read-only access with no hook execution. No user input flows to git operations.

**Compatibility**: Feature is entirely additive. Non-git projects see zero behavior change. Existing tests must all pass.

**Error Handling**: All git2 errors caught and converted to empty results with warning logs. Never fail metrics parsing due to git issues.
