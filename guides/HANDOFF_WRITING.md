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
1. Read HANDOFF.md (if exists)
2. Delete immediately after reading: `rm HANDOFF.md`
3. Use context to resume work

### At Session End
1. Write fresh HANDOFF.md
2. DO NOT read old content (already deleted at start)
3. Include current status and immediate next action

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

## What's Next

**Immediate action**: Complete Step 3 - Implement core module functionality

**Context needed**:
- Review PLAN.md Step 3 for acceptance criteria
- Verify integration points after implementation
- Update documentation once feature is working

## Key Files

- `src/module` - Implementation in progress (Step 3)
- `tests/module_test` - Tests passing for Steps 1-2, Step 3 tests written but failing
- `PLAN.md` - Step 3 acceptance criteria
```

---

## Conclusion

HANDOFF.md is a handoff tool, not permanent documentation. Write it at session end, delete it at session start. Keep it focused on immediate next action and essential context.
