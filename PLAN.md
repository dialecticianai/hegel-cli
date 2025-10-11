# Workflow Rules System - Implementation Plan

**Goal**: Implement deterministic guardrails that interrupt workflow execution when anti-patterns are detected

**Scope**: Four rule types (repeated_command, repeated_file_edit, phase_timeout, token_budget) with YAML configuration, regex pattern matching, and decision checkpoint interrupts

**Methodology**: Strict TDD discipline (Red → Green → Commit), autonomous implementation, ≥90% test coverage

---

## TDD Discipline

- **Red → Green → Commit**: Write failing tests first, implement minimal code to pass, commit immediately
- **Commit format**: `type(scope): Step N - description` (e.g., `feat(rules): Step 1 - rule type deserialization`)
- **Coverage target**: ≥90% for all new code in `src/rules/*`
- **Test-first**: Every feature must have tests before implementation
- **Incremental**: Each step builds on previous, no skipping ahead

---

## Architecture Review (Context for Implementation)

## Implementation Review Notes

### Architecture Overview

**Integration Point**: `engine::get_next_prompt()` (src/engine/mod.rs:57-96)
- Current flow: Load node → Evaluate transitions → Build new state → Return prompt
- **New flow**: Load node → Evaluate transitions → Build new state → **Evaluate rules → Modify prompt if violation** → Return prompt

**Data Access**:
- Metrics via `parse_unified_metrics(state_dir)` returns `UnifiedMetrics`
- UnifiedMetrics contains: hook_metrics, token_metrics, state_transitions, phase_metrics
- PhaseMetrics has all per-phase data needed for rule evaluation

**Key Files to Modify**:
1. `src/engine/mod.rs` - Add `rules` field to `Node` struct, integrate rule evaluation in `get_next_prompt()`
2. Create `src/rules/mod.rs` - New module for rule types, evaluation, interrupt generation
3. `Cargo.toml` - Add `regex` dependency for pattern matching
4. Workflow YAMLs - Add optional `rules` field to node definitions (backward compatible)

---

## Data Structure Extensions

### 1. Node Struct Extension (src/engine/mod.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub prompt: String,
    pub transitions: Vec<Transition>,

    // New: optional rules field (backward compatible)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<RuleConfig>,
}
```

### 2. Rule Types (new: src/rules/mod.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    RepeatedCommand {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,  // Regex pattern
        threshold: usize,
        window: u64,  // seconds
    },
    RepeatedFileEdit {
        #[serde(skip_serializing_if = "Option::is_none")]
        path_pattern: Option<String>,  // Regex pattern
        threshold: usize,
        window: u64,
    },
    PhaseTimeout {
        max_duration: u64,  // seconds
    },
    TokenBudget {
        max_tokens: u64,
    },
}

pub struct RuleViolation {
    pub rule_type: String,
    pub diagnostic: String,
    pub suggestion: String,
    pub recent_events: Vec<String>,  // Last 5 events for context
}
```

### 3. Evaluation Context

```rust
pub struct RuleEvaluationContext<'a> {
    pub current_phase: &'a str,
    pub phase_start_time: &'a str,
    pub phase_metrics: Option<&'a PhaseMetrics>,  // Current phase metrics
    pub hook_metrics: &'a HookMetrics,  // All hooks (for time window filtering)
}
```

---

## Integration Strategy

### get_next_prompt() Modification

**Current** (src/engine/mod.rs:57-96):
```rust
pub fn get_next_prompt(
    workflow: &Workflow,
    state: &WorkflowState,
    claims: &HashMap<String, bool>,
) -> Result<(String, WorkflowState)> {
    // ... evaluate transitions, build new state ...
    Ok((next_node_obj.prompt.clone(), new_state))
}
```

**New** (with rule evaluation):
```rust
pub fn get_next_prompt(
    workflow: &Workflow,
    state: &WorkflowState,
    claims: &HashMap<String, bool>,
    state_dir: &Path,  // NEW: for metrics access
) -> Result<(String, WorkflowState)> {
    // ... existing transition logic ...

    // NEW: Evaluate rules for resulting node
    let prompt = if !next_node_obj.rules.is_empty() {
        let metrics = parse_unified_metrics(state_dir)?;
        let context = build_evaluation_context(&new_state, &metrics)?;

        if let Some(violation) = evaluate_rules(&next_node_obj.rules, &context)? {
            // Interrupt REPLACES normal prompt (decision checkpoint)
            generate_interrupt_prompt(&violation, &new_state.current_node)
        } else {
            next_node_obj.prompt.clone()
        }
    } else {
        next_node_obj.prompt.clone()
    };

    Ok((prompt, new_state))
}
```

**Signature Changes Cascade**:
- `commands::next_prompt()` needs to pass `state_dir` to `get_next_prompt()`
- Already has access via `FileStorage` parameter

