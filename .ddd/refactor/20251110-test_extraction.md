# Refactor Analysis: Test Code Extraction

**Date**: 2025-11-10
**Type**: SoC violation - test code mixed with implementation

---

## Problem Statement

Six module files violate Separation of Concerns by mixing implementation and test code, creating large files that are inefficient to read and navigate:

| File | Total | Impl | Test | Test % |
|------|-------|------|------|--------|
| engine/mod.rs | 1,392 | 326 | 1,066 | 77% |
| storage/mod.rs | 1,227 | 587 | 640 | 52% |
| metrics/mod.rs | 913 | 372 | 541 | 59% |
| engine/template.rs | 676 | 162 | 514 | 76% |
| engine/handlebars.rs | 512 | 160 | 352 | 69% |
| metrics/git.rs | 497 | 158 | 339 | 68% |
| **Total** | **5,217** | **1,765** | **3,452** | **66%** |

**Token cost**: Every file read loads 3,452 lines of test code unnecessarily when only implementation is needed.

---

## SoC Violations Identified

1. **Mixed concerns in single files**: Implementation and test code colocated
2. **Poor navigability**: 500-1,400 line files difficult to navigate
3. **Unnecessary token overhead**: Tests loaded even when only implementation needed
4. **Inconsistent with codebase pattern**: Other modules (rules, analyze, commands/workflow) already use `/tests/` subdirectories

---

## Existing Pattern

The codebase already has a clean test extraction pattern:

```
src/rules/
  ├── mod.rs (impl only)
  └── tests/
      ├── mod.rs (just module declarations)
      └── evaluator.rs (extracted tests)

src/analyze/
  ├── mod.rs (impl only)
  └── tests/
      ├── mod.rs
      └── gap_detection.rs

src/commands/workflow/
  ├── *.rs (impl only)
  └── tests/
      ├── mod.rs
      ├── commands.rs
      ├── transitions.rs
      ├── stash.rs
      └── ...
```

---

## Proposed Refactoring

### 1. engine/mod.rs (1,066 test lines)

**Create**: `src/engine/tests/`

**Test groupings** (based on typical engine concerns):
- `state_machine.rs` - Core state transitions, node traversal
- `discovery.rs` - Discovery mode tests
- `execution.rs` - Execution mode tests
- `validation.rs` - Workflow validation tests
- `mod.rs` - Module declarations

**Result**: engine/mod.rs reduced to ~326 lines (impl only)

---

### 2. storage/mod.rs (640 test lines)

**Create**: `src/storage/tests/`

**Test groupings**:
- `state_persistence.rs` - State JSON read/write tests
- `jsonl_logs.rs` - JSONL append/read tests (hooks, states, transcript)
- `locking.rs` - File locking tests
- `migration.rs` - State migration tests (if any)
- `mod.rs` - Module declarations

**Result**: storage/mod.rs reduced to ~587 lines (impl only)

---

### 3. metrics/mod.rs (541 test lines)

**Create**: `src/metrics/tests/`

**Test groupings**:
- `hooks.rs` - Hook metrics parsing tests
- `transcript.rs` - Transcript metrics parsing tests
- `states.rs` - State metrics parsing tests
- `integration.rs` - Cross-metrics tests
- `mod.rs` - Module declarations

**Result**: metrics/mod.rs reduced to ~372 lines (impl only)

---

### 4. engine/template.rs (514 test lines)

**Create**: `src/engine/tests/` (shared with engine/mod.rs)

**Test groupings**:
- `template.rs` - Template expansion, placeholder handling tests

**Result**: engine/template.rs reduced to ~162 lines (impl only)

---

### 5. engine/handlebars.rs (352 test lines)

**Create**: `src/engine/tests/` (shared with engine/mod.rs)

**Test groupings**:
- `handlebars.rs` - Handlebars integration tests

**Result**: engine/handlebars.rs reduced to ~160 lines (impl only)

---

### 6. metrics/git.rs (339 test lines)

**Create**: `src/metrics/tests/` (shared with metrics/mod.rs)

**Test groupings**:
- `git.rs` - Git metrics extraction tests

**Result**: metrics/git.rs reduced to ~158 lines (impl only)

---

## Expected Impact

### Token Efficiency
- **Before**: Reading any of these 6 files loads 3,452 test lines
- **After**: Implementation files load only 1,765 lines (53% reduction)
- **Savings**: 1,687 lines of unnecessary test code not loaded when reading impl

### File Size Reduction
- 6 large files (497-1,392 lines) → 6 focused impl files (158-587 lines)
- Average file size: 869 lines → 294 lines (66% reduction)
- All files now under 600 lines (well under 200-line splitting threshold for impl)

### Consistency
- Aligns with existing codebase pattern (rules, analyze, commands/workflow)
- Clear SoC: implementation vs. tests
- Better navigability and discoverability

---

## Implementation Steps

1. **For each module** (engine, storage, metrics):
   - Create `src/<module>/tests/` directory
   - Create `tests/mod.rs` with submodule declarations
   - Extract tests to focused files by logical grouping
   - Remove test code from implementation files
   - Verify `cargo test` passes after each extraction

2. **Order**: Start with smallest (metrics/git.rs) to validate pattern, then proceed to larger files

3. **Validation**: Run `cargo test` after each file extraction to ensure no breakage

---

## Out of Scope

- Adding new test coverage (extraction only, no additions)
- Refactoring test logic (preserve as-is)
- Changing test helper patterns (use existing patterns)
- Performance optimization
- Functionality changes

---

## Approval Needed

Confirm:
1. Test grouping strategy matches expectations
2. Order of extraction (smallest to largest)
3. Ready to proceed with implementation
