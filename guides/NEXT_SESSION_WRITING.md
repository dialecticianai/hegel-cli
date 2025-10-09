# NEXT_SESSION_WRITING.md — Guide for Writing NEXT_SESSION.md Files

This guide explains how to create session handoff documents for AI assistant continuity.

---

## Purpose

**NEXT_SESSION.md** is an ephemeral handoff document for AI assistants working across multiple sessions. It captures where you left off so the next session can pick up seamlessly.

**Key Properties**:
- Ephemeral (gitignored, deleted after reading)
- Session-scoped (not task-scoped)
- Forward-looking (what to do next, not what was done)

---

## When to Write

**CRITICAL: Only at END OF SESSION, not end of task**

Write NEXT_SESSION.md when:
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
1. Read NEXT_SESSION.md (if exists)
2. Delete immediately after reading: `rm NEXT_SESSION.md`
3. Use context to resume work

### At Session End
1. Write fresh NEXT_SESSION.md
2. DO NOT read old content (already deleted at start)
3. Include current status and immediate next action

---

## Structure

```markdown
# Next Session

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
- ✅ "Continue Step 3: Implement FileStorage class"
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
# Next Session

## Where We Left Off

Implemented Steps 1-2 of multi-user auth (StaticTokenVerifier + UserIdentityMiddleware). Currently on Step 3: FileStorage with per-user state isolation.

## What We Learned

- FastMCP Context API works as expected for user_id extraction
- Atomic writes pattern: temp file + os.rename (no flock needed)
- alice/bob dev tokens validated in integration tests

## What's Next

**Immediate action**: Complete Step 3.b - Implement FileStorage class with atomic writes

**Context needed**:
- Review PLAN.md Step 3 for acceptance criteria
- Check `.state/` directory structure once FileStorage is working
- Add `.state/` to `.gitignore` after validation

## Key Files

- `src/storage.py` - FileStorage implementation in progress (Step 3.b)
- `tests/test_storage.py` - Tests passing for Steps 1-2, Step 3 tests written but failing
- `PLAN.md` - Step 3 acceptance criteria
```

---

## Conclusion

NEXT_SESSION.md is a handoff tool, not permanent documentation. Write it at session end, delete it at session start. Keep it focused on immediate next action and essential context.
