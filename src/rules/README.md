# src/rules/

Layer 3: Deterministic workflow enforcement. Evaluates rules to prevent phase advancement when constraints are violated, with interrupt protocol for agent feedback.

## Purpose

Rules provide deterministic guardrails for workflows. They check preconditions (file existence, token budgets, time limits) before allowing phase transitions. When rules are violated, the interrupt protocol injects explanatory prompts to guide agents toward compliance.

## Structure

```
rules/
├── mod.rs               Public exports (evaluate_rules, interrupt_if_violated)
├── types.rs             Rule definitions (RuleConfig enum, RuleEvaluationContext)
├── evaluator.rs         Rule evaluation engine (commit checking, token budgets, timeouts)
├── interrupt.rs         Interrupt protocol (rule violation → prompt injection)
│
└── tests/               Rule evaluation tests
    └── evaluator.rs     Unit tests for all rule evaluators
```

## Rule Types

- `require_commits` - Enforce git commits before advancing (with lookback window)
- `repeated_command` - Detect repeated bash commands (anti-loop protection)
- `repeated_file_edit` - Detect repeated file edits (anti-thrashing protection)
- `phase_timeout` - Time-based phase limits
- `token_budget` - Token budget enforcement (per-phase limits)

## How It Works

1. **Evaluation**: Before phase transition, evaluate all rules for target node
2. **Violation Handling**: If any rule fails, call interrupt protocol
3. **Prompt Injection**: Inject explanation of violation into workflow output
4. **Block Transition**: Prevent state change until rule compliance achieved

## Force Bypass

Rules can be bypassed via CLI:
- `hegel next --force` - Skip all rules
- `hegel next --force require_commits` - Skip only commit checks

Config toggles:
- `commit_guard: false` - Globally disable require_commits rules
- `use_git: false/true` - Override git repository detection
