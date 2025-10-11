# Workflow Rules System Specification

**Deterministic guardrails that interrupt workflow execution when anti-patterns are detected**

---

## Overview

### What it does
The rules system evaluates workflow metrics in real-time to detect anti-patterns (repeated commands, file churn, token burn, timeout). When rules trigger, the system injects diagnostic interrupts into the workflow prompt to guide course correction.

### Key principles
- **Deterministic enforcement**: Pure state-based rule evaluation, no LLM calls
- **Workflow-integrated**: Rules defined in workflow YAML, evaluated per-node
- **Regex-based precision**: Pattern matching for targeted detection (language-specific)
- **Transparent diagnostics**: Interrupt prompts include context, metrics, suggestions
- **Minimal overhead**: Evaluation happens during existing `get_next_prompt()` call

### Scope
- Four core rule types: `repeated_command`, `repeated_file_edit`, `phase_timeout`, `token_budget`
- YAML configuration in workflow node definitions
- Integration with Phase 1 metrics (`UnifiedMetrics`)
- Interrupt prompt generation with diagnostic context

### Integration context
- Plugs into existing `engine::get_next_prompt()` (src/engine/mod.rs)
- Consumes metrics from Phase 1 parsers (hooks.jsonl, states.jsonl, transcripts)
- Modifies prompt output without changing state machine

---

## Data Model

### Rule Definition (YAML)

```yaml
nodes:
  code:
    prompt: |
      You are in the CODE phase...
    rules:
      # Detect cargo build loops (Rust-specific)
      - repeated_command:
          pattern: "cargo (build|test)"
          threshold: 5
          window: 120

      # Detect source file churn
      - repeated_file_edit:
          path_pattern: "src/.*\\.rs"
          threshold: 8
          window: 180

      # Phase timeout (any language)
      - phase_timeout:
          max_duration: 600

      # Token budget (any language)
      - token_budget:
          max_tokens: 5000
    transitions:
      - when: "code_complete"
        to: review
```

**Field Definitions**:

- `repeated_command`:
  - `pattern` (optional): Regex matching bash commands. `None` = match all commands
  - `threshold`: Count of repetitions required to trigger
  - `window`: Time window in seconds to evaluate repetitions

- `repeated_file_edit`:
  - `path_pattern` (optional): Regex matching file paths. `None` = match all files
  - `threshold`: Count of edits required to trigger
  - `window`: Time window in seconds to evaluate edits

- `phase_timeout`:
  - `max_duration`: Maximum seconds allowed in current phase

- `token_budget`:
  - `max_tokens`: Maximum combined input+output tokens for current phase

### Rule Evaluation Context (Rust)

```rust
pub struct RuleEvaluationContext {
    pub current_phase: String,
    pub phase_start_time: DateTime<Utc>,
    pub metrics: UnifiedMetrics,
    pub workflow_mode: WorkflowMode,  // discovery | execution
}

pub enum RuleType {
    RepeatedCommand {
        pattern: Option<String>,
        threshold: usize,
        window_secs: u64,
    },
    RepeatedFileEdit {
        path_pattern: Option<String>,
        threshold: usize,
        window_secs: u64,
    },
    PhaseTimeout {
        max_duration_secs: u64,
    },
    TokenBudget {
        max_tokens: u64,
    },
}

pub struct RuleViolation {
    pub rule_type: String,
    pub diagnostic: String,
    pub suggestion: String,
    pub metrics_snapshot: serde_json::Value,
}
```

### Interrupt Prompt Format

When a rule triggers, the interrupt **replaces** the normal prompt with a decision checkpoint:

```
🚨 WORKFLOW INTERRUPT: Repeated Command Detected

Diagnostic: cargo build executed 5 times in 2 minutes
Pattern: cargo (build|test)
Recent executions:
  - 04:26:15: cargo build
  - 04:26:42: cargo build
  - 04:27:01: cargo build
  - 04:27:18: cargo build
  - 04:27:45: cargo build

Suggestion: You're stuck in a build loop. Review the error message carefully.
Consider using TDD: write a failing test first, then fix the specific issue.

---

REFLECT AND DECIDE:

Can you resolve this issue autonomously, or do you need human assistance?

If you can fix this yourself:
  - Explain your corrective approach briefly
  - Run: hegel continue
  - This will return the normal CODE phase prompt (bypassing rules this once)

If you need human help:
  - Explain what you've tried and why you're stuck
  - Wait for human guidance before proceeding
```

---

## Core Operations

### Operation 1: Load Rules from Workflow YAML

**Syntax**: Internal operation during workflow load
```rust
let workflow = load_workflow("workflows/discovery.yaml")?;
let node = workflow.get_node("code")?;
let rules = node.rules; // Vec<RuleType>
```

**Behavior**:
- Parse YAML rules section for current node
- Deserialize into `Vec<RuleType>` enum variants
- Validate regex patterns (compile to ensure valid)
- Store rules in `WorkflowNode` struct

**Validation**:
- Regex patterns must compile (`regex::Regex::new()`)
- Thresholds must be > 0
- Time windows must be > 0
- Unknown rule types error at load time
- Missing required fields error at load time

