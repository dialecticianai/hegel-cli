# Workflow `done` Node Analysis

Analysis of all workflow `done` nodes to guide refactoring. Goal: Remove prompts from `done` nodes and create penultimate phases where meaningful work is required.

---

## Discovery Workflow

**Current `done` node (lines 98-113):**
```yaml
done:
  prompt: |
    Discovery workflow complete.

    {{README_WRITING}}

    Final task: Create a README.md summarizing the toy model for future AI context refresh.

    Keep it concise (100-200 words):
    - What the system does
    - Key APIs/interfaces
    - Usage examples
    - Integration points

    Respond with your completed README.md content.
  transitions: []
```

**Analysis:**
- ✅ **Meaningful deliverable**: README.md creation
- **Action**: Create new `readme` phase before `done`
- Move README_WRITING guide and instructions to `readme` phase
- Keep `done` prompt minimal/informational only

**Proposed structure:**
```
learnings → readme → done
```

---

## Execution Workflow

**Current `done` node (lines 132-150):**
```yaml
done:
  prompt: |
    Execution workflow complete.

    {{README_WRITING}}

    Final task: Create production-ready README.md.

    Include:
    - System overview and purpose
    - Installation and setup
    - API documentation
    - Usage examples
    - Error handling guide
    - Performance characteristics
    - Integration points

    Respond with your completed README.md content.
  transitions: []
```

**Analysis:**
- ✅ **Meaningful deliverable**: Production-ready README.md
- **Action**: Create new `readme` phase before `done`
- More comprehensive than discovery (installation, error handling, performance)
- Keep `done` prompt minimal/informational only

**Proposed structure:**
```
review → readme → done (if review_complete)
refactor → code → ... (if needs_refactor)
```

---

## Research Workflow

**Current `done` node (lines 112-136):**
```yaml
done:
  prompt: |
    Research mode complete.

    Your deliverables:
    - Learning documents (learnings/*.md) - synthesized knowledge
    - Meta-assessments (learnings/.ddd/*.md) - progress tracking
    - Open questions (learnings/.ddd/*_open_questions.md) - Discovery roadmap
    - Cached sources (.webcache/) - stable offline references

    Final check:
    - All priority areas studied?
    - Key concepts documented?
    - Questions catalogued?
    - Ready for practical validation (Discovery mode)?

    Your deliverables:
    - Learning documents (learnings/*.md)
    - Meta-assessments (learnings/.ddd/*.md)
    - Open questions (learnings/.ddd/*_open_questions.md)
    - Cached sources (.webcache/)

    Run 'hegel next' to transition to Discovery mode.
  transitions: []
```

**Analysis:**
- ❌ **No specific deliverable** - just summarizes what was already done
- Deliverables list is duplicated (appears twice)
- "Final check" questions are vague, not actionable
- **Action**: Remove prompt entirely (just informational recap)

**Proposed structure:**
```
questions → done
```

**Rationale**: All work complete in `questions` phase. `done` should be silent terminal node.

---

## Minimal Workflow

**Current `done` node (line 12):**
```yaml
plan:
  prompt: "Write a PLAN.md with testable steps."
  transitions: []
```

**Analysis:**
- ❌ **No `done` node** - workflow ends at `plan` phase
- This is actually fine for minimal workflow (testing purposes)
- **Action**: No change needed OR add empty `done` node for consistency

**Current structure:**
```
spec → plan (terminal)
```

**Proposed structure (for consistency):**
```
spec → plan → done (empty prompt)
```

---

## Init-Greenfield Workflow

**Current `done` node (lines 120-130):**
```yaml
done:
  prompt: |
    Greenfield project initialized successfully.

    Next steps:
    - Review CLAUDE.md, VISION.md, ARCHITECTURE.md
    - Run 'hegel start discovery' to begin first feature exploration

    Project initialization complete.
  transitions: []
```

**Analysis:**
- ❌ **No specific deliverable** - just informational
- All real work done in `git_init` phase
- **Action**: Remove prompt (or make even more minimal)

**Current structure:**
```
architecture → git_init → done
```

**Proposed structure:**
```
architecture → git_init → done (empty or minimal prompt)
```

---

## Init-Retrofit Workflow

**Current `done` node (lines 224-235):**
```yaml
done:
  prompt: |
    DDD retrofit complete.

    Next steps:
    - Review CLAUDE.md, VISION.md, ARCHITECTURE.md, CODE_MAP.md files
    - Run 'hegel start discovery' to explore first refactoring or feature
    - If on feature branch: consider creating PR for team review

    Retrofit initialization finished successfully.
  transitions: []
```

**Analysis:**
- ❌ **No specific deliverable** - just informational
- All real work done in `git_commit` phase
- **Action**: Remove prompt (or make even more minimal)

**Current structure:**
```
architecture → git_commit → done
```

**Proposed structure:**
```
architecture → git_commit → done (empty or minimal prompt)
```

---

## Summary Table

| Workflow | Current `done` Prompt | Has Deliverable? | Refactor Action |
|----------|----------------------|------------------|-----------------|
| **discovery** | README creation instructions | ✅ Yes | Create `readme` phase, move instructions there |
| **execution** | Production README instructions | ✅ Yes | Create `readme` phase, move instructions there |
| **research** | Deliverables recap + questions | ❌ No | Remove prompt entirely (silent terminal) |
| **minimal** | N/A (no done node) | ❌ No | Add empty `done` node for consistency |
| **init-greenfield** | "Next steps" guidance | ❌ No | Remove prompt or make minimal |
| **init-retrofit** | "Next steps" guidance | ❌ No | Remove prompt or make minimal |

---

## Refactoring Plan

### Phase 1: Create penultimate phases where needed
1. **discovery.yaml**: Add `readme` phase after `learnings`
2. **execution.yaml**: Add `readme` phase after `review`

### Phase 2: Clean up `done` nodes
3. **discovery.yaml**: Remove instructions from `done`, leave minimal completion message
4. **execution.yaml**: Remove instructions from `done`, leave minimal completion message
5. **research.yaml**: Remove prompt entirely OR single-line completion message
6. **minimal.yaml**: Add `done` node with empty prompt for consistency
7. **init-greenfield.yaml**: Simplify `done` to minimal completion message
8. **init-retrofit.yaml**: Simplify `done` to minimal completion message

### Phase 3: Add validation rule
9. Implement workflow schema validation: `done` nodes MUST NOT have prompts (or only minimal informational text)

---

## Notes

**Prompts vs Informational Text:**
- **Prompt** = Instructs agent to perform work (write docs, make decisions, create files)
- **Informational** = Passive summary or completion message (no agent action required)

**Criteria for penultimate phase:**
- Does `done` prompt ask agent to create a deliverable?
- Does it contain {{GUIDE}} template injection?
- Does it have bulleted requirements or checklists?
- If YES to any → extract to penultimate phase

**`done` node purpose:**
- Terminal node for workflow graph
- Optional minimal completion message
- NO agent work required
- Enables clean meta-mode transitions
