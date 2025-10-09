# CLAUDE.md

**Hegel**: CLI tool for Dialectic-Driven Development workflows. State-based orchestration, no external dependencies.

---

## Philosophy (Compressed from LEXICON.md)

**Context is king** - State determines what's possible. Files load tokens, line counts are physics.

**Refactor early, not late** - 18x token overhead is immediate cost, not future debt. Structure for reading efficiency, not writing comfort.

**Infrastructure compounds** - Each abstraction saves future tokens. Test helpers, scripts, submodules all pay dividends forever.

**Remember you're not human** - Comprehensive is just complete. No cost to thoroughness. Line count thresholds are literal constraints.

**The human always knows best** - Execute instructions, don't editorialize. Questions are literal, not criticism.

**Artifacts are disposable, clarity is durable** - Code can be rewritten, insights cannot. Documentation is the deliverable.

**Housekeeping before heroics** - Automate the pattern before repeating it. Write scripts on second occurrence, not third pain.

Full philosophy: See `LEXICON.md` when making architectural decisions.

---

## Critical Files & State

**State directory**: `.hegel/` (gitignored)
- `state.json` - Current workflow state (atomic writes via temp+rename)
- `states.jsonl` - State transition log (file-locked appends)
- `hooks.jsonl` - Claude Code hook events (file-locked appends)

**Workflows**: `workflows/*.yaml` - Phase definitions, transition logic
**Guides**: `guides/*.md` - Template content for workflow prompts
**Metrics**: `src/metrics/{hooks,transcript,states}.rs` - JSONL parsers (isolated by domain)

**Session continuity**: `HANDOFF.md` (ephemeral, gitignored - see protocol below)

---

## HANDOFF.md Protocol

**CRITICAL: Only update at END OF SESSION**

**Session start:**
1. Read `HANDOFF.md` if exists
2. **Delete immediately**: `rm HANDOFF.md`

**Session end:**
1. Write fresh `HANDOFF.md` (don't read old - already deleted)
2. Include: status, learnings, next action, key files
3. **NO FURTHER CODE WORK** after writing HANDOFF
4. Only housekeeping: doc updates, commits
5. **NEVER commit HANDOFF.md** (gitignored)

---

## Development Procedures

**Commit format**: `type(scope): subject` with footer:
```
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Doc updates BEFORE code commits**:
- `README.md` - User-facing changes
- `ROADMAP.md` - Delete completed phases (future-only policy)
- `LEXICON.md` - New architectural principles
- Coverage/LOC reports auto-update via pre-commit hook

**Testing**: TDD discipline, ≥80% coverage target. `cargo test` runs all.

**Module organization**: Split files >200 impl lines into submodules. Token efficiency over monolithic convenience.

---

## Claude Code Hooks Integration

Hegel captures Claude activity via hooks: `.claude/settings.json` routes events to `hegel hook <event>` (reads stdin JSON, appends to `.hegel/hooks.jsonl`).

Currently passive logging. Future: Cycle detection, budget enforcement from metrics.

---

## Workflow Engine

**Commands**: `start <workflow>` → `next '{"claim":true}'` → `status` → `reset`

**State machine**: Load YAML → evaluate claims → transition nodes → save state

**Templates**: `{{GUIDE_NAME}}` (required) and `{{?optional}}` in prompts, resolved from `guides/`

**Atomic writes**: All `.hegel/*.json` writes use temp file + rename to prevent corruption.

---

## Next Steps Protocol

After completing tasks, propose next action with alternatives:
- ✅ "Should I implement X, or refactor Y first?"
- ❌ "Done. What's next?" (forces user to decide scope)

Wait for explicit approval before proceeding.

---

**Philosophy**: Thesis. Antithesis. Synthesis. State-based workflow orchestration with no external dependencies.