**New Command: `hegel continue`** (src/commands/workflow.rs):
```rust
pub fn continue_prompt(storage: &FileStorage) -> Result<()> {
    // Load current state
    let state = storage.load()?;
    let workflow = parse_workflow_from_state(&state)?;
    let workflow_state = state.workflow_state
        .context("No workflow state found")?;

    // Get current node prompt (no transition, no rules)
    let node = workflow.nodes.get(&workflow_state.current_node)
        .context("Current node not found")?;

    // Render and display
    let rendered = render_template(&node.prompt, Path::new("guides"), &HashMap::new())?;
    println!("{}", "Continuing from interrupt".yellow());
    println!("{}: {}", "Current node".bold(), workflow_state.current_node);
    println!("{}", "Prompt:".bold().cyan());
    println!("{}", rendered);

    Ok(())
}
```

---

## Module Structure

### src/rules/mod.rs (new file, ~300 lines)

**Exports**:
```rust
pub use types::{RuleConfig, RuleViolation};
pub use evaluator::evaluate_rules;
pub use interrupt::generate_interrupt_prompt;

mod types;      // Rule enum, violation struct, context struct
mod evaluator;  // Core evaluation logic with regex matching
mod interrupt;  // Prompt formatting with diagnostics
```

**Submodules**:
1. `types.rs` (~100 lines) - Data structures, serde derives
2. `evaluator.rs` (~150 lines) - Rule evaluation, regex compilation, time window filtering
3. `interrupt.rs` (~50 lines) - Interrupt prompt generation with formatting

---

## Test Strategy

### Test Helpers Extension (src/test_helpers.rs)

**Add rule builders**:
```rust
pub fn repeated_command_rule(pattern: Option<&str>, threshold: usize, window: u64) -> RuleConfig {
    RuleConfig::RepeatedCommand {
        pattern: pattern.map(String::from),
        threshold,
        window,
    }
}

pub fn token_budget_rule(max_tokens: u64) -> RuleConfig {
    RuleConfig::TokenBudget { max_tokens }
}

// etc for other rule types
```

**Node builder with rules**:
```rust
pub fn node_with_rules(prompt: &str, transitions: Vec<Transition>, rules: Vec<RuleConfig>) -> Node {
    Node {
        prompt: prompt.to_string(),
        transitions,
        rules,
    }
}
```

### Test Organization

