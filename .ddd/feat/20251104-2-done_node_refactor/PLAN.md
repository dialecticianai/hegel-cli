# Workflow Done Node Refactor Plan

## Overview

Refactor workflow YAML files to move agent instructions out of `done` nodes into penultimate phases. Add validation to enforce empty `done` prompts.

**Scope:** 6 workflow files + validation logic in engine

**Methodology:** Manual YAML refactoring with comprehensive validation testing. No behavior changes to workflow execution.

---

## Step 1: Add Workflow Validation

### Goal
Reject workflows where `done` nodes have prompt fields.

### Step 1.a: Write Tests
- Test workflow with `done` prompt is rejected

### Step 1.b: Implement
- Check `done` node has no prompt field when loading workflow
- Return error if prompt field exists

### Success Criteria
- [ ] Workflow loading rejects `done` nodes with prompts

---

## Step 2: Refactor Discovery Workflow

### Goal
Move README instructions to new `readme` phase.

### Step 2.a: Update YAML
- Create `readme` node with `done` prompt content
- Update transitions: `learnings → readme → done`
- Remove `prompt` field from `done` node

### Success Criteria
- [ ] Workflow loads and transitions work correctly

---

## Step 3: Refactor Execution Workflow

### Goal
Move README instructions to new `readme` phase.

### Step 3.a: Update YAML
- Create `readme` node with `done` prompt content
- Update transitions: `review → readme → done` (review_complete path)
- Remove `prompt` field from `done` node

### Success Criteria
- [ ] Workflow loads and both paths work correctly

---

## Step 4: Clean Up Other Workflows

### Goal
Remove `prompt` fields from `done` nodes.

### Step 4.a: Update YAMLs
- Research, minimal, init-greenfield, init-retrofit: Remove `prompt` field from `done`

### Success Criteria
- [ ] All workflows pass validation

---

## Step 5: Integration Testing

### Goal
Verify all workflows work correctly.

### Step 5.a: Test
- Run full test suite
- Manually test discovery and execution workflows

### Success Criteria
- [ ] All tests pass
- [ ] Workflows execute correctly
