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

## Mode-Specific Guidance

### Greenfield Mode

**Context**: New project, defining vision from scratch

**Approach**:
- Work with user to articulate the problem space
- Define target users and success criteria collaboratively
- No existing documentation to integrate with
- Vision can be aspirational but must be concrete

**Key questions to ask**:
1. "What problem are we solving?" (get specific, real-world context)
2. "Who is this for?" (primary and secondary users)
3. "What does success look like?" (qualitative and quantitative)
4. "What are we explicitly NOT building?" (scope boundaries)
5. "What principles guide design tradeoffs?" (3-5 core principles)

**Adversarial check**: "What assumptions about users or the problem am I making? What perspectives am I missing?"

### Retrofit Mode

**Context**: Existing project, may have implicit or explicit vision

**Approach**:
- **First, check for existing vision documentation**
  - README.md often contains problem statement
  - docs/ may have vision or goals documents
  - Issue trackers or project boards may reveal priorities
- **Extract and formalize** if documented elsewhere
- **Collaborate with user to define** if missing or unclear
- **Update if evolved** — projects change, vision should reflect current direction

**Key considerations**:
- Existing users have expectations (don't pivot without acknowledging)
- Current feature set implies scope decisions
- Technical debt reflects past tradeoffs
- Team may have unwritten shared understanding

**Key questions to ask**:
1. "Does existing documentation capture project vision?"
   - Check README.md, docs/, project homepage
2. "If yes: Should I formalize it into VISION.md?"
   - Extract and structure existing content
3. "If no: [Ask standard vision questions from greenfield]"
   - But frame in context of existing codebase
4. "Has the project vision evolved? Any updates needed?"
   - Original goals may have shifted
   - New users or use cases may have emerged

**Adversarial check**: "Am I missing existing project context? Does my vision align with current direction? Did I check issue trackers or roadmaps for stated goals?"

**Example retrofit flow**:
```
Agent: "I found a README.md with this problem statement: [quote]
and these goals: [quote]. Should I formalize these into VISION.md,
or do they need updating?"

User: "Those are outdated, we pivoted last year."

Agent: "Understood. Let's define the current vision:
1. What problem does the project solve now?
2. Who uses it currently?
3. What does success look like today?
..."
```

---

## Common Pitfalls

**Too broad**: "Improve developer productivity" → Be specific about which developers, which productivity bottleneck

**Feature-focused**: "Build X, Y, Z features" → Focus on outcomes those features enable

**No tradeoffs**: Every design has costs → Acknowledge what you're NOT optimizing for

**Metrics-only**: Numbers without qualitative context → Balance quantitative and qualitative success

**Ignoring existing context (retrofit)**: Don't write greenfield vision for mature project → Ground vision in current reality

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
