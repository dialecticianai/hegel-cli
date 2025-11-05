# Meta-Document: How to Write an Effective SPEC.md

_Guide to writing specifications in the Dialectic-Driven Development paradigm._

---

## When to Use This

**Modes**: Both Discovery and Execution
**Discovery Mode**: `toys/toyN_name/.ddd/SPEC.md` — defines behavioral contract for isolated toy experiment
**Execution Mode**: `.ddd/feat/<feature_name>/SPEC.md` — defines behavioral contract for production feature
**Sequence**: Write after KICKOFF (execution mode) or directly after toy directory creation (discovery mode)

**CRITICAL**: Do discovery/research BEFORE writing. Ground specs in actual codebase, not assumptions.

---

## Purpose

A **SPEC.md is a contract spike**: it defines what the system must accept, produce, and guarantee.  
It exists to make implementation falsifiable — to ensure tests and validation have clear ground truth.

---

## What a SPEC.md Is / Is Not

### ❌ Not
- Implementation details (classes, functions, algorithms)
- Code snippets or function signatures
- Internal design notes (unless exposed in the contract)
- Tutorials, manuals, or user guides
- Vague aspirations ("the system should work well")

### ✅ Is
- **High-level prose descriptions** of behavior
- Precise input/output formats
- Defined state transitions or invariants
- Operation semantics (commands, APIs, behaviors)
- Error and validation rules
- Concrete test scenarios and acceptance criteria

**Format**: Write in clear, structured prose. Use examples to illustrate behavior, not to show implementation.

---

{{templates/mirror_workflow}}

---

## Before Writing: Discovery Phase

**Read the codebase FIRST.** Do not invent data structures or APIs without checking reality.

### Required Research Steps

1. **Existing Code**: Read relevant source files for:
   - Current types and data structures
   - Existing APIs and contracts
   - Module organization and boundaries
   - Patterns to match or extend

2. **Testing Infrastructure**: Check TESTING.md and test patterns:
   - What test utilities exist?
   - How are similar features tested?
   - Can this be tested autonomously?

3. **Dependencies**: Understand what libraries/modules already provide:
   - Don't reinvent functionality that exists
   - Trust existing abstractions
   - Verify data availability before assuming

4. **Architectural Boundaries**: Understand module responsibilities:
   - Where do types belong?
   - What's the separation of concerns?
   - Are there migration constraints (breaking vs. backward compatible)?

### Document Your Findings

Before writing the spec, document:
- What exists that can be reused
- What needs modification
- What's genuinely missing
- Any uncertainties to clarify with user

---

## Core Structure

### 1. Header

**Discovery Mode**:
```markdown
# Toy Model N: [System Name] Specification
One-line purpose statement
```

**Execution Mode**:
```markdown
# [Feature Name] Specification
One-line purpose statement
```

### 2. Overview
- **What it does:** core purpose in 2–3 sentences
- **Key principles:** 3–5 bullets on design philosophy
- **Scope:** Discovery mode — isolates 1–2 complexity axes; Execution mode — production feature scope
- **Integration context:** Discovery — note inputs/outputs to other toys; Execution — note integration points in existing codebase

### 3. Data Model
Define external data formats with **realistic examples**:
- **Reference actual file paths** for existing types (e.g., `src/metrics/mod.rs::UnifiedMetrics`)
- All required fields shown
- Nested structures expanded
- Field purposes explained
- **Specify NEW vs. MODIFIED vs. REMOVED** for each type
- Review for redundancy: clean separation of concerns
- Mark uncertainties as **(TBD: verify)** rather than speculating
- JSON schemas when clarity demands

### 4. Core Operations
Document commands or APIs with a consistent pattern:
- **Syntax** (formal usage)
- **Parameters** (required/optional, ranges, defaults)
- **Examples** (simple + complex)
- **Behavior** (state changes, outputs, side effects)
- **Validation** (rules, errors, edge cases)

### 5. Test Scenarios
3 categories:
1. **Simple** — minimal case
2. **Complex** — realistic usage
3. **Error** — essential error cases only (not exhaustive edge cases)

**Execution mode**: Focus on scenarios that drive TDD and validate core behavior. Skip hypothetical edge cases, security hardening scenarios, and defensive coverage.

Optionally, **Integration** — only if the feature touches another system.

### 6. Success Criteria

**Agent-Verifiable Only**: Criteria must be checkable without human judgment.

Good (agent-verifiable):
- Tests pass: `cargo test`
- Build succeeds: `cargo build`
- Schemas match expected structure
- Operation X preserves invariant Y
- Error messages are structured JSON
- Round-trip import/export retains labels

Bad (requires human judgment):
- ❌ "No UI flicker"
- ❌ "Smooth navigation"
- ❌ "Responsive UI"
- ❌ "Intuitive layout"

**Optional Human Testing**: Separate section for subjective QA concerns that require manual verification.

**Note:** Use plain bullets, not checkboxes `[ ]`. SPECs are frozen artifacts, not tracking documents.

---

## Quality Heuristics

High-quality SPECs are:
- **Precise** — eliminate ambiguity
- **Minimal** — Discovery mode: isolate 1–2 complexity axes; Execution mode: single focused feature
- **Falsifiable** — every statement testable
- **Contextual** — note integration points when they matter
- **Mode-appropriate** — Discovery specs justify toy isolation; Execution specs are lean and agile
- **Prose-focused** — written in clear, descriptive language rather than code
- **Agile in Execution** — focus on working functionality, not defensive edge cases

Low-quality SPECs are:
- Vague ("system processes data")
- Over-prescriptive (dictating implementation)
- Bloated with internal details or code snippets
- Missing testable criteria
- Implementation-heavy (showing how rather than what)
- **Over-engineered (Execution mode)** — security hardening, exhaustive edge cases, premature optimization
- **Ungrounded** — invented data structures without checking existing code
- **Architecturally confused** — types in wrong modules, misunderstood boundaries
- **Reinventing wheels** — duplicating functionality that libraries already provide
- **Subjective success criteria** — "smooth", "responsive", "no flicker" (not agent-verifiable)
- **Verbose examples that age poorly** — ASCII art UI mockups, box-drawing layouts

---

---

## After Writing: Self-Consistency Check

Before finalizing, verify:

1. **Grounding**: Did you read actual code files? Are types/APIs real or invented?
2. **Cross-reference**: Does it align with TESTING.md, CLAUDE.md, existing code?
3. **Architectural clarity**: Are types in the right modules? Boundaries clear?
4. **Lean check**: Can any sections be deleted without losing essential info?
5. **Scope discipline**: Edge cases, performance tuning deferred to "Out of Scope"?
6. **Verifiable criteria**: Success criteria agent-checkable (tests, builds, schemas)?
7. **No redundancy**: Type hierarchies clean, no nested duplication?

---

## Conclusion

A SPEC.md is not a design novel.
It is a **minimal, precise contract** grounded in actual codebase reality that locks in what must hold true, so tests and implementations can be judged unambiguously.

Discovery mode: If multiple axes of complexity emerge, split them into separate toy models.
Execution mode: Research first, discuss constraints, write lean specs with agent-verifiable criteria.