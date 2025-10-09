# ORIENTATION_WRITING.md — Guide for Writing ORIENTATION.md Files

This guide explains how to create ORIENTATION.md working documents for toys and features.

---

## When to Use This

**Discovery Mode**: `toys/toyN_name/.ddd/ORIENTATION.md`
Written during the "re-orientation step" before starting a new toy. Validates assumptions, reviews plan, confirms readiness.

**Execution Mode**: `.ddd/feat/<feature_name>/ORIENTATION.md`
Written when starting or resuming work on a feature. Captures current context, open questions, next steps.

**Lifecycle**: **Working document only** — deleted when work is complete, never committed to version control.

---

## Purpose

An **ORIENTATION.md** is a **scratch pad** for grounding yourself in the current work:
- Where am I in the workflow?
- What assumptions need validation?
- What's the next concrete action?
- What blockers or uncertainties exist?

**Not permanent documentation** — this is a disposable thinking aid.
**Not committed** — local working file only, deleted when feature/toy is complete.

---

## When to Create

### Discovery Mode (Toys)
Create ORIENTATION.md during the **re-orientation step** before starting a new toy:
1. Previous toy is complete (implementation + LEARNINGS extracted)
2. New toy directory exists with SPEC.md and PLAN.md
3. **Re-orientation step**: Write ORIENTATION.md to validate readiness
4. If assumptions hold → proceed with toy
5. If assumptions fail → adjust PLAN.md or defer toy

### Execution Mode (Features)
Create ORIENTATION.md when:
- Starting a new feature (after writing KICKOFF, SPEC, PLAN)
- Resuming work after interruption (returning to in-progress feature)
- Hitting a blocker (document current state, open questions)

---

## Core Structure

### 1. Current Status
Where are you in the workflow?

```markdown
## Current Status

**Mode**: Discovery/Execution
**Phase**: SPEC / PLAN / Implementation / Review / Refactor
**Last action**: [What you just completed]
**Next action**: [What you're about to do]
```

### 2. Context Refresh
What's the essential context?

```markdown
## Context

**Goal**: [One sentence — what this toy/feature does]
**Key assumptions**:
- [Assumption 1 — needs validation]
- [Assumption 2 — already validated]
- [Assumption 3 — uncertain]

**Dependencies**: [Any external dependencies, prior toys, existing modules]
**Integration points**: [What this connects to]
```

### 3. Open Questions
What's uncertain or needs resolution?

```markdown
## Open Questions

- [ ] Does FastMCP support custom middleware? (Read docs)
- [ ] Can JWT tokens be validated without external service? (Experiment)
- [ ] Is file-based storage adequate for multi-user? (Check performance)
```

### 4. Validation Checklist
What needs to be confirmed before proceeding?

```markdown
## Validation Checklist

**Before starting implementation**:
- [ ] Read FastMCP middleware docs (.webcache/)
- [ ] Confirm JWT library supports our auth pattern
- [ ] Verify file locking is needed for concurrent writes
- [ ] Check if prior toy covered similar ground
```

### 5. Next Actions
What's the immediate next step?

```markdown
## Next Actions

1. **Immediate**: Read .webcache/server_middleware.md to understand middleware API
2. **Then**: Validate assumption about token extraction in Step 1.a tests
3. **Then**: Implement Step 1.b if tests clarify approach
4. **Blocked on**: Understanding FastMCP Context API
```

---

## Discovery Mode Example

