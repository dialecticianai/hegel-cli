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

### 1.1 Multi-Agent Support

**Goal:** Abstract Claude Code integration as one of many possible agent integrations.

**Approach:** Logstash-style pluggable input adapters with auto-install on first use.

**Features:**
- Auto-detect agent from environment variables (`CLAUDE_CODE_SESSION_ID`, `CURSOR_SESSION_ID`, etc.)
- Pluggable adapters: `hegel-plugin-cursor`, `hegel-plugin-codex`, `hegel-plugin-gemini`
- Auto-install on demand when agent detected
- Normalize agent-specific events to canonical schema → `.hegel/hooks.jsonl`
- Distribution: Pre-compiled binaries from GitHub releases
- Communication: JSONL stdin/stdout protocol between Hegel and plugins

**Implementation:**
- `input-claude-code` - Current implementation (baseline)
- `input-codex` - OpenAI Codex telemetry integration
- `input-gemini` - Google Gemini integration (post-MVP)
- `input-cursor` - Cursor event system integration (post-MVP)

**Plugin installation:**
```bash
# Auto-triggered on first use:
# Downloads from: https://github.com/dialecticianai/hegel-plugin-cursor/releases
# Installs to: ~/.hegel/plugins/cursor-adapter
```

### 1.2 Research Mode Workflow

**Goal:** Add Research mode as distinct workflow for external knowledge gathering phase.

**Workflow structure:**
- New `workflows/research.yaml` definition
- Phases: Research activities (study, cache, document, questions)
- Outputs: Learning documents + catalogued open questions (`.ddd/5_open_questions.md` or similar)
- Transition: Explicit claim to transition to Discovery mode when ready

**Metrics tracking:**
- Time spent in research phase
- Files added to `.webcache/`
- Learning docs created/updated
- Questions catalogued
- Less emphasis on: bash commands, code edits (not coding yet)

**New guides needed:**
- `STUDY_PLANNING.md` - How to organize research priorities
- `KNOWLEDGE_CAPTURE.md` - How to write learning docs
- `QUESTION_TRACKING.md` - How to catalogue open questions

### 1.3 Meta-Modes Formalization

**Goal:** Represent Learning/Porting/Standard progression patterns in Hegel.

**Concept:** Meta-modes are patterns of mode transitions (Research ↔ Discovery, Discovery → Execution, etc.)

**Implementation TBD:** Need to design how meta-modes surface in Hegel:
- Workflow composition (multiple .yaml files chained)?
- Higher-level workflow definitions?
- Automatic transitions between modes based on claims?
- Analytics that recognize meta-mode patterns?

### 1.4 Project Initialization

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

### 2.2 Mode-Specific Subagents

**Goal:** Integration with platform subagent features (Claude Code Task tool, Cursor agent spawning, etc.)

**Features:**
- Detect when platform supports subagents
- Provide workflow-aware context to subagents
- Guide injection already handles phase-specific context
- Track subagent spawning in metrics

**Implementation:**
- Detection: Check for agent capabilities via env vars or config
- Context: Current workflow phase + relevant guides
- Metrics: Log subagent spawns to hooks.jsonl

### 2.3 External Agent Orchestration

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
