# src/rules/

Layer 3: Deterministic workflow enforcement. Evaluates rules to prevent phase advancement when constraints are violated, with interrupt protocol for agent feedback.

## Purpose

Rules provide deterministic guardrails for workflows. They check preconditions (file existence, token budgets, time limits) before allowing phase transitions. When rules are violated, the interrupt protocol injects explanatory prompts to guide agents toward compliance.

## Structure

```
rules/
├── mod.rs               Public exports (evaluate_rules, interrupt_if_violated)
├── types.rs             Rule definitions (require_files, max_tokens, phase_timeout, etc.)
├── evaluator.rs         Rule evaluation engine (stateless, context-based, phase_start_time support)
└── interrupt.rs         Interrupt protocol (rule violation → prompt injection)
```

## Rule Types

- `require_files` - Enforce file existence before advancing
- `max_tokens` - Token budget enforcement (per-phase limits)
- `phase_timeout` - Time-based phase limits

## How It Works

1. **Evaluation**: Before phase transition, evaluate all rules for target node
2. **Violation Handling**: If any rule fails, call interrupt protocol
3. **Prompt Injection**: Inject explanation of violation into workflow output
4. **Block Transition**: Prevent state change until rule compliance achieved
