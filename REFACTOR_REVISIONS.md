# Refactor Plan Revisions: Preserving Hegel's Unique Features

## Critical Differences: Hegel vs ccusage

### What ccusage Does (Usage Analysis Tool)
- Parses log files AFTER the fact for cost/token reporting
- Read-only analysis (no workflow orchestration)
- Focuses on: daily/monthly aggregation, cost calculation, session grouping
- No real-time intervention or guardrails

### What Hegel Does (Workflow Orchestration Engine)
- **Real-time hook processing** - Events come via stdin during execution
- **Workflow state machine** - Enforces phase transitions with rules
- **Guardrails & interrupts** - Can STOP execution and prompt for intervention
- **Guide template injection** - Contextual prompts per workflow phase
- **State tracking** - Tracks workflow history, session metadata, phase metrics
- **Rule evaluation** - Repeated command detection, file edit loops, token budgets, phase timeouts

## Features We MUST Preserve

### 1. Real-Time Hook Processing ✅
**Current:** `hegel hook PostToolUse` reads stdin → appends to hooks.jsonl immediately
**ccusage:** Batch parses log files after the fact

**Preservation strategy:**
- Keep stdin → adapter → append flow in `commands/hook.rs`
- Adapters normalize in real-time, not batch
- Session metadata updates on SessionStart (unique to us)

### 2. Workflow State Machine ✅
**Current:** `src/engine/` with YAML workflows, state transitions, rule evaluation
**ccusage:** No workflow concept at all

**Preservation strategy:**
- Adapters are ONLY for event normalization
- Engine stays unchanged (it already works with generic events)
- Rules continue to evaluate against canonical events

### 3. Guardrails & Interrupts ✅
**Current:** Rules can trigger interrupts with custom prompts
**ccusage:** No interrupts, just reporting

**Preservation strategy:**
- Rules work with canonical events (not adapter-specific)
- `RuleEvaluationContext` gets events from normalized hooks.jsonl
- Interrupt generation stays in `src/rules/interrupt.rs`

### 4. Guide Template System ✅
**Current:** `{{SPEC_WRITING}}` placeholders inject guide content
**ccusage:** No templates or guides

**Preservation strategy:**
- Template rendering in `src/engine/template.rs` unchanged
- Guides remain in `guides/` directory
- Workflows still reference guides via placeholders

### 5. Workflow DAG & Cycle Detection ✅
**Current:** `src/metrics/graph.rs` builds DAG from transitions, detects cycles
**ccusage:** No graph analysis

**Preservation strategy:**
- Graph module unchanged
- Still parses `states.jsonl` for transitions
- ASCII rendering and DOT export preserved

### 6. Phase-Specific Metrics ✅
**Current:** `src/metrics/aggregation.rs` groups by workflow phase
**ccusage:** Groups by date/month, not workflow phase

**Preservation strategy:**
- Keep phase grouping logic
- Adapters don't affect this (they just normalize events)
- Unified metrics structure unchanged

## What We CAN Safely Adopt from ccusage

### ✅ Normalization Pattern
- Agent-specific parsers outputting canonical schema
- Graceful degradation with fallback flags
- Result types for explicit error handling

### ✅ Environment-Based Discovery
- Multi-path support (`CLAUDE_CONFIG_DIR` with fallbacks)
- Auto-detection via env vars
- File discovery methods per adapter

### ✅ In-Source Testing
- Tests alongside implementation code
- `fs-fixture` for realistic file system tests
- Better edge case coverage

### ✅ Shared Utilities
- File locking helpers
- JSONL parsing utilities
- Timestamp normalization

### ❌ What We DON'T Need from ccusage

- **LiteLLM pricing integration** - We don't calculate costs (yet)
- **Date grouping utilities** - We group by phase, not date
- **Cost calculation formulas** - Not our focus
- **Model breakdown tables** - Not relevant to orchestration
- **Offline pricing cache** - N/A

## Revised Architecture

```
src/
├── adapters/                    # NEW (from ccusage pattern)
│   ├── mod.rs                   # Registry + canonical schema
│   ├── claude_code.rs           # Real-time stdin normalization
│   ├── cursor.rs                # Future
│   └── codex.rs                 # Future
├── commands/
│   ├── hook.rs                  # SIMPLIFIED (stdin → adapter → append)
│   └── ...                      # Other commands unchanged
├── engine/                      # UNCHANGED
│   ├── mod.rs                   # Workflow state machine
│   └── template.rs              # Guide injection
├── rules/                       # UNCHANGED
│   ├── evaluator.rs             # Rule evaluation
│   ├── interrupt.rs             # Interrupt generation
│   └── types.rs                 # Rule definitions
├── metrics/                     # MINOR CHANGES
│   ├── hooks.rs                 # Use canonical events
│   ├── aggregation.rs           # Keep phase grouping
│   ├── graph.rs                 # Keep DAG analysis
│   └── ...                      # Rest unchanged
└── storage/                     # UNCHANGED
    └── mod.rs                   # hooks.jsonl, state.json
```

## Key Revisions to Original Plan

### Revision 1: Adapters Are Stateless Normalizers

**Original plan:** Adapters handle log discovery + parsing
**Revised:** Adapters ONLY normalize stdin events in real-time

```rust
pub trait AgentAdapter {
    fn name(&self) -> &str;
    fn detect(&self) -> bool;

    // ONLY normalize single event (real-time)
    fn normalize(&self, input: serde_json::Value) -> Result<CanonicalHookEvent>;

    // NO batch file parsing (that's not our use case)
    // fn parse_log_files() -> NOT NEEDED
}
```

**Rationale:** We process events as they arrive via hooks, not batch parsing logs.