```markdown
# Toy 5 Orientation

## Current Status

**Mode**: Discovery
**Phase**: Re-orientation (before starting implementation)
**Last action**: Completed Toy 4 (claim-based transitions) + extracted LEARNINGS
**Next action**: Validate assumptions in Toy 5 PLAN before proceeding

## Context

**Goal**: Add template rendering with required/optional placeholders to workflow prompts

**Key assumptions**:
- Toy 4's claim evaluation can be reused without modification
- Template syntax {{name}} for required, {{? name}} for optional is unambiguous
- PyYAML will preserve template placeholders (not interpret as syntax)

**Dependencies**: Toy 4 (state machine), PyYAML (workflow parsing)
**Integration points**: Will extend `get_next_prompt()` to render templates

## Open Questions

- [ ] Does PyYAML preserve {{ }} in strings? (Quick test needed)
- [ ] Should optional placeholders default to empty string or be omitted entirely?
- [ ] How to handle missing required placeholders — fail loudly or return error JSON?

## Validation Checklist

**Before starting implementation**:
- [ ] Test PyYAML behavior with {{ }} strings in YAML
- [ ] Review Toy 4's get_next_prompt to confirm integration point
- [ ] Confirm error handling pattern from prior toys (structured JSON errors)

## Next Actions

1. **Immediate**: Test PyYAML {{ }} preservation with quick spike
2. **Then**: If preserved → proceed with Step 1 tests
3. **Then**: If not preserved → revise PLAN.md for alternative syntax
4. **Blocked on**: PyYAML behavior validation (5 minute spike)

## Notes

- Keep template syntax minimal — resist feature creep
- Single-file implementation if possible (≤120 lines target)
- Focus on required vs optional distinction, skip advanced features
```

---

## Execution Mode Example

```markdown
# Multi-User Auth Feature Orientation

## Current Status

**Mode**: Execution
**Phase**: Implementation (Step 3 of 6 in PLAN.md)
**Last action**: Completed Step 2 — JWT middleware extraction working
**Next action**: Implement per-user state storage (FileStorage class)

## Context

**Goal**: Enable multi-user workflow isolation with JWT-based authentication

**Integration points**:
- FastMCP authentication (StaticTokenVerifier for dev, production TBD)
- UserIdentityMiddleware extracts user_id from token
- FileStorage persists per-user state to .state/{user_id}.json

**Completed**:
- Step 1: StaticTokenVerifier with alice/bob dev tokens ✅
- Step 2: UserIdentityMiddleware with Context.get_state("user_id") ✅

**In progress**:
- Step 3: FileStorage with atomic writes (current)

## Open Questions

- [ ] Do we need file locking for concurrent writes? (FastMCP request handling model unclear)
- [ ] Should .state/ be in .gitignore? (Yes, but not yet added)
- [ ] What happens if user_id is None? (Need to handle gracefully)

## Next Actions

1. **Immediate**: Write tests for FileStorage (Step 3.a)
2. **Then**: Implement FileStorage with atomic writes (Step 3.b)
3. **Then**: Integration test for concurrent alice/bob requests (Step 3.c)
4. **Check**: Add .state/ to .gitignore after FileStorage works

## Notes

- FastMCP Context API found in .webcache/server_middleware.md
- Atomic writes: use temp file + os.rename pattern
- Don't over-engineer locking until proven necessary
```

---

## Style Guide

**Be honest about uncertainty**: ORIENTATION is for working through unknowns
**Be specific about next actions**: "Read FastMCP docs" not "Learn more about FastMCP"
**Update frequently**: This is a living document during active work
**Delete when done**: No historical value — remove when toy/feature is complete

---

## Lifecycle

### Discovery Mode
1. **Create**: During re-orientation step (after previous toy complete)
2. **Update**: As assumptions are validated/invalidated during toy development
3. **Delete**: When toy is complete and LEARNINGS extracted

### Execution Mode
1. **Create**: When starting new feature or resuming interrupted work
2. **Update**: Daily or when hitting blockers/uncertainties
3. **Delete**: When feature moves to `.ddd/done/` (see Feature Completion Workflow)

---

## Quality Checklist

A good ORIENTATION.md helps you:
- [ ] Know exactly what to do next (no ambiguity)
- [ ] Identify blockers or uncertainties clearly
- [ ] Validate assumptions before investing time
- [ ] Resume work quickly after interruption
- [ ] Decide whether to proceed or adjust plan

If it doesn't serve these purposes, it's not helping.

---

## Conclusion

ORIENTATION.md is a **working scratch pad**, not permanent documentation. It exists to help you (or an AI assistant) ground yourself in the current work, validate assumptions, and clarify next steps. Write it when you need it, update it as you learn, delete it when the work is done.
