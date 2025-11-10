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

## Phase 1: State & Metrics System Improvements

### 1.1 Invert Apply/Dry-Run Semantics (Priority 1)

**Goal:** Make dry-run the default behavior, require explicit `--apply` flag to make changes.

**Problem:** Current commands use `--dry-run` flag to preview changes, defaulting to applying them. This is dangerous for destructive operations (archive deletion, state migration, etc.). Safe-by-default is better UX.

**Affected commands:**
- `hegel analyze --fix-archives` (repairs archives, creates cowboys, removes duplicates)
- `hegel archive --migrate` (migrates old logs to archives)
- `hegel doctor` (state file health check and repair)

**Current behavior:**
```bash
hegel analyze --fix-archives              # Applies changes immediately
hegel analyze --fix-archives --dry-run    # Preview only
```

**Desired behavior:**
```bash
hegel analyze --fix-archives              # Preview only (dry-run default)
hegel analyze --fix-archives --apply      # Actually apply changes
```

**Implementation:**
- Invert flag semantics in `AnalyzeArgs`, `ArchiveArgs`, `DoctorArgs`
- Update help text to clarify default is preview-only
- Update all callsites to use `!args.apply` instead of `args.dry_run`
- Add deprecation warning if `--dry-run` flag is used (guide users to new semantics)
- Update tests to use `--apply` for actual mutations

### 1.2 Unified Health Check and Auto-Repair (Priority 2)

**Goal:** Make `hegel doctor` the one-stop command for detecting and fixing all state/metrics issues.

**Current state:**
- `hegel doctor` - State file health check + rescue corrupted state
- `hegel analyze --fix-archives` - Archive repairs (git backfill, cowboy creation, duplicate removal)
- `hegel archive --migrate` - Legacy log migration

**Problem:** Functionality is fragmented across commands. Users don't know which command to run for which issue.

**Desired behavior:**
```bash
hegel doctor                    # Detect all issues (state, archives, gaps)
hegel doctor --apply            # Fix all detected issues automatically
hegel doctor --verbose          # Show detailed issue breakdown
```

**What `hegel doctor` should detect and fix:**
1. **State file issues** (existing):
   - Corrupted state.json → rescue from backup
   - Missing required fields → apply migrations
   - Invalid workflow state → reset to clean state

2. **Archive issues** (migrate from `analyze --fix-archives`):
   - Missing git commits → backfill from git history
   - Incomplete workflows → add aborted terminal nodes
   - Duplicate cowboys → remove redundant archives
   - Workflow gaps with activity → create synthetic cowboys
   - Stale cumulative totals → rebuild from archives

3. **Log migration** (migrate from `archive --migrate`):
   - Detect old multi-workflow logs → migrate to archives
   - Orphaned JSONL files → clean up or archive

**Implementation approach:**
1. **Extract reusable modules:**
   - Move `src/analyze/repair.rs` logic to `src/doctor/repairs/archive_repair.rs`
   - Move `src/commands/archive.rs` logic to `src/doctor/repairs/log_migration.rs`
   - Create trait: `HealthCheck { detect(), fix(), name() }`

2. **Unified detection system:**
   ```rust
   pub trait HealthCheck {
       fn name(&self) -> &str;
       fn detect(&self, storage: &FileStorage) -> Result<Vec<Issue>>;
       fn fix(&self, storage: &FileStorage, issues: &[Issue]) -> Result<usize>;
   }
   ```

3. **Doctor orchestration:**
   - Run all health checks in sequence
   - Collect and categorize issues
   - Display unified report
   - Apply fixes when `--apply` flag provided

4. **Preserve existing commands:**
   - `hegel analyze --fix-archives` → delegate to doctor repair modules
   - `hegel archive --migrate` → delegate to doctor log migration
   - Maintain backward compatibility, but suggest `hegel doctor` in output

