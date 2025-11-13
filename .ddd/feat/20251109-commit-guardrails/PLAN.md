# Commit Guardrails Implementation Plan

## Overview

**Goal:** Add require_commits rule type to block phase transitions without git commits, with global config toggle and selective force bypass.

**Scope:**
- Add RequireCommits variant to RuleConfig enum
- Extend HegelConfig with commit_guard and use_git fields
- Extend RuleEvaluationContext with full phase array, config, and git_info
- Add force flag with optional rule type filtering to CLI
- Validate lookback_phases >= 1

**Priorities:**
1. Core rule evaluation (commits present/absent via lookback)
2. Context enrichment (config and git_info for skip logic)
3. Force bypass (filter rules before evaluation)
4. Existing rule updates (context field changes)

**Methodology:** TDD for rule evaluation logic. Use existing test helpers (test_phase_metrics_with, test_git_commit). Test core behavior, skip hypothetical edge cases.

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

Follow existing test pattern from RepeatedCommand variant tests.

### Step 1.b: Implement

Add RequireCommits variant to RuleConfig enum in src/rules/types.rs:
- Add variant with lookback_phases usize field
- Add RequireCommits arm to validate method checking lookback_phases >= 1
- Add test module following existing pattern
- Error message format: "require_commits.lookback_phases must be >= 1"

### Success Criteria

- RuleConfig deserializes require_commits from YAML
- Validation rejects lookback_phases == 0 with clear error
- Validation accepts lookback_phases from 1 to 999+
- Serialization roundtrip preserves value
- Tests follow existing patterns (no new test infrastructure needed)

**Commit Point:** `feat(rules): add RequireCommits rule type with validation`

---

## Step 2: Extend RuleEvaluationContext

### Goal
Add full phase array, config, and git_info to context for lookback and skip logic.

### Step 2.a: Write Tests

Describe test strategy for context changes:
- Test existing rules still work with all_phase_metrics array (find current phase)
- Test rules handle empty phase array gracefully
- Test config and git_info accessible from context
- No new helpers needed - update existing ctx_with_cmds, ctx_with_edits, ctx_with_phase patterns

### Step 2.b: Implement

Update RuleEvaluationContext in src/rules/types.rs:
- Change phase_metrics from Option single to all_phase_metrics slice
- Add config field (reference to HegelConfig)
- Add git_info field (Option reference to GitInfo)
- Update doc comments

Update all rule evaluators in src/rules/evaluator.rs:
- Find current phase in all_phase_metrics array by matching current_phase name
- Update repeated_command, repeated_file_edit, phase_timeout, token_budget evaluators
- Use find to locate current phase where phase_name matches current_phase

Update context construction in src/engine/mod.rs (line 227 area):
- Load config: HegelConfig::load(state_dir)
- Load state to get git_info
- Pass full metrics.phase_metrics slice instead of single phase
- Pass config and git_info references

Update test helper functions in src/rules/evaluator.rs tests:
- ctx_with_cmds, ctx_with_edits, ctx_with_phase to create full phase array
- Use test_phase_metrics_with(true) for minimal phases
- Add config and git_info (can use defaults initially)

### Success Criteria

- RuleEvaluationContext has three new/changed fields
- All existing rules compile and pass tests
- Context construction in engine loads config and state
- No regressions in existing rule behavior
- Test helpers updated to use phase array pattern

**Commit Point:** `refactor(rules): extend context with phase array, config, git_info`

---

## Step 3: Implement Require Commits Rule Evaluation

### Goal
Core logic to check for commits in lookback window.

### Step 3.a: Write Tests

Describe test strategy for commit checking:
- Use test_git_commit helper from test_helpers/archive.rs
- Use test_phase_metrics_with to create phases with git_commits populated
- Test single phase with commits returns no violation
- Test single phase without commits returns violation
- Test lookback_phases 2 combines git_commits from two phases
- Test lookback exceeding available phases uses all phases (graceful)
- Test violation message format includes lookback count
- Test config.commit_guard false skips rule
- Test git_info.has_repo false skips rule
- Test use_git Some(false) override skips rule
- Test use_git Some(true) override enables rule

Follow existing evaluator test patterns using time, phase, and context helpers.

### Step 3.b: Implement

Add evaluate_require_commits function in src/rules/evaluator.rs:
- Check context.config.commit_guard early return if false
- Check git availability via context.git_info and context.config.use_git
- Find current phase index in all_phase_metrics by name matching
- Calculate lookback start: max(0, current_index - lookback_phases + 1)
- Collect git_commits from all_phase_metrics[start..=current]
- Return RuleViolation if combined commits empty
- Return None if any commits found
- Add to evaluate_rules match statement (line 14 area)
- RuleViolation diagnostic: "No commits found in last N phases"
- RuleViolation suggestion: "Create a commit before advancing. Use `hegel next --force require_commits` to override."

### Success Criteria

