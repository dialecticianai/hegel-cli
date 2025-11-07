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

**Scope**: Plans cover implementation only. Do not include manual testing steps or documentation updates (those happen in separate workflow phases).

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
    - Clear, testable checkpoints
    - Functional + quality standards met

    **Note:** Use plain bullets, not checkboxes. PLANs are frozen artifacts, not tracking documents.

---

## Key Practices

### TDD Discipline
- Write failing tests first (when tests drive development)
- Red → Green → Next
- Focus on interfaces and contracts
- Cover essential error paths, not hypothetical edge cases
- Tests should enable forward progress, not block it

### Commit Discipline
- Use conventional commit format: `type(scope): subject`
- Common types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`
- **Group steps into logical commit units** - avoid cluttering git history with trivial commits
- Trivial changes (adding dependencies, creating empty files, small config) should be bundled together
- Substantial implementation (100+ LOC, complete features, significant refactors) can stand alone
- **Explicitly mark commit boundaries in your plan** using "Commit Point" markers between step groups
- Discovery mode: scope is toy name (e.g., `feat(toy1):`)
- Execution mode: scope is feature area (e.g., `feat(auth):`)


### Test Scope
- ✅ Test: core features that drive development, essential errors, integration points
- ❌ Skip: helpers, hypothetical edge cases, perf optimization, internals, exhaustive coverage
- **TDD serves development, not audit requirements**

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
Always check with concrete, objective bullets:

- Module initializes cleanly
- Operations produce expected output
- Errors raised for invalid input
- Test suite passes

**Note:** Use plain bullets, not checkboxes `[ ]`. PLANs are frozen artifacts, not tracking documents.  

---

## Anti-Patterns

**❌ Writing code**: No code fences. Period.
**❌ Writing tests**: Describe what to test, don't write test code.
**❌ Over-detailing**: High-level only. Developer makes tactical decisions.
**❌ Manual testing steps**: No "human validation" or "integration testing" steps requiring manual verification.
**❌ Documentation updates**: No README, CLAUDE.md, or doc file updates. Those happen in separate workflow phases.

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
- Tests that drive development forward, not exhaustive coverage
- Integration points with existing features explicit
- **Agile approach**: working functionality over defensive programming
- Focus on core behavior, add edge cases only when essential

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