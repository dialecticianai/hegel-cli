# HANDOFF_WRITING.md — Guide for Writing HANDOFF.md Files

This guide explains how to create session handoff documents for AI assistant continuity.

---

## Purpose

**HANDOFF.md** is an ephemeral handoff document for AI assistants working across multiple sessions. It captures where you left off so the next session can pick up seamlessly.

**Key Properties**:
- Ephemeral (gitignored, deleted after reading)
- Session-scoped (not task-scoped)
- Forward-looking (what to do next, not what was done)

---

## When to Write

**CRITICAL: Only at END OF SESSION, not end of task**

Write HANDOFF.md when:
- User signals session is ending ("done for now", "let's stop")
- Token budget running low
- Natural stopping point after significant progress

**DO NOT write when**:
- Completing a task mid-session
- Finishing a feature but continuing work
- Still actively working

---

## Session Lifecycle

### At Session Start

**CRITICAL: Follow this sequence for proper context loading**

#### Phase 1: Cognitive Tuning
1. Read `LEXICON.md` first - upgrade your cognition with philosophy, patterns, thresholds
2. Read `HANDOFF.md` (if exists) - session context and where you left off
3. Delete immediately: `rm HANDOFF.md` - force explicit handoff, prevent drift

#### Phase 2: Planning Docs
4. Read `SPEC.md` - understand the what and why
5. Read `PLAN.md` - step-by-step implementation plan
6. Read code maps in `README.md` files - architecture overview and integration points

#### Phase 3: Source Code (Complete Files)
7. **Read ENTIRE files** relevant to your next steps
   - Integration targets (files you'll modify)
   - Completed modules (understand what's available)
   - Dependencies (storage, state management, etc.)
   - Test patterns (follow established patterns)
   - **DO NOT try to save context** - read complete files, not snippets

#### Phase 4: Implementation
8. Now you have full context - begin work

**Why this order matters**:
- LEXICON first = cognitive upgrade (refactor thresholds, test patterns)
- HANDOFF second = session state (where we are, what's next)
- Planning docs = understanding goals and approach
- Complete source files = no missing context, no surprises mid-implementation

### At Session End
1. Write fresh HANDOFF.md
2. DO NOT read old content (already deleted at start)
3. Include current status and immediate next action
4. **Include startup instructions** following the 4-phase pattern above

---

## Structure

```markdown
# Handoff

## Where We Left Off

[1-2 sentences: Current state of work]

## What We Learned

[Key insights from this session - bullet points]

## What's Next

**Immediate action**: [Specific next step to take]

**Context needed**: [Files to review, decisions to recall]

## Key Files

- `path/to/file` - [Why this matters]
- `another/file` - [Current state]
```

---

## Writing Guidelines

**Be specific about next action**:
- ✅ "Continue Step 3: Implement core module"
- ❌ "Keep working on the feature"

**Include essential context**:
- What decisions were made
- What assumptions were validated/invalidated
- What blockers exist

**Reference key files**:
- Which files are in-progress
- Which files contain important context
- Which files need review next

**Be concise**:
- This is a handoff, not a novel
- Focus on actionable information
- Assume next session will read referenced files

---

## What NOT to Include

- Detailed implementation notes (those go in code comments)
- Complete history of session (focus on current state)
- Uncertainty about what to do next (be decisive)
- Generic status updates (be specific)

---

## Example

```markdown
# Handoff

## Where We Left Off

Implemented Steps 1-2 of feature implementation. Currently on Step 3: Core module with state management.

## What We Learned

- API integration works as expected
- State persistence pattern validated
- Initial tests passing, edge cases identified

## Next Session Startup

### Phase 1: Cognitive Tuning & Context Loading

```bash
# 1. Read LEXICON to upgrade your cognition
cat LEXICON.md

# 2. Read HANDOFF to refresh session context
cat HANDOFF.md

# 3. Delete HANDOFF (force explicit handoff)
rm HANDOFF.md
```

### Phase 2: Planning Docs

```bash
cat SPEC.md      # Feature specification
cat PLAN.md      # Step 3 acceptance criteria
# Read code maps in README.md files for architecture overview
```

### Phase 3: Source Code (Complete Files - Don't Save Context)

```bash
# Integration targets (will be modified):
cat path/to/target_module      # Step 3 implementation target
cat path/to/integration_point   # Integration point

# Dependencies (understand what's available):
cat path/to/storage_module      # Storage patterns
cat path/to/test_utilities      # Test patterns to follow
```

### Phase 4: Implementation

Now you have full context. Complete Step 3.

```bash
# Run tests frequently (adapt command to your project)
<your test command>

# Commit when done
git add -A && git commit -m "feat(scope): Step 3 - description"
```

## Key Files

- `path/to/module` - Implementation in progress (Step 3)
- `path/to/tests` - Tests passing for Steps 1-2, Step 3 tests written but failing
- `PLAN.md` - Step 3 acceptance criteria and next steps
```

---

## Conclusion

HANDOFF.md is a handoff tool, not permanent documentation. Write it at session end, delete it at session start. Keep it focused on immediate next action and essential context.