**Files to refactor:**
- `src/analyze/repair.rs` → `src/doctor/repairs/archive_repair.rs`
- `src/analyze/gap_detection.rs` → `src/doctor/repairs/gap_detection.rs`
- `src/analyze/cleanup/` → `src/doctor/repairs/cleanup/` (or keep as-is, import from doctor)
- `src/commands/archive.rs` → delegate to `src/doctor/repairs/log_migration.rs`
- `src/commands/doctor/mod.rs` → orchestrate all health checks

---

## Phase 2: Incomplete Features (TODO Backlog)

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

### 2.3 Subworkflow Execution

**Goal:** Allow running partial workflow graphs with explicit phase sequences.

**Note:** This extends existing `hegel start <workflow> <node>` behavior (which lets you control the start point) to also control the end point and composition.

**Syntax:**
```bash
hegel start execution spec->plan              # Run spec, then plan, then done
hegel start execution code->review            # Run code, then review, then done
hegel start discovery exploration->synthesis  # Custom sequences
```

**Behavior:**
- Parse `node1->node2->...` syntax to build subgraph
- Validate that transitions exist between specified nodes
- Automatically append `done` node as final step
- Execute only the specified phases in sequence
- Useful for iterating on specific workflow segments

**Implementation:**
- Extend `hegel start` command parser to detect `->` sequences
- Validate phase connectivity using workflow YAML graph
- Build temporary workflow state with custom node sequence
- Track subworkflow execution in metrics (differentiate from full workflows)

**Use cases:**
- Test workflow changes iteratively
- Re-run specific segments without full workflow execution
- Quick prototyping of new workflow patterns

### 2.4 Project Bootstrapping

**`hegel seed` command:**
- **Goal:** Instant project scaffolding with opinionated presets
- **Type:** Command (not workflow) - zero LLM involvement, pure infrastructure
- **Use case:** Fast bootstrapping when you want proven defaults, not customization
- **Syntax:**
  ```bash
  hegel seed --preset rust-cli my-tool
  hegel seed --preset python-mcp my-service
  hegel seed --preset static-site my-blog
  ```
- **Preset categories:**
  - `rust-cli` - From hegel-cli patterns: pre-commit hooks (rustfmt, coverage, LOC), build scripts with version bumping, test stability checks, metrics generation
  - `rust-gui-egui` - From hegel-mirror patterns: Same as CLI plus egui/eframe setup, GUI-specific dependencies
  - `python-mcp` - From ddd-mcp patterns: pytest with coverage, thread-safety tests, velocity analysis, codemap sync checker
  - `static-site-quartz` - From dialectician.ai patterns: Quartz build pipeline, content aggregation, rsync deployment
  - `docs-mdbook` - From ddd-book patterns: mdBook setup, pre-commit build/linkcheck, stats generation
- **Implementation:**
  - Presets defined as templates in `presets/` directory
  - Template variables: project name, author, license, language version
  - File generation: scripts, configs, git hooks, README boilerplate
  - Foundation: BUILD_TOOLS.md analysis distilled into reusable templates
- **Philosophy:** Opinionated defaults based on battle-tested patterns. "I've paid the cost of learning why this matters - trust me or use scaffold workflow instead."

**`scaffold` workflow:**
- **Goal:** LLM-guided project scaffolding with reference analysis and synthesis
- **Type:** Workflow (with LLM) - slow, flexible, interactive
- **Use case:** When you need customization, synthesis from multiple references, or architectural guidance
- **When to use:**
  - Right after `hegel init` (common case)
  - Adding new component to existing project
  - Creating parallel service in monorepo
  - Bootstrapping toy model during discovery
  - When presets don't fit your needs
- **Flow:**
  1. User provides reference project path(s), URLs, or describes requirements
  2. Agent analyzes reference structure (directory layout, key files, build patterns, test strategies)
  3. Agent asks clarifying questions about tech stack, constraints, preferences
  4. Generate customized scaffold adapted to context
  5. Create README, configuration files, initial code structure
  6. Document architectural decisions and extracted patterns
  7. Optional: Save as custom preset for future reuse
