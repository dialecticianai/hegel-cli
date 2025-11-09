# Commit Guardrails Implementation Plan

## Overview

**Goal:** Add require_commits rule type to block phase transitions without git commits, with global config toggle and selective force bypass.

**Scope:**
- Add RequireCommits variant to RuleConfig enum
- Extend HegelConfig with commit_guard and use_git fields
- Modify RuleEvaluationContext to provide full phase array
- Add force flag with optional rule type filtering
- Validate lookback_phases >= 1

**Priorities:**
1. Core rule evaluation (commits present/absent)
2. Config integration (global toggle, git override)
3. Force bypass (all rules vs specific type)
4. Existing rule updates (context change)

**Methodology:** TDD for rule evaluation logic, integration tests for CLI and config. Test core behavior, skip hypothetical edge cases.

---

## Step 1: Add RequireCommits Rule Type

### Goal
Extend RuleConfig enum with new variant and validation.

### Step 1.a: Write Tests

Describe test strategy for rule type addition:
- Test deserialization from YAML with lookback_phases field
- Test validation accepts lookback_phases >= 1
- Test validation rejects lookback_phases == 0
- Test validate allows large values like 999
- Test serialization roundtrip preserves lookback_phases

### Step 1.b: Implement

Add RequireCommits variant to RuleConfig enum in src/rules/types.rs:
- Add variant with lookback_phases usize field
- Implement validate method checking lookback_phases >= 1
- Add test module following existing pattern from RepeatedCommand tests
- Update RuleConfig validate match arm to call new validation

### Success Criteria

- RuleConfig deserializes require_commits from YAML
- Validation rejects lookback_phases == 0
- Validation accepts lookback_phases from 1 to 999
- Serialization roundtrip works correctly
- Tests pass for new variant

**Commit Point:** `feat(rules): add RequireCommits rule type with validation`

---

## Step 2: Update RuleEvaluationContext

### Goal
Change context from single phase to full phase array to enable lookback.

### Step 2.a: Write Tests

Describe test strategy for context changes:
- Test existing rules (repeated_command, phase_timeout, etc.) still work with new context structure
- Test rules can find current phase in array
- Test rules handle empty phase array gracefully

### Step 2.b: Implement

Update RuleEvaluationContext structure in src/rules/types.rs:
- Change phase_metrics from Option single phase to all_phase_metrics slice
- Update all rule evaluators in src/rules/evaluator.rs to find current phase in array
- Update context construction in src/engine/mod.rs to pass full phase array
- Fix compilation errors in existing rule evaluation functions

### Success Criteria

- RuleEvaluationContext has all_phase_metrics field
- Existing rules compile and pass tests
- Context construction passes full metrics array
- No regressions in existing rule behavior

**Commit Point:** `refactor(rules): update context to provide full phase array`

---

## Step 3: Implement Require Commits Rule Evaluation

### Goal
Core logic to check for commits in lookback window.

### Step 3.a: Write Tests

Describe test strategy for commit checking:
- Test with commits present in current phase returns no violation
- Test with no commits in current phase returns violation
- Test lookback_phases equals 2 combines two phases
- Test lookback exceeding available phases uses all available
- Test empty git_commits vector triggers violation
- Test violation message includes lookback count

### Step 3.b: Implement

Add evaluate_require_commits function in src/rules/evaluator.rs:
- Find current phase index in all_phase_metrics array
- Calculate lookback window start index using max of zero and current minus lookback plus one
- Collect all git_commits from phases in window
- Return RuleViolation if combined commits is empty
- Return None if any commits found
- Add to evaluate_rules match statement
- Create RuleViolation with appropriate diagnostic and suggestion text

### Success Criteria

- Rule evaluation finds commits across multiple phases
- Lookback window calculation handles edge cases
- Violation returned when no commits found
- None returned when commits present
- Tests cover single phase, multi-phase, and exceeding history cases

**Commit Point:** `feat(rules): implement require_commits evaluation logic`

---

## Step 4: Add Config Fields

### Goal
Extend HegelConfig with commit_guard and use_git toggles.

### Step 4.a: Write Tests

Describe test strategy for config extension:
- Test default config has commit_guard true and use_git None
- Test set commit_guard to false persists
- Test set use_git to true and false persists
- Test get returns correct values for new fields
- Test list includes new fields in output
- Test save and load roundtrip preserves new fields

### Step 4.b: Implement

