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

## Phase 1: Core Infrastructure

### 1.1 Guide Templates

**Goal:** Reduce duplication in writing guides by extracting shared chunks into reusable templates.

**Problem:** Multiple guides repeat identical instructions (File Operations, Mirror workflow, etc.). Changes require updates across multiple files.

**Solution:** Template system for guide composition.

**Structure:**
```
guides/
  templates/
    file_operations.md    # Common file writing workflow
    mirror_workflow.md    # 5-step Mirror review process
    formatting.md         # Markdown formatting standards
  SPEC_WRITING.md         # Uses {{file_operations}} {{mirror_workflow}}
  PLAN_WRITING.md         # Uses {{file_operations}} {{mirror_workflow}}
  ...
```

**Template inclusion syntax:**
```markdown
## File Operations

{{templates/file_operations}}

{{templates/mirror_workflow}}
```

**Implementation:**
- Expand templates at workflow load time (in `render_template()`)
- Templates directory: `guides/templates/`
- Support nested templates (templates can include templates)
- Keep it simple: string replacement, no complex logic

**Benefit:** Update Mirror workflow once, applies to all guides automatically.

### 1.2 Parent Directory .hegel Discovery

**Goal:** `hegel` commands work from any subdirectory within a project (like `git`).

**Problem:** Currently hegel only finds `.hegel/` in current working directory. Running commands from subdirectories (e.g., `toys/toy1/`) fails.

**Solution:** Crawl up directory tree to find `.hegel/`, similar to git's `.git` discovery.

**Implementation:**
- When any command runs, start from `cwd` and walk up parent directories
- Stop when finding `.hegel/` directory or hitting filesystem root
- If found: use that directory as project root
- If not found: error with helpful message ("No .hegel found in current or parent directories")
- Cache discovered project root for session (avoid repeated filesystem walks)

**User experience improvement:**
```bash
# Before (broken)
cd toys/toy1_gpu_noise_match
hegel status  # ERROR: No workflow loaded

# After (works)
cd toys/toy1_gpu_noise_match
hegel status  # Mode: discovery, Current node: plan
```

**Benefit:** Natural workflow - developers can run hegel commands from wherever they're working, just like git.

### 1.3 Project Initialization

**Goal:** `hegel init` command to scaffold new projects with DDD structure.

**Sub-modes:**
- **Greenfield**: Starting from scratch
  - Create directory structure (`.hegel/`, guides/, workflows/)
  - Copy workflow definitions
  - Generate initial docs (README, .gitignore, etc.)
  - Initialize git repo (optional)

- **Retrofit**: Adding DDD to existing project
  - Detect existing structure
  - Propose integration points
  - Non-destructive additions
  - Generate compatibility layer if needed

**Interactive questions:**
- Project type (library, CLI, web app, etc.)
- Preferred workflow (discovery/execution/research)
- Language/framework (affects tooling guardrails)
- Git integration (yes/no)

---

## Phase 2: Safety and Orchestration

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

**Goal:** `hegel fork` command to delegate subtasks to other agent CLIs.

**Syntax:**
```bash
hegel fork --agent=codex 'Implement this specific function'
hegel fork --agent=gemini 'Research this API and summarize'
```

**Features:**
- Wrap external agent CLIs (codex, gemini, cursor cli)
- Pass subtask prompt to external agent
- Capture output/results
- Track forked work in metrics (duration, tokens if available)
- Optionally merge results back to main workflow

**Use cases:**
- Delegate specific subtasks to specialized agents
- Parallel work across multiple agents
- Agent comparison (same task, different agents)

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