### Revision 2: Keep Session Metadata in state.json

**Original plan:** Extract session from events
**Revised:** Keep existing `SessionMetadata` in `state.json`

```rust
// src/storage/mod.rs - UNCHANGED
pub struct SessionMetadata {
    pub session_id: String,
    pub transcript_path: String,
    pub started_at: String,
}
```

**Rationale:** We need this for rule evaluation context (which session are we in?).

### Revision 3: Canonical Schema Includes Workflow Context

**Original plan:** Minimal canonical event
**Revised:** Include fields needed for workflow rules

```rust
pub struct CanonicalHookEvent {
    // Core fields
    pub timestamp: String,
    pub session_id: String,
    pub event_type: EventType,

    // Tool execution (for repeated command detection)
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<serde_json::Value>,

    // Context (for phase-aware rules)
    pub cwd: Option<String>,
    pub transcript_path: Option<String>,

    // KEEP: We need these for our rules
    // - Repeated bash commands need tool_input.command
    // - File edit loops need tool_input.file_path
    // - Token budgets need transcript_path to parse tokens
    // - Phase timeouts need timestamp grouping by phase

    // Adapter metadata
    pub adapter: Option<String>,
    pub fallback_used: Option<bool>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

**Rationale:** Our rules depend on these fields. Don't minimize schema just to match ccusage.

### Revision 4: Rules Stay in Engine, Not Metrics

**Original plan:** Rules evaluate during metrics parsing
**Revised:** Rules evaluate during workflow transitions (existing behavior)

```rust
// src/engine/mod.rs - UNCHANGED
pub fn get_next_prompt(
    workflow: &Workflow,
    state: &WorkflowState,
    claims: &HashMap<String, bool>,
    state_dir: &Path,  // For loading hooks.jsonl
) -> Result<(String, WorkflowState)> {
    // 1. Check rules BEFORE transition
    let violations = evaluate_rules(&node.rules, state_dir)?;
    if let Some(violation) = violations {
        // Return interrupt prompt instead of next node
        return Ok((generate_interrupt_prompt(&violation), state.clone()));
    }

    // 2. Normal transition logic
    // ...
}
```

**Rationale:** Rules are part of workflow orchestration, not metrics reporting.

### Revision 5: Don't Copy Date Grouping Logic

**ccusage has:** Daily/monthly/weekly grouping utilities
**Hegel has:** Phase-based grouping (different concept)

**Don't port:** `formatDate`, `getDayNumber`, `getDateWeek` - we don't need these
**Keep:** `build_phase_metrics` which groups by workflow phase + timestamp ranges

**Rationale:** Our time grouping is phase-scoped, not calendar-scoped.

### Revision 6: Preserve TUI Dashboard

**Current:** `src/tui/` with real-time metrics display
**ccusage:** CLI tables only, no TUI

**Keep unchanged:**
- `src/tui/app.rs` - Dashboard state machine
- `src/tui/tabs/` - Tab implementations
- `hegel top` command - Live monitoring

**Minor update:**
- Use `CanonicalHookEvent` instead of `HookEvent` in tab rendering

**Rationale:** TUI is unique value-add, not in ccusage scope.

## Updated Migration Plan

### Phase 1: Extract Claude Code Adapter (No Behavior Changes)

**Files to create:**
- `src/adapters/mod.rs` - Schema + registry + trait
- `src/adapters/claude_code.rs` - Extract from `commands/hook.rs`

**Files to modify:**
- `src/commands/hook.rs` - Use adapter for normalization
- `src/metrics/hooks.rs` - Update to canonical schema

**Files UNCHANGED:**
- `src/engine/` - Workflow engine
- `src/rules/` - Rule evaluation
- `src/metrics/aggregation.rs` - Phase metrics
- `src/metrics/graph.rs` - DAG analysis
- `src/storage/mod.rs` - State persistence
- `src/tui/` - Dashboard

**Success criteria:**
- ✅ All existing tests pass
- ✅ Backward compatible with existing hooks.jsonl
- ✅ Rules still trigger correctly
- ✅ Workflows still advance with same logic
- ✅ TUI dashboard still displays events

### Phase 2: Add Result Types (Optional Enhancement)

**Only if it improves code quality without breaking workflows**

### Phase 3: Add More Adapters (Proves Abstraction Works)

**When we actually need Cursor/Codex support**

## Critical: What NOT to Change

❌ Don't remove `WorkflowState` tracking
❌ Don't replace phase metrics with date metrics
❌ Don't remove rule evaluation
❌ Don't eliminate guide template system
❌ Don't convert to batch log parsing
❌ Don't remove DAG cycle detection
❌ Don't eliminate TUI dashboard
❌ Don't drop SessionMetadata from state.json

## Summary of Revisions

| Aspect | Original Plan | Revised Plan |
|--------|---------------|--------------|
| **Adapter role** | Log discovery + parsing | Real-time stdin normalization only |
| **Canonical schema** | Minimal fields | Include fields needed for rules |
| **Session tracking** | Extract from events | Keep existing SessionMetadata |
| **Rule evaluation** | During metrics | During workflow transitions (unchanged) |
| **Time grouping** | Copy from ccusage | Keep phase-based grouping |
| **TUI dashboard** | Not mentioned | Explicitly preserved |
| **Workflow engine** | Not mentioned | Explicitly unchanged |

## Key Insight

**ccusage is a reporting tool. Hegel is an orchestration engine.**

We're adopting their **normalization pattern** (adapters → canonical schema), NOT their entire architecture. Our workflow engine, rules, and phase-based organization are core differentiators that must be preserved.

The refactor should make Hegel **agent-agnostic** without making it **less powerful** as a workflow orchestrator.