**Errors**:
```rust
Error::InvalidRulePattern { rule_id, pattern, error }
Error::InvalidRuleThreshold { rule_id, value }
Error::UnknownRuleType { rule_id, type_name }
```

### Operation 2: Continue After Interrupt (New Command)

**Syntax**: `hegel continue`

**Purpose**: Return to normal phase prompt after acknowledging a rule interrupt

**Parameters**: None

**Behavior**:
- Load current workflow state
- Get current node prompt
- Render template with guides
- **Skip rule evaluation** (one-time bypass)
- Display normal phase prompt

**Use Case**:
After a rule interrupt, the LLM reflects and decides to fix the issue autonomously. Running `hegel continue` returns them to the normal phase prompt without re-triggering rules.

**Validation**:
- Workflow must be loaded (error if no active workflow)
- Current node must exist in workflow definition

**Errors**:
```rust
Error::NoWorkflowLoaded
Error::NodeNotFound { node_name }
```

**Example Flow**:
```
1. LLM calls: hegel next '{"spec_complete": true}'
2. Rule triggers: "Repeated Command Detected"
3. LLM receives interrupt prompt with decision checkpoint
4. LLM reflects: "I see the issue, I'll fix it"
5. LLM calls: hegel continue
6. LLM receives normal CODE phase prompt
7. LLM implements fix
8. On next hegel next, rules evaluate again (with updated metrics)
```

---

### Operation 3: Evaluate Rules Against Metrics

**Syntax**: Internal operation during `get_next_prompt()`
```rust
fn evaluate_rules(
    rules: &[RuleType],
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>>
```

**Parameters**:
- `rules`: Rules defined for current workflow node
- `context`: Current phase, metrics, timestamps

**Examples**:

*Simple - Token Budget*:
```rust
// Rule: token_budget: { max_tokens: 5000 }
// Context: phase tokens = 6200
// Result: Some(RuleViolation {
//   diagnostic: "Token budget exceeded: 6,200 / 5,000",
//   suggestion: "Consider simplifying scope or splitting work"
// })
```

*Complex - Repeated Command with Regex*:
```rust
// Rule: repeated_command: { pattern: "cargo (build|test)", threshold: 5, window: 120 }
// Context: bash_commands in last 120s = ["cargo build" x5, "git status" x3]
// Result: Some(RuleViolation {
//   diagnostic: "cargo build executed 5 times in 2 minutes",
//   suggestion: "Review error, consider TDD"
// })
```

**Behavior**:
- Iterate through rules in order
- For each rule:
  - Filter metrics by time window (if applicable)
  - Filter by regex pattern (if applicable)
  - Check threshold condition
  - Return first violation found (short-circuit)
- Return `None` if no violations

**Validation**:
- Time windows computed from current timestamp vs metric timestamps
- Regex matching uses compiled patterns from load phase
- Metric counts must be accurate (use Phase 1 correlation logic)

**Errors**:
```rust
Error::RegexMatchFailed { pattern, error }
Error::MetricAccessFailed { metric_name, error }
```

### Operation 3: Generate Interrupt Prompt

**Syntax**: Internal operation after rule violation
```rust
fn generate_interrupt(
    violation: &RuleViolation,
    normal_prompt: &str,
) -> String
```

**Parameters**:
- `violation`: Triggered rule with diagnostic/suggestion
- `normal_prompt`: Original phase prompt from workflow YAML

**Examples**:

*File Edit Interrupt*:
```
🚨 WORKFLOW INTERRUPT: Repeated File Edit Detected

Diagnostic: src/main.rs edited 8 times in 3 minutes
Pattern: src/.*\.rs
Recent edits:
  - 04:20:12: Edit (src/main.rs)
  - 04:20:45: Edit (src/main.rs)
  - 04:21:18: Edit (src/main.rs)
  - 04:21:52: Edit (src/main.rs)
  - 04:22:15: Edit (src/main.rs)
  - 04:22:38: Edit (src/main.rs)
  - 04:22:59: Edit (src/main.rs)
  - 04:23:14: Edit (src/main.rs)

Suggestion: You're thrashing the same file. Step back and write a failing test
that captures the desired behavior, then implement the fix.

---

REFLECT AND DECIDE:

Can you resolve this issue autonomously, or do you need human assistance?

If you can fix this yourself:
  - Explain your corrective approach briefly
  - Run: hegel continue
  - This will return the normal CODE phase prompt (bypassing rules this once)

If you need human help:
  - Explain what you've tried and why you're stuck
  - Wait for human guidance before proceeding
```

*Timeout Interrupt*:
```
🚨 WORKFLOW INTERRUPT: Phase Timeout Exceeded

Diagnostic: CODE phase running for 11 minutes (limit: 10 minutes)
Phase start: 04:15:23
Current time: 04:26:47
Duration: 664 seconds

Suggestion: This phase is taking too long. Consider:
- Breaking the task into smaller steps
- Transitioning to LEARNINGS to document blockers
- Resetting the workflow with simplified scope

---

REFLECT AND DECIDE:

Can you resolve this issue autonomously, or do you need human assistance?

If you can fix this yourself:
  - Explain your corrective approach briefly
  - Run: hegel continue
  - This will return the normal CODE phase prompt (bypassing rules this once)

If you need human help:
  - Explain what you've tried and why you're stuck
  - Wait for human guidance before proceeding
```

