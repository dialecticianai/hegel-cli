# Commit Guardrails Specification

Prevent phase advancement without git commits when appropriate.

## Overview

**What it does:** Adds a `require_commits` rule type that blocks `hegel next` transitions unless commits exist in the lookback window. Provides global config toggle and selective force-bypass capability.

**Key principles:**
- **Declarative workflow control** - Workflows specify which phases need commits via YAML rules
- **Graceful degradation** - Works with or without git, configurable globally and per-workflow
- **Selective bypass** - Force flag can override all rules or just commit checks
- **Flexible lookback** - Check N phases back for commits (handles grouped planning/docs phases)

**Scope:**
- Add `require_commits` rule type to existing rules system
- Add `commit_guard` and `use_git` boolean config fields
- Modify `hegel next --force [rule_type]` to support selective bypass
- Update `RuleEvaluationContext` to provide full phase history
- Validate `lookback_phases >= 1` at workflow load time

**Integration context:**
- Leverages existing git commit attribution (`src/metrics/git.rs::attribute_commits_to_phases`)
- Extends existing rules system (`src/rules/`)
- Integrates with existing config system (`src/config.rs::HegelConfig`)
- Modifies transition logic (`src/commands/workflow/transitions.rs`)

## Data Model

### MODIFIED: RuleConfig (src/rules/types.rs)

Add new variant to existing enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    // ... existing variants (RepeatedCommand, RepeatedFileEdit, PhaseTimeout, TokenBudget)

    RequireCommits {
        /// Number of phases to check for commits (including current)
        /// Must be >= 1. Value of 999 acts as "check entire workflow history"
        lookback_phases: usize,
    },
}
```

**YAML schema:**
```yaml
rules:
  - type: require_commits
    lookback_phases: 2  # Check current + previous phase
```

**Validation:**
- `lookback_phases >= 1` (enforced in `RuleConfig::validate()`)
- No maximum (allows `lookback_phases: 999` for "entire workflow")
- Runtime graceful: if `lookback_phases > available_phases`, check all available phases

### MODIFIED: HegelConfig (src/config.rs)

Add two new fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HegelConfig {
    pub use_reflect_gui: bool,
    pub code_map_style: String,

    /// Global enable/disable for commit_guard rules
    /// When false, all require_commits rules are ignored
    pub commit_guard: bool,  // NEW - default true

    /// Override git repository detection
    /// When set, overrides state.git_info.has_repo detection
    pub use_git: Option<bool>,  // NEW - default None (use auto-detect)
}
```

**Config commands:**
```bash
hegel config set commit_guard false  # Disable globally
hegel config set use_git false       # Force disable git even if repo exists
hegel config get commit_guard        # Show current value
```

### MODIFIED: RuleEvaluationContext (src/rules/types.rs)

Change from single-phase to full-phase-array:

```rust
pub struct RuleEvaluationContext<'a> {
    pub current_phase: &'a str,
    pub phase_start_time: Option<&'a String>,

    // CHANGED: was Option<&'a PhaseMetrics> (single phase)
    // Now &'a [PhaseMetrics] (full array for lookback)
    pub all_phase_metrics: &'a [PhaseMetrics],

    pub hook_metrics: &'a crate::metrics::HookMetrics,
}
```

**Impact:** All existing rule evaluators must be updated to find current phase in array.

### MODIFIED: CLI Arguments (src/main.rs)

Update `hegel next` subcommand:

```rust
Next {
    /// Force bypass rules (all rules or specific type)
    /// Examples:
    ///   --force                  # Bypass all rules
    ///   --force require_commits  # Bypass only commit checks
    ///   --force phase_timeout    # Bypass only timeout rules
    #[arg(long)]
    force: Option<Option<String>>,  // None = no force, Some(None) = all, Some(Some(type)) = specific
}
```

**Usage:**
```bash
hegel next                      # Normal (rules enforced)
hegel next --force              # Bypass ALL rules
hegel next --force require_commits  # Bypass only commit checks
```

### NO CHANGES: PhaseMetrics, GitCommit, State

Git commit attribution already works (`PhaseMetrics.git_commits: Vec<GitCommit>` populated by existing code).

## Core Operations

### Operation: Evaluate require_commits Rule

**When:** During `hegel next`, before allowing phase transition (existing rule evaluation point in `src/engine/mod.rs:242`)

**Behavior:**
1. Check global config: if `commit_guard == false`, skip all require_commits rules (return None)
2. Check git availability:
   - If `config.use_git == Some(false)`, skip rule (return None)
   - If `state.git_info.has_repo == false` and `config.use_git != Some(true)`, skip rule (return None)
3. Find current phase in `all_phase_metrics` array
4. Collect phases for lookback window:
   - Current phase index = `i`
   - Lookback start index = `max(0, i - lookback_phases + 1)`
   - Phases to check = `all_phase_metrics[start..=i]`
5. Combine `git_commits` from all phases in window
6. If combined commits is empty, return `RuleViolation`
7. Otherwise, return `None` (no violation)

**RuleViolation schema:**
```rust
RuleViolation {
    rule_type: "Require Commits".to_string(),
    diagnostic: "No commits found in last N phases".to_string(),
    suggestion: "Create a commit before advancing. Use `hegel next --force require_commits` to override.".to_string(),
    recent_events: vec![], // Empty for this rule type
}
```

### Operation: Force Bypass

**Syntax:** `hegel next --force [rule_type]`

**Behavior:**
- `--force` (no argument) → Skip all rule evaluation
- `--force require_commits` → Filter out all `RuleConfig::RequireCommits` variants before evaluation
- `--force phase_timeout` → Filter out all `RuleConfig::PhaseTimeout` variants before evaluation
- etc.