Update HegelConfig in src/config.rs:
- Add commit_guard bool field with default true
- Add use_git Option bool field with default None
- Update Default impl to set commit_guard true
- Extend get method to handle commit_guard and use_git keys
- Extend set method with validation for boolean parsing
- Extend list method to include new fields
- Update config tests to cover new fields

### Success Criteria

- Config has commit_guard and use_git fields
- Defaults are commit_guard true and use_git None
- Config commands work for new fields
- Serialization roundtrip preserves values
- Tests pass for get, set, list operations

**Commit Point:** `feat(config): add commit_guard and use_git toggles`

---

## Step 5: Add Config-Based Rule Skipping

### Goal
Skip require_commits rules when commit_guard disabled or git unavailable.

### Step 5.a: Write Tests

Describe test strategy for config integration:
- Test commit_guard false skips all require_commits rules
- Test git unavailable skips require_commits rules
- Test use_git false overrides detected repo
- Test use_git true enables rule even without detected repo
- Test other rule types unaffected by commit_guard setting

### Step 5.b: Implement

Update evaluate_require_commits in src/rules/evaluator.rs:
- Load config from state_dir at evaluation time
- Check commit_guard config field and return None early if false
- Check use_git override and git_info from state
- Return None if git unavailable and not overridden
- Continue with commit check logic if config allows
- Update function signature to accept state_dir parameter if needed
- Thread state_dir through from engine call site

### Success Criteria

- Config commit_guard false bypasses rule
- Git unavailable bypasses rule gracefully
- use_git override respected in both directions
- Other rules unaffected by commit_guard
- Tests verify all config combinations

**Commit Point:** `feat(rules): integrate commit_guard config with evaluation`

---

## Step 6: Add Force Flag to CLI

### Goal
Support force bypass for all rules or specific rule types.

### Step 6.a: Write Tests

Describe test strategy for force flag:
- Test force without argument bypasses all rules
- Test force with require_commits bypasses only that type
- Test force with phase_timeout bypasses only that type
- Test unspecified rules still evaluated when force selective
- Test invalid rule type name handled gracefully

### Step 6.b: Implement

Update Commands enum in src/main.rs:
- Add force field to Next subcommand with Option Option String type
- Document usage in help text with examples
- Update handle_next function to pass force flag to transition logic
- Update evaluate_transition or get_next_prompt to filter rules based on force flag
- Add rule filtering logic before calling evaluate_rules
- Filter by matching rule type name when force is Some specific type

### Success Criteria

- CLI accepts force flag with and without argument
- Force without argument skips all rule evaluation
- Force with type name filters out matching rules only
- Help text shows usage examples
- Tests verify all force flag combinations

**Commit Point:** `feat(cli): add force flag with selective rule bypass`

---

## Step 7: Integration and Validation

### Goal
Ensure all components work together and validate workflows load correctly.

### Step 7.a: Write Tests

Describe integration test strategy:
- Test workflow validation rejects lookback_phases zero
- Test workflow validation accepts lookback_phases one and higher
- Test end-to-end transition with require_commits rule
- Test manual workflow with rule in execution mode
- Test config interaction with workflow rule

### Step 7.b: Implement

Update workflow validation in src/engine/mod.rs:
- RequireCommits variant already validated by RuleConfig validate
- Add integration test creating workflow YAML with require_commits rule
- Test workflow load succeeds with valid lookback_phases
- Test workflow load fails with zero lookback_phases
- Verify error message format matches spec
- Test hegel next with require_commits in real workflow

### Success Criteria

- Workflow validation catches invalid lookback_phases
- Error messages clear and actionable
- Integration test demonstrates full feature flow
- Manual testing confirms CLI and config work end-to-end
- All tests pass with no regressions

**Commit Point:** `test(rules): add integration tests for require_commits`

---

## Final Success Criteria

Build succeeds: cargo build compiles all modified files

Tests pass: cargo test with no regressions

Modified files:
- src/rules/types.rs - RequireCommits variant added
- src/rules/evaluator.rs - Evaluation function implemented
- src/config.rs - Two new config fields
- src/engine/mod.rs - Context construction updated
- src/main.rs - Force flag added to Next command

Integration verified:
- Start execution workflow with require_commits rule
- Transition blocks without commits
- Commit made, transition succeeds
- Force flag bypasses rule
- Config disable skips rule

Documentation:
- CLI help shows force flag usage
- Config help mentions new fields
- No README or CLAUDE.md updates in this phase