- **Key features:**
  - Multi-reference synthesis (combine patterns from multiple sources)
  - Interactive decision-making (agent asks about trade-offs)
  - Template variable substitution (project name, language, etc.)
  - Dependency version resolution (update to latest compatible)
  - Architecture documentation generation
  - Custom preset creation from scaffold output
- **Contrast with seed:**
  - seed = fast, opinionated, zero LLM ("cargo new" style)
  - scaffold = slow, flexible, guided synthesis ("design with me" style)

**debug workflow:**
- **Goal:** Systematic TDD debugging with test-first methodology
- **Use case:** Fix failing tests or bugs through disciplined process
- **Flow:**
  1. Identify failing test or bug report
  2. Write minimal reproduction test (if not exists)
  3. Analyze failure root cause
  4. Propose fix with explanation
  5. Verify fix passes all tests
  6. Document bug pattern and prevention strategy
- **Key features:**
  - Red-green-refactor cycle enforcement
  - Root cause analysis prompts
  - Regression test validation
  - Bug pattern library accumulation

### 2.5 Custom Claude Commands Integration

**Goal:** Integrate Hegel commands directly into Claude Code via custom slash command extension.

**Command:** `/hegel` - Intelligent proxy for `hegel` CLI commands

**Implementation plan:**
1. Implement directly in hegel-cli (this repo) using `.claude/commands/` integration
2. Parse slash command arguments and route to appropriate `hegel` subcommands
3. Formalize into `../claude-plugins` for broader distribution and reuse

**Features:**
- Slash command parsing and routing to `hegel` subcommands
- Context-aware command suggestions based on current workflow state
- Workflow state awareness in Claude Code sessions
- Seamless integration with Claude Code's custom command system

**Use cases:**
- Run `hegel` commands without leaving Claude Code conversation
- Provide workflow context to Claude during execution phases
- Enable Claude to understand and respond to current workflow state
- Simplify workflow advancement (`/hegel next` instead of typing full command)

**Technical approach:**
- `.claude/commands/hegel.md` - Command definition with argument parsing
- Dynamic command generation based on available `hegel` subcommands
- State injection into Claude context (current phase, workflow, history)
- Integration with existing hook system for telemetry

### 2.6 Granular Activity Filtering for Cowboy Detection

**Goal:** Improve cowboy workflow detection to exclude workflow control commands and capture only real development activity.

**Current behavior (temporary):**
- Cowboy workflows created only when git commits OR uncommitted changes exist
- All bash commands/file modifications without git evidence are ignored
- Prevents false positives from `hegel next`/`hegel start` commands

**Desired behavior:**
- Capture bash commands and file modifications that represent actual work
- Exclude workflow control commands (`hegel next`, `hegel start`, `hegel prev`, etc.)
- Exclude meta commands (`git status`, `ls`, `cat`, etc.)
- Include productive commands (`cargo build`, `npm install`, editing code files, etc.)

**Implementation approach:**
1. Create allowlist/blocklist for command patterns
   - Block: `hegel *`, `git status`, `git log`, `git diff`, `ls`, `cat`, `grep`, etc.
   - Allow: Build commands, test commands, package management, code generation
2. Filter bash commands and file modifications by these patterns
3. Create cowboy workflows when:
   - Git commits exist in gap, OR
   - Uncommitted changes exist, OR
   - Non-excluded bash/file activity exists
4. Re-enable currently ignored tests: `test_detect_cowboy_with_bash_activity`, `test_cowboy_with_file_modifications`

**Why it matters:**
- Current approach is too conservative - misses legitimate cowboy work sessions
- LLM agents do significant work via bash commands and file edits before committing
- Need to capture this activity without false positives from workflow navigation

**Complexity:**
- Requires heuristics for "productive" vs "meta" commands
- Different projects may have different productive command patterns
- Configuration may be needed for custom command classification

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
