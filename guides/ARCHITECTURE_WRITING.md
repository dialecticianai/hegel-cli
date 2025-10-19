# Meta-Document: How to Write an Effective ARCHITECTURE.md

_Guide to defining technical architecture and dependencies in the Dialectic-Driven Development paradigm._

---

## When to Use This

**Phase**: Project initialization (greenfield or retrofit)
**Location**: Root of project directory (`ARCHITECTURE.md`)
**Sequence**: Write after VISION.md, before starting Discovery workflows
**Purpose**: Define tech stack, key architectural decisions, and dependencies to explore during Discovery phase

---

## Purpose

An **ARCHITECTURE.md is a technical foundation document**: it captures the core technical decisions, their rationale, and open questions requiring investigation.

It exists to:
- Prevent architecture drift across multiple AI-assisted sessions
- Document decision rationale (why this stack/approach?)
- Surface dependencies and unknowns for Discovery phase exploration
- Provide technical context for future AI assistants joining the project

---

## What an ARCHITECTURE.md Is / Is Not

### ❌ Not
- Detailed implementation plans (that's PLAN.md's job)
- Complete system design documentation
- API reference or code documentation
- A list of every possible dependency

### ✅ Is
- Core technology choices (language, framework, key libraries)
- Key architectural patterns and their rationale
- Known constraints (performance, compatibility, security)
- Open questions requiring investigation
- Integration points and boundaries
- Dependencies to explore during Discovery

---

## Core Structure

### 1. Header

```markdown
# [Project Name] Architecture

Tech stack and architectural decisions
```

### 2. Technology Stack

**What are we building with?**

```markdown
**Language**: Rust (stable)
**Rationale**: Memory safety, zero-cost abstractions, strong type system

**Key Dependencies**:
- serde (serialization) - established, stable API
- anyhow (error handling) - ergonomic, minimal boilerplate
- clap (CLI parsing) - feature-rich, well-documented

**To Explore** (Discovery phase):
- ratatui vs crossterm for TUI features
- toml vs yaml for config format
- Alternative state persistence approaches
```

### 3. Core Architectural Decisions

**What are the foundational patterns?**

Document 3-5 key decisions with rationale:

```markdown
## Decision: File-Based State Persistence

**Choice**: JSON files in `.hegel/` directory
**Rationale**:
- Inspectable (users can examine state directly)
- Local-first (no network dependencies)
- Git-friendly (state changes visible in diffs)
- Portable (no platform-specific formats)

**Tradeoffs**:
- Not suitable for high-frequency updates
- No concurrent access guarantees
- Manual consistency management

**Alternatives considered**:
- SQLite: More robust but less transparent
- In-memory only: Loses state on crash
- Custom binary format: Faster but opaque
```

### 4. System Boundaries

**What are the integration points?**

Define what's inside vs outside your system:

```markdown
**Internal**:
- Workflow state machine
- YAML workflow parser
- Guide template injection
- Metrics extraction

**External** (integration points):
- Claude Code hooks (stdin JSON events)
- User's git repository
- User's filesystem (project files)
- Terminal output (agent-readable)
```

### 5. Known Constraints

**What technical limitations exist?**

Be explicit about constraints that guide design:

```markdown
**Platform**: macOS/Linux initially, Windows future
**Context limits**: Workflow prompts must fit in ~4k tokens
**Performance**: Init commands <100ms, analysis <5s
**Compatibility**: Must work with existing git workflows
**Security**: No network access, local files only
```

### 6. Open Questions

**What needs investigation?**

Surface unknowns for Discovery phase:

```markdown
**To Investigate**:
- [ ] Can tree-sitter parse YAML reliably for workflow validation?
- [ ] What's the optimal JSONL chunk size for metrics parsing?
- [ ] How do we detect project language when multiple exist?
- [ ] Should workflows support conditional transitions (if/else)?
- [ ] Performance characteristics of recursive file walking for detection
```

### 7. Non-Functional Requirements

**What quality attributes matter?**

```markdown
**Reliability**: Crashes must not corrupt state
**Performance**: Sub-second for common operations
**Maintainability**: <200 lines per implementation module
**Testability**: ≥80% line coverage
**Portability**: Minimal platform-specific code
```

---

## Quality Heuristics

High-quality ARCHITECTUREs are:
- **Decisive** — clear choices with rationale, not "we could do X or Y"
- **Honest about tradeoffs** — acknowledges costs of decisions
- **Exploration-aware** — surfaces unknowns for Discovery
- **Constraint-explicit** — documents limitations upfront
- **Boundary-clear** — defines what's in/out of scope

Low-quality ARCHITECTUREs are:
- Vague ("we'll use modern best practices")
- All upside, no tradeoff acknowledgment
- Missing rationale (decisions without "why")
- No open questions (false certainty)
- Scope creep (trying to document everything)

---

## Adversarial Review Questions

Before finalizing your ARCHITECTURE.md, ask yourself:

1. **Decision clarity**: Could a new developer understand why we chose this stack?
2. **Tradeoff honesty**: Have we acknowledged costs, not just benefits?
3. **Constraint completeness**: What technical limitations are we missing?
4. **Boundary precision**: Are integration points clearly defined?
5. **Discovery readiness**: Do we have concrete questions for exploration?
6. **Alternative consideration**: Did we evaluate alternatives or just pick favorites?

---

## Common Pitfalls

**Technology tourism**: Listing every cool library without rationale → Focus on justified choices

**Over-specification**: Designing every module upfront → Document decisions, not implementation

**No unknowns**: Pretending everything is figured out → Surface questions honestly

**Missing tradeoffs**: Only listing benefits → Every decision has costs

**Scope ambiguity**: Unclear what's in vs out → Define boundaries explicitly

---

## Mode-Specific Guidance

### Greenfield Projects

- Start minimal (language + 2-3 core dependencies)
- Emphasize exploration questions
- Expect this document to evolve during Discovery
- Don't over-commit to dependencies before validation

### Retrofit Projects

- Document existing architecture first
- Identify integration constraints
- Note migration considerations
- Highlight compatibility requirements with existing stack

---

## Example Fragment

```markdown
## Decision: Embedded Workflows vs External Files

**Choice**: Embed workflow YAMLs in binary at compile time

**Rationale**:
- Zero-config bootstrapping (`hegel init` works immediately)
- Version coupling (workflows match CLI version)
- No "missing workflow file" errors
- Single binary distribution

**Tradeoffs**:
- Users can't modify core workflows without rebuilding
- Larger binary size (~20kb for all workflows)
- No runtime workflow updates

**Alternatives considered**:
- `~/.hegel/workflows/`: Solves customization but breaks zero-config
- Hybrid (embedded + override): Added complexity, deferred

**Future refinement**: May add workflow override mechanism once user demand surfaces
```

---

## Conclusion

An ARCHITECTURE.md is not a detailed design document.
It is a **technical foundation** that captures core decisions, their rationale, and open questions. It provides enough context for AI assistants to make coherent implementation choices while surfacing uncertainties that require Discovery phase investigation.
