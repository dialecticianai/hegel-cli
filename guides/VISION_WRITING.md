# Meta-Document: How to Write an Effective VISION.md

_Guide to articulating product goals in the Dialectic-Driven Development paradigm._

---

## When to Use This

**Phase**: Project initialization (greenfield or retrofit)
**Location**: Root of project directory (`VISION.md`)
**Sequence**: Write after CLAUDE.md customization, before ARCHITECTURE.md
**Purpose**: Define the "why" - what problem are we solving and what does success look like?

---

## Purpose

A **VISION.md is a product compass**: it defines the problem space, target users, and success criteria.
It exists to keep development focused on outcomes, not just outputs. Every feature decision should trace back to this vision.

---

## What a VISION.md Is / Is Not

### ❌ Not
- A marketing document or sales pitch
- Implementation details or technical architecture
- A comprehensive feature list or roadmap
- Vague aspirations without falsifiable criteria

### ✅ Is
- Clear problem statement with real-world context
- Defined target users and their constraints
- Concrete success metrics (qualitative and quantitative)
- Scope boundaries (what we're NOT building)
- Philosophical principles guiding design decisions

---

## Core Structure

### 1. Header

```markdown
# [Project Name] Vision

One-sentence purpose statement
```

### 2. Problem Statement

**What problem exists?**
- Real-world context (2-3 sentences)
- Who experiences this problem?
- Why do current solutions fail or fall short?
- What's the cost of this problem remaining unsolved?

### 3. Target Users

**Who is this for?**
- Primary user persona (role, context, constraints)
- Secondary users (if applicable)
- Explicit non-users (who is this NOT for?)

Example:
```markdown
**Primary**: AI-assisted developers using Claude Code for production work
**Secondary**: Team leads tracking methodology adoption
**Not for**: Solo developers resistant to structured workflows
```

### 4. Solution Approach

**How do we solve it?**
- Core insight or principle (what's different about our approach?)
- Key capabilities (3-5 bullets, outcome-focused)
- What we're NOT doing (scope boundaries)

### 5. Success Criteria

**How do we know we've succeeded?**

Separate qualitative and quantitative metrics:

**Qualitative:**
- [ ] Users report X experience improvement
- [ ] Methodology becomes self-reinforcing (users want to stay in workflow)
- [ ] Documentation quality increases measurably

**Quantitative:**
- [ ] X% reduction in context-switching overhead
- [ ] Y active projects using workflows within Z months
- [ ] Test coverage maintained above N%

### 6. Guiding Principles

**What philosophical commitments guide design decisions?**

3-5 core principles that resolve tradeoff decisions:
- Principle 1: Transparency over convenience
- Principle 2: Local-first over cloud-dependent
- Principle 3: Deterministic over AI-powered guardrails

These should be **decisive** - when two features conflict, principles resolve the tension.

---

## Quality Heuristics

High-quality VISIONs are:
- **Specific** — "reduce context overhead" not "make development better"
- **Falsifiable** — success criteria are testable
- **Opinionated** — clear boundaries and non-goals
- **User-focused** — emphasizes outcomes over features
- **Principle-driven** — provides decision-making framework

Low-quality VISIONs are:
- Vague ("build a great tool")
- Feature-list dumps
- No clear problem articulation
- Missing success criteria
- All upside, no tradeoffs acknowledged

---

## Adversarial Review Questions

Before finalizing your VISION.md, ask yourself:

1. **Problem clarity**: Could someone unfamiliar with the space understand why this matters?
2. **User specificity**: Are target users concrete enough to make design decisions?
3. **Success falsifiability**: Could we objectively determine if we've succeeded?
4. **Scope boundaries**: Have we said what we're NOT building?
5. **Principle decisiveness**: Would our principles resolve real tradeoff conflicts?
6. **Missing perspectives**: What user constraints or problem aspects am I overlooking?

---

## Common Pitfalls

**Too broad**: "Improve developer productivity" → Be specific about which developers, which productivity bottleneck

**Feature-focused**: "Build X, Y, Z features" → Focus on outcomes those features enable

**No tradeoffs**: Every design has costs → Acknowledge what you're NOT optimizing for

**Metrics-only**: Numbers without qualitative context → Balance quantitative and qualitative success

---

## Example Fragment

```markdown
## Problem Statement

AI-assisted development tools (Claude, Cursor, Codex) enable rapid code generation, but methodology enforcement lags behind. Developers context-switch frequently between "planning mode" and "coding mode" without clear phase boundaries, leading to:

- Specification drift (code diverges from intent)
- Missing documentation (generated code, no captured learnings)
- Test-last development (TDD discipline breaks down)

Current solutions (IDE extensions, linting rules) are either too rigid (blocking workflow) or too passive (ignored warnings). The cost: technical debt accumulates invisibly, and onboarding new AI assistants requires reconstructing lost context.

## Target Users

**Primary**: Professional developers using Claude Code for production codebases, working on teams where documentation and test coverage matter.

**Not for**: Solo hobbyists, developers allergic to process, projects where "move fast and break things" is still the philosophy.
```

---

## Conclusion

A VISION.md is not a manifesto.
It is a **minimal, decisive contract** about what problem we're solving, who we're solving it for, and how we'll know if we've succeeded. Every feature, every architecture decision should trace back to this document.