**Unit tests** (co-located in src/rules/*.rs):
- Rule YAML deserialization (types.rs)
- Regex pattern compilation and matching (evaluator.rs)
- Time window filtering logic (evaluator.rs)
- Interrupt prompt formatting (interrupt.rs)

**Integration tests** (src/engine/mod.rs):
- get_next_prompt() with rules that trigger
- get_next_prompt() with rules that don't trigger
- get_next_prompt() with no rules (backward compat)
- get_next_prompt() with invalid regex (error handling)

---

## Dependencies

### Cargo.toml additions

```toml
[dependencies]
regex = "1.10"  # Pattern matching for command/file rules
```

**Existing dependencies we'll use**:
- `serde` / `serde_yaml` - Rule deserialization
- `chrono` - Timestamp parsing for time windows
- `anyhow` - Error handling

---

## Workflow YAML Schema

### Example: discovery.yaml with rules

```yaml
nodes:
  code:
    prompt: |
      You are in the CODE phase...
    rules:
      - type: repeated_command
        pattern: "cargo (build|test)"
        threshold: 5
        window: 120

      - type: repeated_file_edit
        path_pattern: "src/.*\\.rs"
        threshold: 8
        window: 180

      - type: token_budget
        max_tokens: 5000

    transitions:
      - when: code_complete
        to: learnings
```

**Backward Compatibility**:
- `rules` field is optional (via `#[serde(default)]`)
- Existing workflows without `rules` continue to work unchanged
- Empty `rules: []` is valid (no rules enforced)

---

## Implementation Phases (TDD)

### Phase 1: Rule Types & Deserialization
- Define RuleConfig enum with serde
- Test YAML deserialization for all rule types
- Test validation (threshold > 0, window > 0, regex compiles)
- **Deliverable**: Rules parse from YAML correctly

### Phase 2: Evaluation Logic
- Implement repeated_command matching with regex
- Implement repeated_file_edit matching with regex
- Implement phase_timeout calculation
- Implement token_budget check
- Test time window filtering
- **Deliverable**: Rules evaluate against metrics correctly

### Phase 3: Interrupt Generation
- Format diagnostic messages
- Include recent events (last 5)
- Generate suggestions per rule type
- Add "REFLECT AND DECIDE" checkpoint
- Provide autonomous fix vs human escalation guidance
- **Deliverable**: Violations generate decision checkpoint prompts (replaces normal prompt)

### Phase 4: Continue Command
- Implement `hegel continue` in commands/workflow.rs
- Similar to `next_prompt()` but:
  - No claims parsing (no parameters)
  - Skip rule evaluation (get current node prompt directly)
  - No state transition (stay at current node)
- Add subcommand to main.rs CLI
- **Deliverable**: `hegel continue` bypasses rules and returns normal prompt

### Phase 5: Engine Integration
- Add rules field to Node struct
- Modify get_next_prompt() signature
- Integrate evaluation call
- Update all callers (commands/workflow.rs)
- **Deliverable**: End-to-end workflow with rule interrupts

### Phase 6: Workflow Updates
- Add rules to discovery.yaml CODE phase
- Add rules to execution.yaml CODE phase
- Test with real workflow execution
- **Deliverable**: Production workflows have guardrails

---

## Open Questions / Decisions Needed

1. **Regex validation timing**:
   - Option A: Validate at workflow load (fail fast, but errors on startup)
   - Option B: Validate at first evaluation (lazy, but runtime errors)
   - **Recommendation**: Option A (fail fast, better UX)

2. **Multiple rule violations**:
   - SPEC says "first violation wins" (short-circuit)
   - Confirm: Do we ever want to show multiple violations?
   - **Current decision**: First match only (simpler, less overwhelming)

3. **Rule state persistence**:
   - Do violations get logged to states.jsonl?
   - Could enable "how many times did rules trigger?" metrics
   - **Current decision**: No logging (keep rules stateless), but revisit in Phase 3

4. **Interrupt prompt caching**:
   - Should interrupt prompts be cached to avoid re-evaluation?
   - get_next_prompt() called once per `hegel next` invocation
   - **Current decision**: No caching (single call, not worth complexity)

---

## Coverage Target

**Goal**: ≥90% coverage for src/rules/*

**Critical paths to test**:
- All rule types evaluate correctly
- Regex patterns match expected commands/files
- Time window filtering excludes out-of-range events
- Interrupt prompts format correctly
- Backward compatibility (nodes without rules)
- Error handling (invalid regex, missing metrics)

---

## Next Steps

1. User reviews SPEC.md and provides feedback
2. Finalize any open questions/decisions
3. Convert these notes into full TDD PLAN.md with:
   - Step-by-step implementation (Red → Green → Commit)
   - Detailed test strategies per step
   - Code patterns for guidance (not literal code)
   - Success criteria checkboxes
4. Execute plan autonomously with TDD discipline

---

**Notes for AI execution**:
- Test helpers already support workflow builders, JSONL files, metrics builders
- Follow existing patterns in src/metrics/* for module organization
- Use `#[serde(tag = "type")]` for rule enum discrimination (matches YAML `type:` field)
- Regex compilation errors should surface at workflow load time (fail fast)
- All timestamps use chrono::DateTime::parse_from_rfc3339 for consistency

---

---

# IMPLEMENTATION STEPS (TDD)

---

## Step 1: Rule Type Definitions & YAML Deserialization

### Goal

Define the core `RuleConfig` enum with all four rule types and ensure YAML deserialization works correctly. This establishes the data model foundation for the entire rules system.

### Step 1.a: Write Tests

**Test Strategy**: Focus on YAML parsing using serde_yaml, covering all rule types and error cases

**Core test cases**:
- Test 1: Deserialize repeated_command rule with all fields
- Test 2: Deserialize repeated_command rule with optional pattern=None
- Test 3: Deserialize repeated_file_edit rule with all fields
- Test 4: Deserialize repeated_file_edit rule with optional path_pattern=None
- Test 5: Deserialize phase_timeout rule
- Test 6: Deserialize token_budget rule
- Test 7: Deserialize rule with unknown type returns error
- Test 8: Deserialize rule with missing required field returns error

**Expected validation**: YAML parsing should fail gracefully with descriptive errors for malformed input

**Test file location**: `src/rules/types.rs` (co-located with implementation)

### Step 1.b: Implement

**Tasks**:
1. Create `src/rules/` directory and `mod.rs` with public exports
2. Create `src/rules/types.rs` with RuleConfig enum
3. Add serde derives and tag-based discrimination
4. Implement Display trait for error messages
5. Update `src/main.rs` to include rules module

**Module structure** (src/rules/mod.rs):
```rust
mod types;
pub use types::{RuleConfig, RuleViolation, RuleEvaluationContext};
```

**RuleConfig enum pattern** (src/rules/types.rs):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    RepeatedCommand {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        threshold: usize,
        window: u64,
    },
    // ... other variants
}
```

**Error handling**: Use descriptive error messages from serde for malformed YAML

### Success Criteria

- [ ] RuleConfig enum defined with all four rule types
- [ ] All rule types deserialize from YAML correctly
- [ ] Optional fields (pattern, path_pattern) work with Some() and None
- [ ] Unknown rule types error with helpful message
- [ ] Missing required fields error with helpful message
- [ ] All tests in Step 1.a pass
- [ ] Commit: `feat(rules): Step 1 - rule type definitions and YAML deserialization`

---

## Step 2: Regex Pattern Validation

### Goal

Ensure regex patterns compile successfully at deserialization time (fail fast). Invalid patterns should error immediately when loading workflow YAML, not at runtime during evaluation.

### Step 2.a: Write Tests

**Test Strategy**: Focus on regex::Regex::new() validation, both success and failure cases

**Core test cases**:
- Test 1: Valid regex pattern compiles successfully
- Test 2: Invalid regex pattern (unclosed bracket) returns error
- Test 3: Invalid regex pattern (unclosed parenthesis) returns error
- Test 4: Complex regex pattern (cargo (build|test)) compiles successfully
- Test 5: File path regex pattern (src/.*\.rs) compiles successfully
- Test 6: None pattern (no validation needed) succeeds

**Expected validation**: Validation should happen during deserialization, patterns stored as compiled Regex objects

**Test file location**: `src/rules/types.rs`

### Step 2.b: Implement

**Tasks**:
1. Change RuleConfig to store compiled Regex instead of String
2. Implement custom Deserialize for RuleConfig to validate patterns
3. Add regex crate to Cargo.toml dependencies
4. Wrap Regex in Option<CompiledPattern> wrapper type for serializability

**Pattern validation approach**:
```rust
// Wrapper type for compiled regex (allows clone/debug)
#[derive(Clone)]
pub struct CompiledPattern(pub Regex);

impl CompiledPattern {
    pub fn new(pattern: &str) -> Result<Self> {
        Regex::new(pattern)
            .map(CompiledPattern)
            .context(format!("Invalid regex pattern: {}", pattern))
    }
}

// Custom deserialize for repeated_command
fn deserialize_pattern(s: Option<&str>) -> Result<Option<CompiledPattern>> {
    match s {
        Some(pattern) => CompiledPattern::new(pattern).map(Some),
        None => Ok(None),
    }
}
```

**Alternative simpler approach** (if custom deserialize is complex): Keep String in RuleConfig, add separate `validate_rules()` function called after workflow load

### Success Criteria

- [ ] Regex patterns validated at deserialization time
- [ ] Invalid patterns error immediately with helpful message
- [ ] Valid patterns compile successfully
- [ ] Complex patterns (with groups, escapes) handle correctly
- [ ] Optional patterns (None) don't trigger validation
- [ ] All tests in Step 2.a pass
- [ ] Commit: `feat(rules): Step 2 - regex pattern validation`

---

## Step 3: Repeated Command Rule Evaluation

### Goal

Implement the evaluation logic for repeated_command rules, including time window filtering and regex matching against bash commands from metrics.

### Step 3.a: Write Tests

**Test Strategy**: Use test metrics builders to create hook data, test filtering and counting logic

**Core test cases**:
- Test 1: Command repeated 5 times in window triggers rule (threshold=5, window=120s)
- Test 2: Command repeated 4 times in window doesn't trigger (threshold=5)
- Test 3: Command repeated 6 times but outside window doesn't trigger
- Test 4: Regex pattern matches multiple commands ("cargo (build|test)" matches both)
- Test 5: Regex pattern excludes non-matching commands
- Test 6: No pattern (None) matches all commands
- Test 7: Empty bash commands list doesn't trigger
- Test 8: Commands at exact window boundary (edge case)

**Expected behavior**: Return Some(RuleViolation) when threshold exceeded, None otherwise

**Test file location**: `src/rules/evaluator.rs`

### Step 3.b: Implement

**Tasks**:
1. Create `src/rules/evaluator.rs` module
2. Implement `evaluate_repeated_command()` function
3. Implement time window filtering helper
4. Implement regex matching against bash commands
5. Implement RuleViolation construction with diagnostic details

**Evaluation pattern**:
```rust
fn evaluate_repeated_command(
    rule: &RepeatedCommandRule,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    // 1. Filter commands by time window
    let cutoff_time = calculate_cutoff(context.phase_start_time, rule.window)?;
    let recent_commands = filter_by_timestamp(&context.hook_metrics.bash_commands, &cutoff_time);

    // 2. Filter by regex pattern (if provided)
    let matching_commands = if let Some(ref pattern) = rule.pattern {
        recent_commands.filter(|cmd| pattern.is_match(&cmd.command))
    } else {
        recent_commands  // All commands match
    };

    // 3. Count and check threshold
    let count = matching_commands.len();
    if count >= rule.threshold {
        Ok(Some(build_violation("repeated_command", count, rule.threshold, matching_commands)))
    } else {
        Ok(None)
    }
}
```

**Time window helper**:
```rust
fn calculate_cutoff(start_time: &str, window_secs: u64) -> Result<DateTime<FixedOffset>> {
    let start = DateTime::parse_from_rfc3339(start_time)?;
    Ok(start - Duration::seconds(window_secs as i64))
}
```

### Success Criteria

- [ ] Repeated command counting works correctly
- [ ] Time window filtering excludes old commands
- [ ] Regex pattern matching filters commands
- [ ] None pattern matches all commands
- [ ] RuleViolation constructed with correct diagnostic
- [ ] All tests in Step 3.a pass
- [ ] Commit: `feat(rules): Step 3 - repeated command evaluation`

---

## Step 4: Repeated File Edit Rule Evaluation

### Goal

Implement evaluation logic for repeated_file_edit rules, mirroring repeated_command but for file modifications.

### Step 4.a: Write Tests

**Test Strategy**: Similar to Step 3, but using file_modifications from metrics

**Core test cases**:
- Test 1: File edited 8 times in window triggers rule (threshold=8, window=180s)
- Test 2: File edited 7 times doesn't trigger (threshold=8)
- Test 3: Edits outside window excluded
- Test 4: Path pattern matches multiple files ("src/.*\.rs" matches src/main.rs, src/lib.rs)
- Test 5: Path pattern excludes non-matching files (README.md excluded)
- Test 6: No pattern matches all files
- Test 7: Empty file modifications list doesn't trigger

**Expected behavior**: Return Some(RuleViolation) when threshold exceeded, None otherwise

**Test file location**: `src/rules/evaluator.rs`

### Step 4.b: Implement

**Tasks**:
1. Implement `evaluate_repeated_file_edit()` function
2. Reuse time window filtering helper from Step 3
3. Implement path pattern matching against file_path field
4. Construct RuleViolation with recent file edits

**Implementation note**: Very similar to repeated_command, extract shared filtering logic if duplication emerges

### Success Criteria

- [ ] Repeated file edit counting works correctly
- [ ] Time window filtering excludes old edits
- [ ] Path regex pattern matching filters files
- [ ] None pattern matches all files
- [ ] RuleViolation constructed with correct diagnostic
- [ ] All tests in Step 4.a pass
- [ ] Commit: `feat(rules): Step 4 - repeated file edit evaluation`

---

## Step 5: Phase Timeout Rule Evaluation

### Goal

Implement evaluation logic for phase_timeout rules, calculating phase duration from timestamps.

### Step 5.a: Write Tests

**Test Strategy**: Focus on timestamp arithmetic and duration calculation

**Core test cases**:
- Test 1: Phase running for 11 minutes triggers rule (limit=600s)
- Test 2: Phase running for 9 minutes doesn't trigger (limit=600s)
- Test 3: Active phase (no end_time) calculates duration from current time
- Test 4: Completed phase uses actual duration
- Test 5: Phase duration of 0 doesn't trigger

**Expected behavior**: Return Some(RuleViolation) when duration > max_duration, None otherwise

**Test file location**: `src/rules/evaluator.rs`

### Step 5.b: Implement

**Tasks**:
1. Implement `evaluate_phase_timeout()` function
2. Calculate phase duration from start/end timestamps
3. Handle active phases (end_time=None) using current time
4. Construct RuleViolation with timing diagnostics

**Duration calculation pattern**:
```rust
fn evaluate_phase_timeout(
    rule: &PhaseTimeoutRule,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let phase_metrics = context.phase_metrics?;  // Current phase

    let duration_secs = if let Some(ref end_time) = phase_metrics.end_time {
        // Completed phase
        calculate_duration(&phase_metrics.start_time, end_time)?
    } else {
        // Active phase - use current time
        let now = Utc::now().to_rfc3339();
        calculate_duration(&phase_metrics.start_time, &now)?
    };

    if duration_secs > rule.max_duration {
        Ok(Some(build_timeout_violation(duration_secs, rule.max_duration, &phase_metrics.start_time)))
    } else {
        Ok(None)
    }
}
```

### Success Criteria

- [ ] Phase duration calculated correctly for completed phases
- [ ] Active phase duration uses current time
- [ ] Timeout detection triggers at correct threshold
- [ ] RuleViolation includes timing details
- [ ] All tests in Step 5.a pass
- [ ] Commit: `feat(rules): Step 5 - phase timeout evaluation`

---

## Step 6: Token Budget Rule Evaluation

### Goal

Implement evaluation logic for token_budget rules, summing input and output tokens from phase metrics.

### Step 6.a: Write Tests

**Test Strategy**: Focus on token aggregation from phase metrics

**Core test cases**:
- Test 1: Tokens exceed budget triggers rule (6000 total, limit=5000)
- Test 2: Tokens under budget doesn't trigger (4000 total, limit=5000)
- Test 3: Tokens exactly at budget doesn't trigger (5000 total, limit=5000)
- Test 4: Empty token metrics (0 tokens) doesn't trigger
- Test 5: Input tokens only counted correctly
- Test 6: Output tokens only counted correctly

**Expected behavior**: Return Some(RuleViolation) when total > max_tokens, None otherwise

**Test file location**: `src/rules/evaluator.rs`

### Step 6.b: Implement

**Tasks**:
1. Implement `evaluate_token_budget()` function
2. Sum input_tokens + output_tokens from phase metrics
3. Compare against max_tokens threshold
4. Construct RuleViolation with token breakdown

**Token aggregation pattern**:
```rust
fn evaluate_token_budget(
    rule: &TokenBudgetRule,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let phase_metrics = context.phase_metrics?;

    let total_tokens = phase_metrics.token_metrics.total_input_tokens
        + phase_metrics.token_metrics.total_output_tokens;

    if total_tokens > rule.max_tokens {
        Ok(Some(build_budget_violation(total_tokens, rule.max_tokens)))
    } else {
        Ok(None)
    }
}
```

### Success Criteria

- [ ] Token counting aggregates input + output correctly
- [ ] Budget threshold detection works
- [ ] RuleViolation includes token breakdown
- [ ] All tests in Step 6.a pass
- [ ] Commit: `feat(rules): Step 6 - token budget evaluation`

---

## Step 7: Unified Rule Evaluation Orchestration

### Goal

Implement the main `evaluate_rules()` function that iterates through all rules and returns the first violation (short-circuit).

### Step 7.a: Write Tests

**Test Strategy**: Integration tests with multiple rules, test short-circuit behavior

**Core test cases**:
- Test 1: First rule triggers, returns first violation only
- Test 2: Second rule triggers, first rule passes
- Test 3: No rules trigger, returns None
- Test 4: Multiple rules trigger, returns first only (short-circuit)
- Test 5: Empty rules list returns None
- Test 6: Mix of all rule types, correct one triggers

**Expected behavior**: Return Option<RuleViolation>, short-circuit on first match

**Test file location**: `src/rules/evaluator.rs`

### Step 7.b: Implement

**Tasks**:
1. Implement `evaluate_rules()` main entry point
2. Dispatch to type-specific evaluators (match on RuleConfig enum)
3. Short-circuit on first violation
4. Build RuleEvaluationContext from metrics and state

**Orchestration pattern**:
```rust
pub fn evaluate_rules(
    rules: &[RuleConfig],
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    for rule in rules {
        let violation = match rule {
            RuleConfig::RepeatedCommand { .. } => evaluate_repeated_command(rule, context)?,
            RuleConfig::RepeatedFileEdit { .. } => evaluate_repeated_file_edit(rule, context)?,
            RuleConfig::PhaseTimeout { .. } => evaluate_phase_timeout(rule, context)?,
            RuleConfig::TokenBudget { .. } => evaluate_token_budget(rule, context)?,
        };

        if violation.is_some() {
            return Ok(violation);  // Short-circuit
        }
    }

    Ok(None)
}
```

### Success Criteria

- [ ] Iterates through all rules until first violation
- [ ] Short-circuits on first match (doesn't evaluate remaining rules)
- [ ] Returns None when no violations
- [ ] Dispatches to correct evaluator for each rule type
- [ ] All tests in Step 7.a pass
- [ ] Commit: `feat(rules): Step 7 - unified rule evaluation orchestration`

---

## Step 8: Interrupt Prompt Generation

### Goal

Generate human-readable interrupt prompts with diagnostics, suggestions, and the "REFLECT AND DECIDE" checkpoint.

### Step 8.a: Write Tests

**Test Strategy**: String formatting tests, verify structure and content

**Core test cases**:
- Test 1: Repeated command interrupt includes header, diagnostic, recent events, decision checkpoint
- Test 2: Repeated file edit interrupt formats correctly
- Test 3: Phase timeout interrupt includes timing details
- Test 4: Token budget interrupt includes token breakdown
- Test 5: Recent events limited to 5 most recent
- Test 6: Timestamps formatted as HH:MM:SS
- Test 7: Counts formatted with commas (e.g., "6,200")

**Expected behavior**: Return formatted String with all interrupt sections

**Test file location**: `src/rules/interrupt.rs`

### Step 8.b: Implement

**Tasks**:
1. Create `src/rules/interrupt.rs` module
2. Implement `generate_interrupt_prompt()` function
3. Implement formatters for timestamps, counts, recent events
4. Add rule-type-specific suggestion messages
5. Include "REFLECT AND DECIDE" section with `hegel continue` guidance

**Prompt template pattern**:
```rust
pub fn generate_interrupt_prompt(violation: &RuleViolation, current_phase: &str) -> String {
    format!(
        r#"🚨 WORKFLOW INTERRUPT: {}

Diagnostic: {}

{}

Suggestion: {}

---

REFLECT AND DECIDE:

Can you resolve this issue autonomously, or do you need human assistance?

If you can fix this yourself:
  - Explain your corrective approach briefly
  - Run: hegel continue
  - This will return the normal {} phase prompt (bypassing rules this once)

If you need human help:
  - Explain what you've tried and why you're stuck
  - Wait for human guidance before proceeding
"#,
        violation.rule_type,
        violation.diagnostic,
        format_recent_events(&violation.recent_events),
        violation.suggestion,
        current_phase.to_uppercase()
    )
}
```

**Formatting helpers**:
```rust
fn format_recent_events(events: &[String]) -> String {
    events.iter()
        .take(5)  // Limit to 5
        .map(|e| format!("  - {}", e))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_timestamp(iso: &str) -> String {
    // Extract HH:MM:SS from ISO 8601
    &iso[11..19]
}

fn format_count(n: u64) -> String {
    // Add commas: 6200 -> "6,200"
    n.to_string().as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}
```

### Success Criteria

- [ ] Interrupt prompt includes all required sections
- [ ] Diagnostic message is clear and specific
- [ ] Recent events limited to 5, formatted correctly
- [ ] Timestamps formatted as HH:MM:SS
- [ ] Counts formatted with commas
- [ ] "REFLECT AND DECIDE" section includes `hegel continue` guidance
- [ ] All tests in Step 8.a pass
- [ ] Commit: `feat(rules): Step 8 - interrupt prompt generation`

---

## Step 9: Continue Command Implementation

### Goal

Implement `hegel continue` command that returns normal phase prompt without evaluating rules.

### Step 9.a: Write Tests

**Test Strategy**: Integration tests using workflow environment, verify prompt returned

**Core test cases**:
- Test 1: Continue with active workflow returns current node prompt
- Test 2: Continue with no workflow loaded returns error
- Test 3: Continue renders template with guides correctly
- Test 4: Continue doesn't change workflow state
- Test 5: Continue doesn't log state transition

**Expected behavior**: Return current node's prompt, skip rules, no state changes

**Test file location**: `src/commands/workflow.rs`

### Step 9.b: Implement

**Tasks**:
1. Add `continue_prompt()` function to src/commands/workflow.rs
2. Load workflow and current state
3. Get current node prompt
4. Render template with guides
5. Display formatted output
6. Export from src/commands/mod.rs
7. Add CLI subcommand to src/main.rs

**Implementation pattern** (already outlined in preliminary notes):
```rust
pub fn continue_prompt(storage: &FileStorage) -> Result<()> {
    let state = storage.load()?;
    let workflow = parse_workflow_from_state(&state)?;
    let workflow_state = state.workflow_state
        .context("No workflow state found")?;

    let node = workflow.nodes.get(&workflow_state.current_node)
        .context("Current node not found")?;

    let rendered = render_template(&node.prompt, Path::new("guides"), &HashMap::new())?;

    println!("{}", "Continuing from interrupt".yellow());
    println!("{}: {}", "Current node".bold(), workflow_state.current_node);
    println!();
    println!("{}", "Prompt:".bold().cyan());
    println!("{}", rendered);

    Ok(())
}
```

**CLI integration** (src/main.rs):
```rust
enum Commands {
    // ... existing commands
    Continue,
}

// In match
Commands::Continue => {
    commands::continue_prompt(&storage)?;
}
```

### Success Criteria

- [ ] `hegel continue` command works
- [ ] Returns current node prompt
- [ ] Template rendering with guides works
- [ ] No state changes occur
- [ ] Error when no workflow loaded
- [ ] All tests in Step 9.a pass
- [ ] Commit: `feat(commands): Step 9 - continue command implementation`

---

## Step 10: Node Struct Extension for Rules

### Goal

Add optional `rules` field to Node struct and ensure backward compatibility with existing workflows.

### Step 10.a: Write Tests

**Test Strategy**: YAML workflow deserialization tests, verify backward compatibility

**Core test cases**:
- Test 1: Node with rules field deserializes correctly
- Test 2: Node without rules field deserializes (backward compat)
- Test 3: Node with empty rules list deserializes
- Test 4: Multiple rules in one node deserialize
- Test 5: Workflow with mixed nodes (some with rules, some without) works

**Expected behavior**: Optional rules field, defaults to empty vec

**Test file location**: `src/engine/mod.rs`

### Step 10.b: Implement

**Tasks**:
1. Add `rules` field to Node struct in src/engine/mod.rs
2. Add serde annotations for optional field
3. Update existing tests to handle new field
4. Test with actual workflow YAML files

**Node struct modification**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub prompt: String,
    pub transitions: Vec<Transition>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<RuleConfig>,
}
```

**Import addition** (src/engine/mod.rs):
```rust
use crate::rules::RuleConfig;
```

### Success Criteria

- [ ] Node struct has rules field
- [ ] Field is optional with default empty vec
- [ ] Existing tests still pass (backward compat)
- [ ] Can deserialize nodes with and without rules
- [ ] All tests in Step 10.a pass
- [ ] Commit: `feat(engine): Step 10 - add rules field to Node struct`

---

## Step 11: Engine Integration - get_next_prompt Modification

### Goal

Integrate rule evaluation into `get_next_prompt()`, evaluating rules before returning prompt and generating interrupt if violation detected.

### Step 11.a: Write Tests

**Test Strategy**: Engine integration tests with rules that trigger and don't trigger

**Core test cases**:
- Test 1: get_next_prompt with no rules returns normal prompt
- Test 2: get_next_prompt with rules that don't trigger returns normal prompt
- Test 3: get_next_prompt with rules that trigger returns interrupt prompt
- Test 4: get_next_prompt with multiple rules returns first violation
- Test 5: Rule evaluation uses current phase metrics
- Test 6: Invalid regex in rules errors at workflow load

**Expected behavior**: Return interrupt prompt when rule triggers, normal prompt otherwise

**Test file location**: `src/engine/mod.rs`

### Step 11.b: Implement

**Tasks**:
1. Modify `get_next_prompt()` signature to accept state_dir
2. After transition logic, evaluate rules if any exist
3. Build RuleEvaluationContext from metrics and current state
4. Call `evaluate_rules()` and `generate_interrupt_prompt()` if violation
5. Update all callers in src/commands/workflow.rs to pass state_dir

**Modified signature**:
```rust
pub fn get_next_prompt(
    workflow: &Workflow,
    state: &WorkflowState,
    claims: &HashMap<String, bool>,
    state_dir: &Path,  // NEW
) -> Result<(String, WorkflowState)>
```

**Rule evaluation integration** (pattern from preliminary notes):
```rust
// After transition logic, before returning...

// Evaluate rules for resulting node
let prompt = if !next_node_obj.rules.is_empty() {
    let metrics = parse_unified_metrics(state_dir)?;

    // Find current phase metrics
    let phase_metrics = metrics.phase_metrics
        .iter()
        .find(|p| p.phase_name == new_state.current_node && p.end_time.is_none());

    let context = RuleEvaluationContext {
        current_phase: &new_state.current_node,
        phase_start_time: phase_metrics.map(|p| p.start_time.as_str()).unwrap_or(""),
        phase_metrics,
        hook_metrics: &metrics.hook_metrics,
    };

    if let Some(violation) = evaluate_rules(&next_node_obj.rules, &context)? {
        generate_interrupt_prompt(&violation, &new_state.current_node)
    } else {
        next_node_obj.prompt.clone()
    }
} else {
    next_node_obj.prompt.clone()
};

Ok((prompt, new_state))
```

**Update callers** (src/commands/workflow.rs):
```rust
// In next_prompt() function
let (prompt_text, new_state) = get_next_prompt(
    &workflow,
    workflow_state,
    &claims,
    storage.state_dir(),  // NEW
)?;
```

### Success Criteria

- [ ] get_next_prompt() signature includes state_dir parameter
- [ ] Rules evaluated when present
- [ ] Interrupt prompt returned when rule triggers
- [ ] Normal prompt returned when no rules or no violations
- [ ] All callers updated (commands/workflow.rs)
- [ ] All tests in Step 11.a pass
- [ ] Integration test: full workflow with rule interrupt works
- [ ] Commit: `feat(engine): Step 11 - integrate rule evaluation into get_next_prompt`

---

## Step 12: Production Workflow Updates

### Goal

Add rules to discovery.yaml and execution.yaml CODE phases with sensible defaults for common anti-patterns.

### Step 12.a: Write Tests

**Test Strategy**: Workflow loading tests, verify rules parse correctly

**Core test cases**:
- Test 1: Load discovery.yaml with rules succeeds
- Test 2: Load execution.yaml with rules succeeds
- Test 3: Rules in CODE phase parse correctly
- Test 4: Start workflow and reach CODE phase with rules

**Expected behavior**: Workflows load successfully, rules ready for evaluation

**Test file location**: Integration test in src/commands/workflow.rs

### Step 12.b: Implement

**Tasks**:
1. Add rules section to discovery.yaml CODE node
2. Add rules section to execution.yaml CODE node
3. Use conservative thresholds (don't over-trigger)
4. Test workflow execution manually

**Discovery workflow CODE node** (workflows/discovery.yaml):
```yaml
  code:
    prompt: |
      You are in the CODE phase...
    rules:
      # Detect build/test loops
      - type: repeated_command
        pattern: "cargo (build|check|test)"
        threshold: 6
        window: 180

      # Detect file thrashing
      - type: repeated_file_edit
        path_pattern: "src/.*"
        threshold: 10
        window: 300

      # Token budget (generous for discovery)
      - type: token_budget
        max_tokens: 10000

    transitions:
      - when: code_complete
        to: learnings
```

**Execution workflow CODE node** (workflows/execution.yaml):
```yaml
  code:
    prompt: |
      You are in the CODE phase...
    rules:
      # Stricter thresholds for production
      - type: repeated_command
        pattern: "cargo (build|check|test)"
        threshold: 5
        window: 120

      - type: repeated_file_edit
        path_pattern: "src/.*"
        threshold: 8
        window: 180

      # Phase timeout (10 minutes)
      - type: phase_timeout
        max_duration: 600

      # Token budget (stricter for execution)
      - type: token_budget
        max_tokens: 8000

    transitions:
      - when: code_complete
        to: review
```

### Success Criteria

- [ ] discovery.yaml CODE node has rules
- [ ] execution.yaml CODE node has rules
- [ ] Workflows load without errors
- [ ] Rules parse correctly
- [ ] Manual test: workflow with rule trigger works end-to-end
- [ ] All tests in Step 12.a pass
- [ ] Commit: `feat(workflows): Step 12 - add rules to production workflows`

---

## Final Integration Testing & Documentation

### Post-Implementation Tasks

**After all steps complete**:

1. **Coverage check**: Run `./scripts/generate-coverage-report.sh`, verify ≥90% for src/rules/*
2. **Integration test**: Full workflow cycle with rule trigger → continue → complete
3. **Update CODE_MAP.md**: Add rules module documentation
4. **Update README.md**: Document `hegel continue` command
5. **Update ROADMAP.md**: Delete Phase 2 (completed), keep future phases only

**Final commit**: `docs: update documentation for rules system`

### Testing Checklist

- [ ] All unit tests pass (cargo test)
- [ ] Coverage ≥90% for src/rules/*
- [ ] Integration test: discovery workflow with rule interrupt
- [ ] Integration test: execution workflow with rule interrupt
- [ ] Manual test: `hegel continue` after interrupt
- [ ] Backward compatibility: workflows without rules still work

---

## Dependencies

Add to Cargo.toml before starting:
```toml
[dependencies]
regex = "1.10"
```

All other dependencies already present (serde, serde_yaml, chrono, anyhow, colored).

---

## Execution Notes

- Follow steps sequentially, no skipping ahead
- Each step is self-contained: tests → implementation → commit
- Use test_helpers.rs builders extensively (workflow(), metrics builders, JSONL creators)
- Pattern match existing code style from src/metrics/* and src/engine/*
- Keep functions focused and small (<100 lines)
- Extract helpers early if duplication emerges
- Commit immediately after each step completes (Red → Green → Commit)
