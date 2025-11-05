# Hegel CLI Roadmap

**Vision**: Make Hegel the orchestration layer for Dialectic-Driven Development, integrating seamlessly with Claude Code to enforce methodology through deterministic guardrails and workflow automation.

> **Note for AI assistants**: This roadmap is **future-only**. When a phase is completed, remove it from this document entirely. Do not mark phases as complete - delete them. The roadmap should only show what's coming next.

---

## Guiding Principles

1. **CLI-first**: Everything works in the terminal (with optional GUI extensions like mirror, see section 4.1)
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

## Phase 1.5: Incomplete Features (TODO Backlog)

These features have partial implementations marked with `#[allow(dead_code)]` + TODO comments.

### Multi-Agent Hook Routing
**Files:** `src/adapters/mod.rs`
- `AgentAdapter::name()` - Adapter name method
- `AdapterRegistry::get()` - Get adapter by name for explicit selection
**Use case:** Route hook events to different adapters based on agent type

### Phase-Specific Rules
**Files:** `src/rules/types.rs`
- `RuleEvaluationContext.current_phase` - Current workflow phase field
**Use case:** Enable rules that only trigger in specific phases


---

## Phase 2: Safety and Orchestration

### 2.0 Improve `hegel analyze` Output - Summary Default

**Goal:** Make `hegel analyze` more useful by providing summary mode by default and detail flags for deeper inspection.

**Current state:**
- ✅ Section flags implemented: `--summary`, `--workflow-transitions`, `--phase-breakdown`, `--workflow-graph`
- ❌ Default behavior shows ALL sections (3000+ lines for projects with ~250 commits)
- ❌ No `--full` flag to explicitly request all sections

**Remaining work:**
- **Flip default behavior**: Show only `--summary` section when no flags provided
- **Add `--full` flag**: Enable all sections (current default behavior)
- This makes the command scalable as project history grows

**Benefits:**
- Quick glance at project health (default)
- Progressive detail as needed (via section flags)
- Scalable to large projects
- Still preserves all detailed metrics when requested (via `--full`)

### 2.1 Mode-Specific Subagents

**Goal:** Integration with platform subagent features (Claude Code Task tool, Cursor agent spawning, etc.)

**Priority use case:** AST-grep pattern library for systematic refactoring

**Initial implementation - `ast-grep` subagent:**
- **Problem**: `ast-grep` is too niche for LLM training data, pattern syntax is domain-specific
- **Solution**: Few-shot learning subagent with pattern library
- **Structure**:
  - Store working patterns in `.hegel/astq_patterns/` (or similar)
  - Each pattern includes: tree-sitter query, description, example matches
  - Subagent loads patterns + ast-grep docs from `.webcache/` as context
  - Iterative refinement: test pattern → check results → adjust → repeat

**Example pattern library format:**
```yaml
rust:
  method_chains:
    - pattern: "$OBJ.cyan()"
      description: "Match .cyan() method calls on any expression"
      example_file: "examples/color_methods.rs"
      matches:
        - '"text".cyan()'
        - 'format!("{}", x).cyan()'
        - 'my_var.cyan()'
```

**Workflow:**
1. User: "Refactor all `.cyan()` to `Theme::highlight()`"
2. Hegel spawns ast-grep subagent with pattern library context
3. Subagent crafts/tests patterns iteratively
4. Returns working pattern + match list
5. User approves → Hegel applies rewrite

**Infrastructure compounding:** Pattern library grows with each refactoring, making future refactors faster.

**General subagent features:**
- Detect when platform supports subagents
- Provide workflow-aware context to subagents
- Guide injection already handles phase-specific context
- Track subagent spawning in metrics

**Implementation:**
- Detection: Check for agent capabilities via env vars or config
- Context: Current workflow phase + relevant guides + specialized libraries
- Metrics: Log subagent spawns to hooks.jsonl

### 2.2 External Agent Orchestration

**Status:** ✅ Core implementation complete (codex, gemini). Additional agents pending.

**Implemented:**
```bash
hegel fork                                    # List available agents with compatibility
hegel fork --agent=codex "Implement X"        # Delegate to codex
hegel fork --agent=gemini -- -o json "Query"  # Gemini with JSON output
hegel fork --agent=codex -- --full-auto "Y"   # Codex with auto-approval
```

**Core features:**
- ✅ Agent detection with version compatibility checking (Node.js, Python)
- ✅ Automatic nvm integration for correct Node.js version selection
- ✅ Blocking execution with captured output (agent-to-agent workflow)
- ✅ Passthrough arguments via `--` separator
- ✅ Modular agent implementations (codex, gemini, generic fallback)

**Future additions:**
- Add aider support (Python-based pair programming)
- Add cody support (Sourcegraph CLI)
- Add claude support (if non-interactive mode exists)
- Track execution metrics (duration, tokens if API exposes them)
- Optional result merging/formatting helpers
- Parallel agent execution (multiple agents, same task)

**Use cases:**
- Delegate specific subtasks to specialized agents
- Compare agent outputs (same prompt, different agents)
- Leverage agent-specific features (codex sandbox, gemini web search)

---

## Phase 3: Experimental Features

### 3.1 Internal/External Mode Switching

**Goal:** Explicit prompt structure separating reasoning mode from communication mode to prevent "sophistication regression."

**Problem:** LLMs pattern-match to user's communication style, which can drag down internal reasoning quality when user uses casual/vague language.

**Solution:** XML-style tags in workflow prompts:

```yaml
nodes:
  spec:
    prompt: |
      You are in the SPEC phase.

      <internal_reasoning>
      Think with maximum precision and rigor. Use dense philosophical
      language, formal logic, explicit constraints. Don't pattern-match
      to user's casual communication style.
      </internal_reasoning>

      {{SPEC_WRITING}}

      Write SPEC.md with technical rigor - this document is for AI agents
      to parse, not casual human reading.

      <external_output>
      After writing the SPEC, explain what you've specified to the user
      in language appropriate to their demonstrated expertise level.
      </external_output>
```

**Principle:** Internal artifacts (SPEC.md, PLAN.md, etc.) maintain rigor and density. External communication adapts to user's level.

**Implementation:**
- Add `<internal_reasoning>` and `<external_output>` tags to all workflow prompts
- Built into workflow definitions (users get benefit automatically)
- LLMs trained on XML/HTML recognize these semantic boundaries

---

*Thesis, antithesis, synthesis. Let's build it.*