**Behavior**:
- Construct interrupt header with emoji (🚨) and rule type
- Format diagnostic with specific metrics and context
- Include recent event details (timestamps, values)
- Add actionable suggestion based on rule type
- Add decision checkpoint: "REFLECT AND DECIDE"
- Provide guidance for autonomous fix vs human escalation
- Return interrupt prompt (normal prompt is NOT appended)

**Validation**:
- Timestamps formatted as HH:MM:SS
- Counts formatted with commas (e.g., "6,200")
- Event lists limited to 5 most recent (avoid prompt bloat)

---

## Test Scenarios

### Simple Test Cases

**Test 1: Token budget triggers**
- Rule: `token_budget: { max_tokens: 1000 }`
- Metrics: Phase tokens = 1500 (800 input + 700 output)
- Expected: Violation with diagnostic "Token budget exceeded: 1,500 / 1,000"

**Test 2: Repeated command (no pattern)**
- Rule: `repeated_command: { threshold: 3, window: 60 }`
- Metrics: Last 60s = ["ls" x4, "pwd" x1]
- Expected: Violation for "ls" (4 > 3)

**Test 3: Phase timeout**
- Rule: `phase_timeout: { max_duration: 300 }`
- Context: Phase running for 400 seconds
- Expected: Violation with diagnostic "Phase running for 6m 40s (limit: 5m)"

### Complex Test Cases

**Test 4: Repeated command with regex pattern**
- Rule: `repeated_command: { pattern: "cargo (build|test)", threshold: 5, window: 120 }`
- Metrics: Last 120s = ["cargo build" x3, "cargo test" x2, "cargo fmt" x1, "git status" x5]
- Expected: Violation (cargo build + cargo test = 5 matches)

**Test 5: File edit with path pattern**
- Rule: `repeated_file_edit: { path_pattern: "src/.*\\.rs", threshold: 6, window: 180 }`
- Metrics: Last 180s = edits to ["src/main.rs" x4, "src/lib.rs" x3, "README.md" x2]
- Expected: Violation (7 Rust files edited, threshold 6)

**Test 6: Multiple rules, first violation wins**
- Rules: [token_budget (not violated), repeated_command (violated), phase_timeout (violated)]
- Expected: Return repeated_command violation only (short-circuit)

### Error Test Cases

**Test 7: Invalid regex pattern**
- Rule: `repeated_command: { pattern: "[invalid(", threshold: 3, window: 60 }`
- Expected: Error at workflow load time: `InvalidRulePattern`

**Test 8: Zero threshold**
- Rule: `token_budget: { max_tokens: 0 }`
- Expected: Error at workflow load time: `InvalidRuleThreshold`

**Test 9: Negative time window**
- Rule: `repeated_command: { threshold: 3, window: -10 }`
- Expected: YAML parse error (u64 cannot be negative)

### Integration Test Cases

**Test 10: End-to-end workflow interrupt**
- Setup: Start discovery workflow at CODE phase
- Rule: `repeated_command: { pattern: "cargo build", threshold: 3, window: 60 }`
- Actions: Execute `cargo build` 4 times via hooks
- Call: `hegel next '{"claim": true}'`
- Expected: Prompt includes interrupt header + diagnostic + normal CODE prompt

**Test 11: No rules defined (passthrough)**
- Setup: Workflow node with no `rules` field
- Call: `hegel next '{"claim": true}'`
- Expected: Normal prompt returned, no interrupts

**Test 12: Rule triggers, then resets after transition**
- Setup: Token budget violated in CODE phase
- Action: Transition to REVIEW phase
- Expected: Interrupt only appears in CODE, REVIEW starts clean

---

## Success Criteria

- [ ] `hegel continue` command implemented (bypasses rules, returns normal prompt)
- [ ] `hegel continue` requires active workflow (error if none loaded)
- [ ] YAML rules parse into Rust `RuleType` enum variants
- [ ] Regex patterns validated at workflow load time
- [ ] Repeated command detection filters by regex pattern correctly
- [ ] Repeated file edit detection filters by path pattern correctly
- [ ] Phase timeout calculates duration from state transition timestamps
- [ ] Token budget sums input + output tokens from Phase 1 metrics
- [ ] Rule evaluation returns first violation (short-circuit)
- [ ] Interrupt prompts include diagnostic, suggestion, and recent events
- [ ] Interrupt prompts include "REFLECT AND DECIDE" checkpoint
- [ ] Interrupt prompts provide autonomous fix vs human escalation guidance
- [ ] Interrupt replaces normal prompt (normal prompt NOT included)
- [ ] Rules scoped to current workflow node only
- [ ] Invalid rules error at workflow load (fail fast)
- [ ] No rules defined = passthrough (no interrupts)
- [ ] Rule violations reset on phase transition
- [ ] All error messages are structured and actionable
- [ ] Test coverage ≥90% for rules module