- Rule evaluation combines commits across phases
- Lookback window handles 0 available phases gracefully
- Config integration skips when commit_guard false
- Git info integration skips when no repo
- use_git override works both directions
- Violation messages match spec format
- Tests use existing helpers (no new infrastructure)

**Commit Point:** `feat(rules): implement require_commits evaluation with config integration`

---

## Step 4: Add Config Fields

### Goal
Extend HegelConfig with commit_guard and use_git toggles.

### Step 4.a: Write Tests

Describe test strategy for config extension:
- Follow existing config test patterns (test_default_config, test_save_and_load_roundtrip)
- Test default config has commit_guard true and use_git None
- Test set and get for commit_guard (true/false)
- Test set and get for use_git (true/false/unset)
- Test list includes new fields
- Test save/load roundtrip preserves values
- Test invalid values rejected with clear errors

### Step 4.b: Implement

Update HegelConfig in src/config.rs:
- Add commit_guard bool field
- Add use_git Option bool field
- Update Default impl: commit_guard true, use_git None
- Extend get method with commit_guard and use_git keys
- Extend set method with boolean parsing for both fields
- Handle use_git unset by parsing empty string or "none" to None
- Extend list method to include new fields
- Update existing tests, add new tests for new fields

### Success Criteria

- Config struct has two new fields
- Defaults match spec (commit_guard true, use_git None)
- Get/set/list commands work for new fields
- Roundtrip serialization preserves values
- Tests follow existing patterns in src/config.rs

**Commit Point:** `feat(config): add commit_guard and use_git toggles`

---

## Step 5: Add Force Flag to CLI

### Goal
Support force bypass for all rules or specific rule types.

### Step 5.a: Write Tests

Describe test strategy for force flag:
- Test force with no argument bypasses all rules
- Test force with require_commits bypasses only that type
- Test force with phase_timeout bypasses only that type
- Test non-matching rules still evaluated
- Test multiple rules with selective force

CLI parsing tests in main or commands module. Integration via engine tests.

### Step 5.b: Implement

Update CLI in src/main.rs:
- Add force field to Next subcommand: Option<Option<String>>
- Document in help text with examples
- Pass force flag through commands::handle_next

Update get_next_prompt in src/engine/mod.rs (line 174):
- Add force_bypass optional parameter
- Filter node.rules before calling evaluate_rules
- If force_bypass is Some(None): skip all rules (empty vec)
- If force_bypass is Some(Some(type)): filter out matching rule types
- Match by stringifying rule type (require_commits, phase_timeout, etc)
- Pass filtered rules to evaluate_rules

Update evaluate_transition in src/commands/workflow/transitions.rs:
- Pass force flag to get_next_prompt call

### Success Criteria

- CLI accepts --force and --force <type>
- Force with no argument skips all rule evaluation
- Force with type name filters specific rules only
- Help text documents usage
- Tests verify filtering logic works
- Integration test demonstrates full flow

**Commit Point:** `feat(cli): add force flag with selective rule bypass`

---

## Step 6: Integration and Validation

### Goal
Ensure all components work together end-to-end.

### Step 6.a: Write Tests

Describe integration test strategy:
- Workflow validation already works (RuleConfig::validate called on load)
- Add engine test with require_commits rule in workflow YAML
- Test hegel next blocks without commits
- Test hegel next succeeds with commits
- Test hegel next --force bypasses rule
- Test config disable bypasses rule

### Step 6.b: Implement

Add integration tests in src/engine/mod.rs test module:
- Create workflow YAML string with require_commits rule
- Use existing test pattern from engine tests
- Load workflow, verify validation catches lookback_phases 0
- Create test with PhaseMetrics including git_commits
- Verify transition logic respects rule
- No implementation changes needed (already integrated)

### Success Criteria

- Workflow validation rejects invalid lookback_phases
- Integration tests demonstrate full feature flow
- All cargo test passes
- No regressions in existing workflows

**Commit Point:** `test(rules): add integration tests for require_commits`

---

## Final Success Criteria

Build succeeds: cargo build

Tests pass: cargo test (no regressions, all new tests pass)

Modified files compile:
- src/rules/types.rs - RequireCommits variant, context extended
- src/rules/evaluator.rs - Evaluation function, context updates
- src/config.rs - Two new config fields
- src/engine/mod.rs - Context construction, force filtering
- src/main.rs - Force flag on Next command
- src/commands/workflow/transitions.rs - Pass force to engine

Integration verified via automated tests:
- Config roundtrip tests verify commit_guard and use_git persistence
- Rule evaluation tests verify commits required/skipped based on config
- Engine tests verify workflow transitions block/allow based on commits
- Force flag tests verify selective and full rule bypass
- All verification agent-executable via cargo test

Test helpers used (no new infrastructure):
- test_phase_metrics_with(minimal) for phases
- test_git_commit(timestamp) for commits
- Existing time, cmd, edit, phase, ctx_* patterns
- TempDir for filesystem tests
