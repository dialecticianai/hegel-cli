# Meta-Document: How to Write a PLAN.md

_Guide to planning development with TDD discipline._

---

**🚨 CRITICAL: PLANS ARE PROSE ONLY 🚨**

**DO NOT write code in PLAN.md files. NO code fences. NO implementation. NO test code.**

Plans describe WHAT to build and WHY, not HOW (that's for the implementation phase).

---

## When to Use This

**Modes**: Both Discovery and Execution
**Discovery Mode**: `toys/toyN_name/.ddd/PLAN.md` — time-boxed experiment plan
**Execution Mode**: `.ddd/feat/<feature_name>/PLAN.md` — production feature implementation plan
**Sequence**: Write after SPEC.md, before implementation
**Context**: Discovery plans are exploratory; Execution plans build incrementally on production codebase

---

## What a PLAN.md Actually Is

A **PLAN.md is high-level prose** describing the development approach step-by-step.

**CRITICAL**: Write in **PROSE ONLY**. NO code fences. NO implementation. NO test code.

### ❌ NEVER:
- Code fences with implementation
- Code fences with test code
- Variable names, function signatures, exact syntax
- Anything copy-pasteable

### ✅ ALWAYS:
- Plain English descriptions of what to build
- What to test and why (no test code)
- Step sequence and dependencies
- Success criteria checkboxes

---

{{templates/mirror_workflow}}

---

## Structure

### Header
- **Overview**: Goal, scope, priorities
- **Methodology**: TDD principles; what to test vs. not test

### Step Template

    ## Step N: <Feature Name>

    ### Goal
    Why this step matters

    ### Step N.a: Write Tests
    - Outline test strategy (no literal code)
    - Key cases: core, error, integration
    - Expected validation behavior

    ### Step N.b: Implement
    - Tasks: file/module creation, core ops, integration
    - Code patterns for illustration only
    - State and error handling guidance

    ### Success Criteria
    - [ ] Clear, testable checkpoints
    - [ ] Functional + quality standards met

---

## Key Practices

### TDD Discipline
- Write failing tests first
- Red → Green → Next
- Focus on interfaces and contracts
- Cover error paths explicitly
- **Commit after every numbered step** (Red → Green cycle)

### Commit Discipline
- Use conventional commit format: `type(scope): subject`
- Common types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`
- Include step number in subject: `feat(toy1): complete Step 3 - global state`
- Discovery mode: scope is toy name (e.g., `feat(toy1):`)
- Execution mode: scope is feature area (e.g., `feat(auth):`)
- Commit immediately after completing each step — do not batch

### Test Scope
- ✅ Test: core features, errors, integration points
- ❌ Skip: helpers, edge cases, perf, internals

### Implementation Description (Prose Only)

Describe what to build in plain English:
- "Parse workflow YAML and validate required fields"
- "Handle invalid input by returning validation errors"
- "Update state after successful operation"

### Tasks
Break implementation into minimal units:

    1. Create directory/files
    2. Implement core command parsing
    3. Add integration test path
    4. Error handling

### Success Criteria
Always check with concrete, objective boxes:

- [ ] Module initializes cleanly
- [ ] Operations produce expected output
- [ ] Errors raised for invalid input
- [ ] Test suite passes  

---

## Anti-Patterns

**❌ Writing code**: No code fences. Period.
**❌ Writing tests**: Describe what to test, don't write test code.
**❌ Over-detailing**: High-level only. Developer makes tactical decisions.

---

## Mode-Specific Guidance

### Discovery Mode PLANs
- Time-boxed experiments — bias toward minimal scope
- Single-file spikes ≤120 lines when feasible
- May include dead-end exploration (document in LEARNINGS)
- Toys are reference implementations, not production code

### Execution Mode PLANs
- Build incrementally on production codebase (`src/`)
- No isolated experiments — all code is production code
- Comprehensive test coverage required
- Integration points with existing features explicit

---

## Why This Works
- **Clear sequencing**: prevents scope drift
- **TDD enforcement**: quality-first mindset
- **Concrete validation**: objective step completion
- **Minimal guidance**: gives direction without over-specifying
- **Commit discipline**: maintains clean history and enables step-by-step review  

---

## Conclusion
A good PLAN.md is a **map, not the territory**. It sequences work, enforces TDD, and defines success. It avoids detail bloat while ensuring implementers know exactly **what to test, what to build, and when it’s done**.