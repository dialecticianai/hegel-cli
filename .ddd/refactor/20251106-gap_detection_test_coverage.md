# Gap Detection Test Coverage Implementation Plan

TDD-focused plan for comprehensive test coverage of gap_detection.rs

---

## Overview

**Goal:** Achieve ≥80% test coverage for `src/analyze/gap_detection.rs` (currently 0%)

**Scope:** Create test infrastructure and implement 7 core scenarios covering happy paths and essential error cases

**Priorities:**
1. Test organization matching TESTING.md patterns
2. Verify cowboys span exact gap boundaries (core contract)
3. Cover all scenarios from HANDOFF with minimal duplication
4. Use existing test helpers for clean, expressive tests

**Methodology:**
- TDD discipline: write tests first where they drive development (tests are the deliverable)
- Focus on public API: `ensure_cowboy_coverage()`
- Verify behavior through counts and timestamp assertions
- Skip helper function tests if coverage achieved through integration
- Commit after every numbered step

---

## Step 1: Test Infrastructure Setup

### Goal
Establish test module structure following TESTING.md tier-2 pattern

### Step 1.a: Write Module Structure
Create the organizational scaffolding:
- Test module declaration file
- Main test file with imports
- Helper setup if needed beyond existing test_helpers

### Step 1.b: Implement
- Create test module directory under analyze
- Add module declarations
- Import necessary types and helpers from test_helpers and storage

### Success Criteria
- Module compiles without errors
- Test file structure matches commands/workflow/tests pattern
- Can reference ensure_cowboy_coverage and test helpers

---

## Step 2: Basic Gap Scenarios

### Goal
Test fundamental gap detection with and without activity

### Step 2.a: Write Tests
Two scenarios covering core behavior:
- Gap with git activity creates cowboy
- Gap without git activity creates nothing

Validation:
- Check return counts match expected created/removed
- Verify cowboy timestamps match gap boundaries exactly
- Confirm archive exists on filesystem

### Step 2.b: Implement
- Set up test storage using setup helpers
- Create two workflow archives with time gap
- Inject test git commits for activity scenario
- Call ensure_cowboy_coverage with test data
- Assert counts and read back archives to verify timestamps

### Success Criteria
- Both tests pass
- Timestamps verified to span gap start to gap end
- Coverage includes gap identification logic

---

## Step 3: Existing Cowboy Scenarios (Preservation)

### Goal
Verify system preserves correct cowboys and removes incorrect ones

### Step 3.a: Write Tests
Three scenarios for existing cowboy handling:
- Correct cowboy already exists, preserved
- Wrong timestamp cowboy replaced
- Multiple cowboys with one correct, others removed

Validation:
- Return counts reflect create/remove operations
- Correct cowboy timestamp boundaries preserved
- Incorrect cowboys no longer exist on filesystem

### Step 3.b: Implement
- Write existing cowboy archives to test storage
- Mix correct and incorrect timestamps
- Call ensure_cowboy_coverage
- Verify filesystem state matches expectations
- Check timestamps of remaining cowboys

### Success Criteria
- All three tests pass
- Logic for identifying correct cowboys covered
- Removal paths exercised

---

## Step 4: Gap Without Activity (Cleanup)

### Goal
Ensure spurious cowboys are removed from gaps without git commits

### Step 4.a: Write Tests
Single scenario:
- Gap with no activity but cowboy exists
- System removes the cowboy

Validation:
- Return counts show removal
- Cowboy archive no longer exists

### Step 4.b: Implement
- Create gap between workflows
- Write cowboy archive in that gap
- Provide empty git commits list
- Verify cowboy removed

### Success Criteria
- Test passes
- Cleanup logic for inactive gaps covered

---

## Step 5: Multiple Gaps Scenario

### Goal
Verify independent handling of multiple gaps with different activity patterns

### Step 5.a: Write Tests
Single scenario with complexity:
- Three workflows creating two gaps
- First gap has git activity
- Second gap has no activity
- System creates cowboy only for first gap

Validation:
- Return counts correct
- Cowboys exist only where activity detected
- Each gap handled independently

### Step 5.b: Implement
- Create three workflow archives
- Inject git commits only in first gap
- Call ensure_cowboy_coverage once
- Verify correct gap receives cowboy

### Success Criteria
- Test passes
- Multiple gap iteration logic covered

---

## Step 6: Coverage Verification and Iteration

### Goal
Assess coverage and add targeted tests if needed for ≥80% threshold

### Step 6.a: Generate Coverage Report
Run coverage tooling to identify uncovered lines

### Step 6.b: Analyze Gaps
- Check if private helpers need direct testing
- Identify any error paths not yet exercised
- Determine if additional scenarios needed

### Step 6.c: Add Targeted Tests (If Needed)
Write minimal tests to cover remaining critical paths:
- Timestamp parsing edge cases if not covered
- Error handling paths if present
- Any conditional branches missed

### Success Criteria
- Coverage report shows ≥80% for gap_detection.rs
- All critical paths exercised
- No redundant tests added

---

## Step 7: Documentation and Final Verification

### Goal
Update related documentation and verify clean test suite

### Step 7.a: Run Full Test Suite
Ensure no regressions in existing tests

### Step 7.b: Update Documentation
- Confirm TESTING.md accurately reflects new test organization
- No other docs need updates (implementation unchanged)

### Step 7.c: Build and Install
Run build script to verify no warnings or errors

### Success Criteria
- All tests pass (gap_detection + full suite)
- Coverage target met
- No compiler warnings
- Documentation current
