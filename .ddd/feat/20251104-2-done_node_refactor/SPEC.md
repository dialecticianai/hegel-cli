# Workflow Done Node Refactor Specification

Refactor workflow YAML files to separate agent instructions from terminal nodes.

## Overview

**What it does:** Ensures workflow `done` nodes are silent terminals (no agent instructions). Meaningful work moves to penultimate phases.

**Key principles:**
- Terminal nodes should not prompt agent action
- Separate concerns: work phases vs. completion markers
- Enable clean meta-mode transitions

**Scope:** Refactor 6 workflow YAML files to extract instructions from `done` nodes.

**Integration:** Changes workflow definitions loaded by engine/mod.rs.

## Data Model

**Workflow YAML structure (relevant fields):**
```yaml
nodes:
  phase_name:
    prompt: "Agent instructions"  # Required for work phases
    transitions:
      - when: "claim"
        to: "next_node"

  done:
    prompt: ""  # MUST be empty or minimal (new constraint)
    transitions: []  # Terminal node
```

## Core Operations

### Operation: Validate Workflow Schema

**Behavior:** When loading workflow YAML, validate that `done` nodes have no prompts or only minimal informational text (≤140 chars, no {{GUIDE}} templates, no bullet lists).

**Validation rules:**
- `done` node prompt MUST be empty string OR
- `done` node prompt ≤140 chars AND no `{{` template markers AND no `- ` bullet lists

**Error handling:** Return validation error with workflow name and current `done` prompt length if invalid.

### Operation: Refactor Workflow File

**Syntax:** Manual YAML editing (no CLI command)

**For workflows with deliverables (discovery, execution):**
1. Create new penultimate phase (e.g., `readme`)
2. Move `done` prompt content to new phase
3. Update transitions: `previous_phase → readme → done`
4. Clear `done` prompt to empty string

**For workflows without deliverables (research, init-*):**
1. Remove `done` prompt content
2. Leave empty string or minimal 1-line message

## Test Scenarios

### Simple: Workflow loads with empty done node
- Load `minimal.yaml` with empty `done` prompt
- Expect: No validation errors

### Complex: Workflow with penultimate phase
- Load `discovery.yaml` with `readme` phase before `done`
- Expect: Transitions work, `done` has no prompt

### Error: Invalid done node
- Load workflow with 200-char `done` prompt containing bullets
- Expect: Validation error with clear message

## Success Criteria

- [ ] All 6 workflows have empty or minimal `done` prompts (≤140 chars)
- [ ] Discovery and execution workflows have `readme` penultimate phase
- [ ] Workflow validation rejects `done` nodes with substantive prompts
- [ ] All existing tests pass after refactor
- [ ] No breaking changes to workflow engine behavior
