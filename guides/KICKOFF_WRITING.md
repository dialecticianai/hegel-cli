# KICKOFF_WRITING.md — Guide for Binary-Weave Kickoff Docs

This document instructs the agent how to write a **KICKOFF.md** for a new DDD feature in execution mode.
The goal is to produce a single, explicit binary-weave plan — not a flat list of features, not parallel streams.
The weave always alternates: *new primitive → integration with prior product*.

---

## When to Use This

**Mode**: Execution mode only
**Location**: `.ddd/feat/<feature_name>/KICKOFF.md`
**Context**: Production features built on existing codebase
**Not Used In**: Discovery mode — toys don't use KICKOFF (go straight to SPEC)

**Document Sequence (Execution Mode)**:
1. **KICKOFF.md** — Binary-weave plan (start here)
2. **SPEC.md** — Behavioral contract
3. **PLAN.md** — TDD implementation steps
4. **CODE** — Implementation
5. **LEARNINGS.md** — Optional architectural insights

---

## Core Shape of a Kickoff

1. **Napkin Physics**:  
   - Problem (1 sentence)  
   - Assumptions (3–5 bullets)  
   - Invariant (one crisp property that must always hold)  
   - Mechanism (≤5 bullets describing the minimal path)  

2. **Binary-Weave Plan**:
   - Always introduce **one new primitive at a time** (Component A, Component B, Component C …).
   - Always follow by **integrating it with the prior product** (A+B=C, C+D=E, …).
   - Each integration produces the **new "current product"**.
   - No step introduces more than one new primitive.
   - No integration combines more than two things.
   - Continue until the final feature emerges.  

3. **End State**:
   - Name the final feature.
   - Summarize which primitives and integrations were woven.
   - State the durable invariants.
   - Clarify that each component builds on tested, production-ready code from prior stages.  

---

## Formatting Expectations

- **Stage numbering is sequential.**  
  - *Stage 1*: Primitive A, Primitive B  
  - *Stage 2*: A + B = C  
  - *Stage 3*: Primitive D  
  - *Stage 4*: C + D = E  
  - *Stage 5*: Primitive F  
  - *Stage 6*: E + F = G  
  - …continue until final product.  

- **Each stage entry must have**:
  - **Name** (Primitive or Integration)
  - **What it does** (one sentence)
  - **Invariant** (instantaneous, non-blocking, etc.)  

- **Avoid parallel numbering.** Don’t list “Stage 2.3” or “Stage 2.4”.  
- **Avoid over-specification.** The kickoff is a weave map, not a spec.  
- **Avoid skipping.** Each stage should follow the weave pattern strictly.
- **Assume production context.** Features are built on the existing production codebase (`src/`), not isolated experiments.
- **Integration over isolation.** Unlike Discovery mode toys which isolate complexity, execution mode features integrate with existing production code.  

---

## Tone & Style

- Write plainly and compactly — scaffolding, not prose.  
- Prioritize clarity of the weave over detail of implementation.  
- Keep invariants crisp and behavioral, not vague.  
- Use ≤2 bullets per primitive/integration when possible.  

---

## One-Shot Checklist

- [ ] Napkin Physics included?
- [ ] Sequential stages?
- [ ] Exactly one new primitive per stage?
- [ ] Integration always combines current product with one new primitive?
- [ ] Final feature and invariants stated at end?
- [ ] Production context assumed (builds on existing codebase)?

If all are checked, the kickoff is valid.  

---