**Implementation location:** `src/engine/mod.rs::get_next_prompt()` or wrapper in transitions.rs

### Operation: Config Management

**Existing commands extended:**
```bash
# Get values
hegel config get commit_guard      # → "true" or "false"
hegel config get use_git           # → "true", "false", or "(not set)"

# Set values
hegel config set commit_guard false
hegel config set use_git true

# List all
hegel config list  # Shows commit_guard and use_git with other settings
```

### Operation: Workflow Validation

**When:** Workflow YAML load (`src/engine/mod.rs::load_workflow_from_str`)

**Validation rules:**
- `require_commits.lookback_phases >= 1` (error if 0)
- No upper bound check (allow 999 for "check all phases")

**Error message:**
```
Invalid rule in node 'plan': require_commits.lookback_phases must be >= 1
```

## Test Scenarios

### Simple: Single-phase commit check

**Setup:**
- Workflow with code node: `rules: [{type: require_commits, lookback_phases: 1}]`
- User in code phase, made 1 commit
- Config: `commit_guard: true` (default)

**Action:** `hegel next`

**Expected:**
- Transition succeeds
- Prompt from next node displayed

### Simple: No commits blocks transition

**Setup:**
- Workflow with code node: `rules: [{type: require_commits, lookback_phases: 1}]`
- User in code phase, made 0 commits
- Config: `commit_guard: true`

**Action:** `hegel next`

**Expected:**
- Transition blocked
- Interrupt prompt displayed:
  ```
  ⚠️  Require Commits

  No commits found in last 1 phases

  Suggestion: Create a commit before advancing. Use `hegel next --force require_commits` to override.
  ```

### Complex: Multi-phase lookback (grouped phases)

**Setup:**
- Workflow: spec → plan → code
- Plan node has: `rules: [{type: require_commits, lookback_phases: 2}]`
- User transitions: spec → plan
- No commits made in spec or plan phases

**Action:** `hegel next` (attempting plan → code)

**Expected:**
- Transition blocked
- Diagnostic: "No commits found in last 2 phases"
- Suggestion mentions `--force require_commits`

**Follow-up:** User makes commit, runs `hegel next` again

**Expected:**
- Transition succeeds (commit found in lookback window)

### Complex: Force bypass specific rule

**Setup:**
- Node has two rules: `require_commits` and `phase_timeout`
- Timeout not exceeded, but no commits exist
- User wants to advance anyway

**Action:** `hegel next --force require_commits`

**Expected:**
- `require_commits` rule skipped
- `phase_timeout` rule still evaluated
- If timeout passes, transition succeeds

### Complex: Lookback exceeds phase history

**Setup:**
- Workflow just started (only in spec phase)
- Spec node has: `rules: [{type: require_commits, lookback_phases: 999}]`
- User made 1 commit

**Action:** `hegel next`

**Expected:**
- Rule checks only available phases (just spec)
- Finds 1 commit in spec phase
- Transition succeeds (graceful handling of lookback > phase count)

### Error: Config disables globally

**Setup:**
- Config: `commit_guard: false`
- Node has: `rules: [{type: require_commits, lookback_phases: 1}]`
- No commits exist

**Action:** `hegel next`

**Expected:**
- Rule evaluation skipped entirely (global disable)
- Transition succeeds despite no commits

### Error: No git repository

**Setup:**
- Not in a git repository (state.git_info.has_repo == false)
- Node has: `rules: [{type: require_commits, lookback_phases: 1}]`
- Config: `use_git` not set (None)

**Action:** `hegel next`

**Expected:**
- Rule evaluation skipped (no git repo detected)
- Transition succeeds
- No error messages (graceful skip)

### Error: Invalid workflow validation

**Setup:**
- Workflow YAML contains:
  ```yaml
  plan:
    rules:
      - type: require_commits
        lookback_phases: 0  # Invalid
  ```

**Action:** `hegel start execution`

**Expected:**
- Workflow fails to load
- Error message: "Invalid rule in node 'plan': require_commits.lookback_phases must be >= 1"

## Success Criteria

Tests pass: `cargo test`

New tests cover:
- Rule evaluation with commits present → no violation
- Rule evaluation with no commits → violation
- Lookback window combining multiple phases
- Lookback exceeding available phases (graceful)
- Global config disable (`commit_guard: false`) skips rule
- Git unavailable skips rule
- Force bypass all rules
- Force bypass specific rule type
- Workflow validation rejects `lookback_phases: 0`

Build succeeds: `cargo build`

Modified files compile:
- `src/rules/types.rs` - New `RequireCommits` variant
- `src/rules/evaluator.rs` - New evaluation function
- `src/config.rs` - Two new config fields
- `src/engine/mod.rs` - Updated context construction
- `src/main.rs` - CLI force flag argument
- `src/commands/workflow/transitions.rs` - Force flag handling (if needed)

Integration test: Manual workflow execution
- Start execution workflow with modified plan node (add require_commits rule)
- Verify transition blocks without commits
- Create commit, verify transition succeeds
- Test `--force require_commits` bypass

Config roundtrip:
- `hegel config set commit_guard false`
- `hegel config get commit_guard` → "false"
- State persists across sessions

CLI help text updated:
- `hegel next --help` shows `--force [RULE_TYPE]` usage
- `hegel config --help` mentions commit_guard and use_git

No regressions:
- Existing rules (repeated_command, phase_timeout, token_budget) still work
- Existing `hegel next` behavior unchanged when no force flag
- Workflows without require_commits rules unaffected
