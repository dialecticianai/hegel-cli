# Hegel CLI Roadmap

**Vision**: Make Hegel the orchestration layer for Dialectic-Driven Development, integrating seamlessly with Claude Code to enforce methodology through deterministic guardrails and workflow automation.

> **Note for AI assistants**: This roadmap is **future-only**. When a phase is completed, remove it from this document entirely. Do not mark phases as complete - delete them. The roadmap should only show what's coming next.

---

## Phase 1: Metrics Collection & Analysis

**Goal**: Parse hook data and build metrics to feed cycle detection and budget enforcement

**Context**: Hook events are now captured in `~/.hegel/hooks.jsonl`. This phase processes that data into actionable metrics.

**Tasks**:
- [ ] Build metrics parser
  - Parse hooks.jsonl into structured data
  - Extract file modifications (which files, how many times)
  - Extract bash commands (command, frequency, recency)
  - Track time elapsed per workflow phase
  - Store metrics in state file or separate metrics.json
- [ ] Implement metrics display
  - `hegel metrics` command
  - Show current counts, time elapsed
  - Display file edit patterns
  - Show command history

**Success Criteria**:
- Parse hooks.jsonl into structured metrics
- `hegel metrics` displays file edits, commands, timing
- Metrics feed into state for future rule evaluation

---

## Phase 2: Cycle Detection & Budget Enforcement

**Goal**: Deterministic guardrails that interrupt workflow when stuck or over-budget

**Philosophy**: No LLM calls for enforcement - pure state-based rule evaluation using metrics from Phase 1.

**Tasks**:
- [ ] Implement cycle detection rules
  - Detect repeated bash command patterns (e.g., `cargo build` 5x in 2 min)
  - Detect file edit loops (same file edited 10x without progress)
  - Detect test failure loops (tests failing repeatedly)
  - Configurable thresholds per rule
- [ ] Implement budget enforcement
  - Token budget per workflow phase
  - Time budget per workflow phase
  - File modification budget (prevent thrashing)
  - Command execution budget (prevent infinite retries)
- [ ] Build interrupt system
  - Generate interrupt prompts when rules trigger
  - Inject prompt into workflow (override normal flow)
  - Include diagnostics: "You've run X 5 times, here's what's happening..."
  - Suggest corrective actions based on rule type
- [ ] Rules configuration
  - YAML-based rules definitions
  - Per-workflow rules (discovery vs execution)
  - User-customizable thresholds
  - Ability to add custom rules

**Example Rules**:
```yaml
rules:
  - id: build_loop_detection
    condition: "bash_command['cargo build'].count > 5 && time_since_first < 120"
    action: interrupt
    prompt: "You're stuck in a build loop. Review the error, consider TDD."

  - id: token_budget_exceeded
    condition: "phase_tokens > budget.spec_phase"
    action: interrupt
    prompt: "Token budget exceeded for SPEC phase. Consider simplifying scope."

  - id: same_file_thrashing
    condition: "file_edits[any].count > 10 && tests_passing == false"
    action: interrupt
    prompt: "You've edited the same file 10x without tests passing. Reset to TDD."
```

**Success Criteria**:
- Rules trigger based on metrics from Phase 1
- Interrupt prompts appear when thresholds crossed
- Configuration is transparent and modifiable
- Guardrails prevent common anti-patterns (build loops, file thrashing)

---

## Guiding Principles

1. **CLI-first**: Everything works in the terminal
2. **Local-first**: State lives in files, no cloud dependencies
3. **Transparent**: No black boxes, all rules/state inspectable
4. **Deterministic**: Guardrails based on logic, not LLM calls
5. **Composable**: Hegel orchestrates, Claude Code executes
6. **Open source**: Free forever, community-driven

---

## Success Metrics

- **Adoption**: CLI installs, active workflows started
- **Enforcement**: Interrupt triggers (are guardrails helping?)
- **Velocity**: Features shipped faster with DDD methodology
- **Quality**: Fewer bugs, better architecture (via mandatory refactoring)

---

## Non-Goals

- ❌ Become an IDE (stay CLI-focused)
- ❌ Replace Claude Code (we're the orchestration layer)
- ❌ Require external dependencies (fully local)
- ❌ Hide complexity (make state and rules legible)

---

---

*Thesis, antithesis, synthesis. Let's build it.*
